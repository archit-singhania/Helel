//! Interactive bounded PTY sessions.

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::{
    io::{self, Read, Write},
    path::Path,
    sync::{Mutex, mpsc},
};

const MAX_DRAIN_BYTES: usize = 256 * 1024;

pub struct TerminalSession {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    output: Mutex<mpsc::Receiver<Vec<u8>>>,
}

impl TerminalSession {
    /// Starts a direct program in a native pseudo-terminal.
    ///
    /// # Errors
    /// Returns an error when the PTY or process cannot be created.
    pub fn start(
        root: &Path,
        program: &str,
        args: &[String],
        cols: u16,
        rows: u16,
    ) -> io::Result<Self> {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)?;
        let mut command = CommandBuilder::new(program);
        command.args(args);
        command.cwd(root);
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(io::Error::other)?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let writer = pair.master.take_writer().map_err(io::Error::other)?;
        let (sender, receiver) = mpsc::sync_channel(256);
        std::thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => {
                        if sender.send(buffer[..count].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(Self {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
            output: Mutex::new(receiver),
        })
    }
    /// Writes raw terminal input.
    ///
    /// # Errors
    /// Returns an error when the session is unavailable or its pipe closes.
    pub fn write(&self, input: &[u8]) -> io::Result<()> {
        if input.len() > 65_536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "terminal input is oversized",
            ));
        }
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| io::Error::other("terminal input lock unavailable"))?;
        writer.write_all(input)?;
        writer.flush()
    }
    /// Drains currently buffered output up to the history transport limit.
    ///
    /// # Errors
    /// Returns an error when terminal output state is unavailable.
    pub fn drain(&self) -> io::Result<Vec<u8>> {
        let receiver = self
            .output
            .lock()
            .map_err(|_| io::Error::other("terminal output lock unavailable"))?;
        let mut result = Vec::new();
        while let Ok(chunk) = receiver.try_recv() {
            let remaining = MAX_DRAIN_BYTES.saturating_sub(result.len());
            result.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
            if result.len() >= MAX_DRAIN_BYTES {
                break;
            }
        }
        Ok(result)
    }
    /// Resizes the pseudo-terminal.
    ///
    /// # Errors
    /// Returns an error for zero dimensions or a backend resize failure.
    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        if cols == 0 || rows == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "terminal dimensions must be positive",
            ));
        }
        self.master
            .lock()
            .map_err(|_| io::Error::other("terminal lock unavailable"))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)
    }
    /// Terminates the terminal child.
    ///
    /// # Errors
    /// Returns an error when the child cannot be accessed or killed.
    pub fn stop(&self) -> io::Result<()> {
        self.child
            .lock()
            .map_err(|_| io::Error::other("terminal child lock unavailable"))?
            .kill()
            .map_err(io::Error::other)
    }
    /// Reports whether the PTY child is still running.
    ///
    /// # Errors
    /// Returns an error when child status cannot be queried.
    pub fn is_running(&self) -> io::Result<bool> {
        Ok(self
            .child
            .lock()
            .map_err(|_| io::Error::other("terminal child lock unavailable"))?
            .try_wait()
            .map_err(io::Error::other)?
            .is_none())
    }
}
