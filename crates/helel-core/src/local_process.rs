//! Supervised newline-delimited local child-process transport.

use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, mpsc};
use std::time::Duration;

pub struct JsonLineProcess {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    output: Mutex<mpsc::Receiver<String>>,
}

impl JsonLineProcess {
    /// Starts a direct local executable without a command shell.
    ///
    /// # Errors
    /// Returns an error when the process or its standard streams cannot be opened.
    pub fn start(program: &str, args: &[String]) -> io::Result<Self> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("local process stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("local process stdout unavailable"))?;
        let (sender, receiver) = mpsc::sync_channel(256);
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            output: Mutex::new(receiver),
        })
    }

    /// Sends one bounded JSON line and waits for one response line.
    ///
    /// # Errors
    /// Returns an error for oversized messages, broken pipes, timeouts, or oversized responses.
    pub fn request(&self, json: &str, timeout: Duration) -> io::Result<String> {
        self.send(json)?;
        self.receive(timeout)
    }

    /// Sends one bounded JSON line.
    ///
    /// # Errors
    /// Returns an error for invalid input or a broken pipe.
    pub fn send(&self, json: &str) -> io::Result<()> {
        if json.contains('\n') || json.len() > 1_048_576 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid JSON-line request size",
            ));
        }
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| io::Error::other("local process stdin lock unavailable"))?;
        writeln!(stdin, "{json}")?;
        stdin.flush()?;
        drop(stdin);
        Ok(())
    }

    /// Receives one bounded response line.
    ///
    /// # Errors
    /// Returns an error on timeout, closed transport, or oversized output.
    pub fn receive(&self, timeout: Duration) -> io::Result<String> {
        let response = self
            .output
            .lock()
            .map_err(|_| io::Error::other("local process output lock unavailable"))?
            .recv_timeout(timeout)
            .map_err(|_| {
                io::Error::new(io::ErrorKind::TimedOut, "local process response timed out")
            })?;
        if response.len() > 1_048_576 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "local process response is oversized",
            ));
        }
        Ok(response)
    }

    /// Stops the supervised process.
    ///
    /// # Errors
    /// Returns an error if process state cannot be accessed or termination fails.
    pub fn stop(&self) -> io::Result<()> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| io::Error::other("local process lock unavailable"))?;
        if child.try_wait()?.is_none() {
            child.kill()?;
        }
        let _ = child.wait();
        Ok(())
    }
}

impl Drop for JsonLineProcess {
    fn drop(&mut self) {
        if let Ok(child) = self.child.get_mut() {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trips_and_stops_local_process() {
        let process = JsonLineProcess::start("/bin/cat", &[]).unwrap();
        assert_eq!(
            process
                .request(r#"{"health":true}"#, Duration::from_secs(1))
                .unwrap(),
            r#"{"health":true}"#
        );
        process.stop().unwrap();
    }
}
