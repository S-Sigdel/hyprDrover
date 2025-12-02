pub mod hypr_commands;
pub mod hypr_listener;

// Re-export commonly used types
pub use hypr_commands::{
    HyprCommandClient, HyprCommandError, dispatch, get_json, send_hypr_command,
};

pub use hypr_listener::{HyprEvent, IpcEventListener};
