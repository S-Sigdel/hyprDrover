use std::process::{Command, Stdio};
use std::error::Error;

pub use self::spawn_process as spawn;

/// Spawns a process in the background (detached)
/// This will be used later when we implement the "Restore" feature
pub fn spawn_process(command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
