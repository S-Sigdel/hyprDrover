mod ipc;

use ipc::{HyprCommandClient, HyprEvent, IpcEventListener};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hyprland Session Manager - IPC Demo");
    println!("====================================\n");

    // Example 1: Send some commands
    demo_commands()?;

    // Example 2: Listen to events
    demo_event_listener()?;

    Ok(())
}

fn demo_commands() -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Command Demo:");
    println!("-----------------");

    let client = HyprCommandClient::new()?;

    // Get current workspaces
    println!("Fetching workspaces...");
    match client.get_workspaces() {
        Ok(workspaces) => println!("Workspaces JSON: {}\n", workspaces),
        Err(e) => println!("Error getting workspaces: {}\n", e),
    }

    // Get current clients (windows)
    println!("Fetching clients...");
    match client.get_clients() {
        Ok(clients) => println!("Clients JSON: {}\n", clients),
        Err(e) => println!("Error getting clients: {}\n", e),
    }

    // Get active window
    println!("Fetching active window...");
    match client.get_active_window() {
        Ok(window) => println!("Active window JSON: {}\n", window),
        Err(e) => println!("Error getting active window: {}\n", e),
    }

    Ok(())
}

fn demo_event_listener() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📥 Event Listener Demo:");
    println!("-----------------------");
    println!("Listening for Hyprland events... (Press Ctrl+C to stop)\n");

    let mut listener = IpcEventListener::connect()?;

    listener.listen(|event| match event {
        HyprEvent::WorkspaceChanged {
            workspace_id,
            workspace_name,
        } => {
            println!(
                "🖥️  Workspace changed to: {} ({})",
                workspace_name, workspace_id
            );
        }
        HyprEvent::ActiveWindow { class, title } => {
            println!("🪟  Active window: {} - {}", class, title);
        }
        HyprEvent::WindowOpened {
            address,
            workspace,
            class,
            title,
        } => {
            println!(
                "✅ Window opened: {} - {} (workspace: {}, addr: {})",
                class, title, workspace, address
            );
        }
        HyprEvent::WindowClosed { address } => {
            println!("❌ Window closed: {}", address);
        }
        HyprEvent::WindowMoved { address, workspace } => {
            println!("↔️  Window moved: {} to workspace {}", address, workspace);
        }
        HyprEvent::FocusedMonitor {
            monitor_name,
            workspace_name,
        } => {
            println!(
                "🖥️  Monitor focused: {} (workspace: {})",
                monitor_name, workspace_name
            );
        }
        HyprEvent::Fullscreen { state } => {
            println!(
                "⛶  Fullscreen: {}",
                if state { "enabled" } else { "disabled" }
            );
        }
        HyprEvent::CreateWorkspace {
            workspace_id,
            workspace_name,
        } => {
            println!(
                "➕ Workspace created: {} ({})",
                workspace_name, workspace_id
            );
        }
        HyprEvent::DestroyWorkspace {
            workspace_id,
            workspace_name,
        } => {
            println!(
                "➖ Workspace destroyed: {} ({})",
                workspace_name, workspace_id
            );
        }
        HyprEvent::Unknown { raw } => {
            println!("❓ Unknown event: {}", raw);
        }
        _ => {
            println!("ℹ️  Event: {:?}", event);
        }
    })?;

    Ok(())
}

// Alternative: Filtered event listener example
#[allow(dead_code)]
fn demo_filtered_listener() -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening only for window events...\n");

    let mut listener = IpcEventListener::connect()?;

    listener.listen_filtered(
        |event| {
            println!("Window event: {:?}", event);
        },
        |event| {
            // Only listen to window-related events
            matches!(
                event,
                HyprEvent::WindowOpened { .. }
                    | HyprEvent::WindowClosed { .. }
                    | HyprEvent::WindowMoved { .. }
                    | HyprEvent::ActiveWindow { .. }
            )
        },
    )?;

    Ok(())
}
