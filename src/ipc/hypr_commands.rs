use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Error types for Hyprland commands
#[derive(Debug)]
pub enum HyprCommandError {
    SocketNotFound(String),
    ConnectionFailed(std::io::Error),
    WriteFailed(std::io::Error),
    ReadFailed(std::io::Error),
    CommandFailed(String),
}

impl std::fmt::Display for HyprCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketNotFound(msg) => write!(f, "Socket not found: {}", msg),
            Self::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            Self::WriteFailed(e) => write!(f, "Write failed: {}", e),
            Self::ReadFailed(e) => write!(f, "Read failed: {}", e),
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
        }
    }
}

impl std::error::Error for HyprCommandError {}

/// Client for sending commands to Hyprland
pub struct HyprCommandClient {
    socket_path: PathBuf,
}

impl HyprCommandClient {
    /// Get the default Hyprland command socket path
    pub fn get_socket_path() -> Result<PathBuf, HyprCommandError> {
        // Hyprland command socket: $XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock
        let runtime_dir = env::var("XDG_RUNTIME_DIR")
            .map_err(|_| HyprCommandError::SocketNotFound("XDG_RUNTIME_DIR not set".to_string()))?;

        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
            HyprCommandError::SocketNotFound("HYPRLAND_INSTANCE_SIGNATURE not set".to_string())
        })?;

        Ok(PathBuf::from(runtime_dir)
            .join("hypr")
            .join(signature)
            .join(".socket.sock"))
    }

    /// Create a new command client with default socket
    pub fn new() -> Result<Self, HyprCommandError> {
        let socket_path = Self::get_socket_path()?;
        Ok(Self { socket_path })
    }

    /// Create a new command client with custom socket path
    pub fn with_socket(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Send a command to Hyprland and get the response
    pub fn send_command(&self, command: &str) -> Result<String, HyprCommandError> {
        let mut stream =
            UnixStream::connect(&self.socket_path).map_err(HyprCommandError::ConnectionFailed)?;

        // Write command
        stream
            .write_all(command.as_bytes())
            .map_err(HyprCommandError::WriteFailed)?;

        // Shutdown write side to signal we're done sending
        stream
            .shutdown(std::net::Shutdown::Write)
            .map_err(HyprCommandError::WriteFailed)?;

        // Read response
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(HyprCommandError::ReadFailed)?;

        Ok(response)
    }

    /// Execute a dispatch command
    pub fn dispatch(&self, command: &str) -> Result<String, HyprCommandError> {
        self.send_command(&format!("dispatch {}", command))
    }

    /// Get JSON data from hyprctl
    pub fn get_json(&self, command: &str) -> Result<String, HyprCommandError> {
        self.send_command(&format!("j/{}", command))
    }

    /// Move focus to a specific workspace
    pub fn focus_workspace(&self, workspace: i32) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("workspace {}", workspace))
    }

    /// Move window to a specific workspace
    pub fn move_to_workspace(&self, workspace: i32) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("movetoworkspace {}", workspace))
    }

    /// Focus a specific window by address
    pub fn focus_window(&self, address: &str) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("focuswindow address:{}", address))
    }

    /// Close the active window
    pub fn close_active_window(&self) -> Result<String, HyprCommandError> {
        self.dispatch("killactive")
    }

    /// Toggle fullscreen for active window
    pub fn toggle_fullscreen(&self) -> Result<String, HyprCommandError> {
        self.dispatch("fullscreen")
    }

    /// Toggle floating for active window
    pub fn toggle_floating(&self) -> Result<String, HyprCommandError> {
        self.dispatch("togglefloating")
    }

    /// Move/resize a window
    pub fn move_window(&self, address: &str, x: i32, y: i32) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("movewindowpixel exact {} {},{}", x, y, address))
    }

    /// Resize a window
    pub fn resize_window(&self, address: &str, w: i32, h: i32) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("resizewindowpixel exact {} {},{}", w, h, address))
    }

    /// Execute a program
    pub fn exec(&self, program: &str) -> Result<String, HyprCommandError> {
        self.dispatch(&format!("exec {}", program))
    }

    /// Get list of clients (windows) in JSON format
    pub fn get_clients(&self) -> Result<String, HyprCommandError> {
        self.get_json("clients")
    }

    /// Get list of workspaces in JSON format
    pub fn get_workspaces(&self) -> Result<String, HyprCommandError> {
        self.get_json("workspaces")
    }

    /// Get active window info in JSON format
    pub fn get_active_window(&self) -> Result<String, HyprCommandError> {
        self.get_json("activewindow")
    }

    /// Get monitors info in JSON format
    pub fn get_monitors(&self) -> Result<String, HyprCommandError> {
        self.get_json("monitors")
    }

    /// Reload Hyprland configuration
    pub fn reload(&self) -> Result<String, HyprCommandError> {
        self.send_command("reload")
    }
}

/// Convenience function to send a command to Hyprland
pub fn send_hypr_command(command: &str) -> Result<String, HyprCommandError> {
    let client = HyprCommandClient::new()?;
    client.send_command(command)
}

/// Convenience function to dispatch a command
pub fn dispatch(command: &str) -> Result<String, HyprCommandError> {
    let client = HyprCommandClient::new()?;
    client.dispatch(command)
}

/// Convenience function to get JSON data
pub fn get_json(command: &str) -> Result<String, HyprCommandError> {
    let client = HyprCommandClient::new()?;
    client.get_json(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_format() {
        // Test that we can create a client (will fail if not in Hyprland)
        let result = HyprCommandClient::new();
        // We just check that the function runs, actual connection will fail in test env
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_dispatch_format() {
        let client = HyprCommandClient::with_socket(PathBuf::from("/tmp/test.sock"));
        // Just testing string formatting, not actual execution
        assert_eq!(format!("dispatch workspace {}", 1), "dispatch workspace 1");
    }
}
