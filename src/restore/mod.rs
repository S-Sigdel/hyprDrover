pub mod position;
pub mod spawn;

use std::error::Error;
use crate::ipc::{self, SessionSnapshot};

/// Orchestrates the restoration of a session
pub fn restore_session(snapshot: &SessionSnapshot) -> Result<(), Box<dyn Error>> {
    // 1. Get current state
    let current_state = ipc::capture_state()?;
    let mut available_clients = current_state.clients;

    // 2. Match and restore
    for saved_client in &snapshot.clients {
        // Try to find a matching client in the current session
        if let Some(index) = available_clients.iter().position(|c| {
            c.class == saved_client.class 
        }) {
            let current_client = available_clients.remove(index);
            println!("   Restoring window: {} ({})", current_client.class, current_client.title);

            position::restore_window_position(&current_client, saved_client)?;

        } else {
            println!("   ⚠️ Window missing: {}", saved_client.class);
            
            // Notify user
            let _ = std::process::Command::new("notify-send")
                .arg("Restoring Session")
                .arg(format!("Launching {}...", saved_client.class))
                .spawn();

            // Launch app on workspace
            // Heuristic: Use initial_class or class, converted to lowercase
            let raw_name = if !saved_client.initial_class.is_empty() {
                &saved_client.initial_class
            } else {
                &saved_client.class
            };
            
            let command = resolve_command(raw_name);

            println!("      -> Launching: {}", command);
            let workspace_cmd = format!("exec [workspace {} silent] {}", saved_client.workspace.id, command);
            
            if let Err(e) = ipc::dispatch(&workspace_cmd) {
                eprintln!("Failed to launch {}: {}", command, e);
            }
        }
    }

    Ok(())
}

fn resolve_command(class: &str) -> String {
    let lower = class.to_lowercase();
    match lower.as_str() {
        "brave-browser" => "brave".to_string(),
        "code" => "code".to_string(), // VS Code often has class "Code"
        "google-chrome" => "google-chrome-stable".to_string(),
        _ => lower,
    }
}
