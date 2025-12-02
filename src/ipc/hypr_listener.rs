use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

/// A simple listener to Hyprland's IPC socket.
/// This will connect to Hyprland and print any events received.
pub struct HyprListener {
    stream: UnixStream,
}

impl HyprListener {
    /// Connect to the Hyprland IPC socket.
    /// You can tweak the socket path if your Hyprland uses a different one.
    pub fn new(socket_path: &str) -> std::io::Result<Self> {
        // Try to connect to the socket
        let stream = UnixStream::connect(socket_path)?;
        Ok(Self { stream })
    }

    /// Start listening to events and handle them.
    /// For now, we just print them, later you can add event parsing
    pub fn listen(&mut self) -> std::io::Result<()> {
        // Wrap the stream in a buffered reader for line-by-line reading
        let reader = BufReader::new(&self.stream);

        for line in reader.lines() {
            let line = line?;
            // TODO: parse the event and handle it properly
            println!("[Hypr Event] {}", line);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect() {
        // Use a fake path for testing; in real usage this should be the actual socket
        let listener = HyprListener::new("/tmp/hypr/test.sock");
        assert!(listener.is_err(), "Should fail connecting to fake socket");
    }
}
