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
    /// Rows scrolled back from the live bottom. 0 means we're at the live
    /// edge and new output is visible immediately.
    scrollback_offset: usize,
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
            scrollback_offset: 0,
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
        // Sending input typically scrolls back to the live edge in most
        // terminal emulators; mirror that.
        self.snap_to_live();
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
        // Resizing can change the bounded scrollback; reclamp.
        let max = self.parser.screen().scrollback();
        if self.scrollback_offset > max {
            self.scrollback_offset = max;
        }
    }

    /// Drain queued PTY output into the parser and update child status.
    ///
    /// Returns `true` if any bytes were consumed or the child status
    /// transitioned during this call — useful for an adaptive sleep loop.
    pub fn drain_output(&mut self) -> bool {
        let mut activity = false;
        while let Ok(bytes) = self.output_rx.try_recv() {
            self.parser.process(&bytes);
            activity = true;
        }

        if self.alive {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.alive = false;
                    self.exit_status = Some(status.to_string());
                    activity = true;
                }
                Ok(None) => {}
                Err(err) => {
                    self.alive = false;
                    self.exit_status = Some(format!("wait error: {err}"));
                    activity = true;
                }
            }
        }
        activity
    }

    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    /// Cursor position as `(row, col, visible)`. Coordinates are zero-based
    /// within the pane's rendering area.
    pub fn cursor(&self) -> (u16, u16, bool) {
        let screen = self.parser.screen();
        let (row, col) = screen.cursor_position();
        (row, col, !screen.hide_cursor())
    }

    /// Current scrollback offset. 0 == live edge.
    pub fn scrollback_offset(&self) -> usize {
        self.scrollback_offset
    }

    pub fn snap_to_live(&mut self) {
        if self.scrollback_offset != 0 {
            self.scrollback_offset = 0;
            self.parser.screen_mut().set_scrollback(0);
        }
    }

    pub fn scroll_by(&mut self, delta: isize) {
        let new_offset = if delta >= 0 {
            self.scrollback_offset.saturating_add(delta as usize)
        } else {
            self.scrollback_offset.saturating_sub(delta.unsigned_abs())
        };
        self.set_scrollback_offset(new_offset);
    }

    pub fn scroll_to_top(&mut self) {
        // Setting a huge value is clamped to scrollback_len internally.
        self.set_scrollback_offset(usize::MAX);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.set_scrollback_offset(0);
    }

    fn set_scrollback_offset(&mut self, offset: usize) {
        self.parser.screen_mut().set_scrollback(offset);
        // Read back the value vt100 actually clamped to.
        self.scrollback_offset = self.parser.screen().scrollback();
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
