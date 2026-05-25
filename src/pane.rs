use std::io::{Read, Write};
use std::thread;

use anyhow::Context;
use crossbeam_channel::{Receiver, unbounded};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::layout::PaneId;

const SCROLLBACK_LINES: usize = 2_000;

pub struct Pane {
    id: PaneId,
    shell: String,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output_rx: Receiver<Vec<u8>>,
    parser: vt100::Parser,
    size: PtySize,
    alive: bool,
    exit_status: Option<String>,
}

impl Pane {
    pub fn spawn(id: PaneId, rows: u16, cols: u16) -> anyhow::Result<Self> {
        let size = PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        };
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(size).context("failed to open PTY")?;

        let shell = default_shell();
        let mut command = CommandBuilder::new(&shell);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        if let Ok(cwd) = std::env::current_dir() {
            command.cwd(cwd.as_os_str());
        }

        let child = pair
            .slave
            .spawn_command(command)
            .with_context(|| format!("failed to spawn shell {shell}"))?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("failed to take PTY writer")?;
        let output_rx = spawn_reader_thread(&mut reader);

        Ok(Self {
            id,
            shell,
            master: pair.master,
            writer,
            child,
            output_rx,
            parser: vt100::Parser::new(size.rows, size.cols, SCROLLBACK_LINES),
            size,
            alive: true,
            exit_status: None,
        })
    }

    pub fn id(&self) -> PaneId {
        self.id
    }

    pub fn shell_name(&self) -> &str {
        self.shell.rsplit('/').next().unwrap_or(&self.shell)
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn exit_status(&self) -> Option<&str> {
        self.exit_status.as_deref()
    }

    pub fn write_input(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        if !self.alive {
            return Ok(());
        }

        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if self.size.rows == rows && self.size.cols == cols {
            return;
        }

        self.size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let _ = self.master.resize(self.size);
        self.parser.screen_mut().set_size(rows, cols);
    }

    pub fn drain_output(&mut self) {
        while let Ok(bytes) = self.output_rx.try_recv() {
            self.parser.process(&bytes);
        }

        if self.alive {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.alive = false;
                    self.exit_status = Some(status.to_string());
                }
                Ok(None) => {}
                Err(err) => {
                    self.alive = false;
                    self.exit_status = Some(format!("wait error: {err}"));
                }
            }
        }
    }

    pub fn screen_text(&self) -> String {
        self.parser.screen().contents()
    }
}

impl Drop for Pane {
    fn drop(&mut self) {
        if self.alive {
            let _ = self.child.kill();
        }
    }
}

fn spawn_reader_thread(reader: &mut Box<dyn Read + Send>) -> Receiver<Vec<u8>> {
    let (tx, rx) = unbounded();
    let mut reader = std::mem::replace(reader, Box::new(std::io::empty()));

    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    rx
}

fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|shell| !shell.trim().is_empty())
        .unwrap_or_else(|| "/bin/sh".to_string())
}
