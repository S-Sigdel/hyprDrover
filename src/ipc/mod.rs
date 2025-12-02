//export public ipc interface
pub mod hypr_listener;
pub mod hypr_commands;

pub use hypr_listener::IpcEventListener;
pub use hypr_commands::{send_hypr_command, HyprCommandError};

