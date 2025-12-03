use std::fs;
use std::path::Path;
use std::error::Error;
use chrono::Local;
use crate::ipc::SessionSnapshot;

pub fn save_session_to_file(snapshot: &SessionSnapshot) -> Result<String, Box<dyn Error>> {
    // Create sessions directory if it doesn't exist
    let session_dir = Path::new("sessions");
    if !session_dir.exists() {
        fs::create_dir(session_dir)?;
    }

    // Generate filename with timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("session_{}.json", timestamp);
    let file_path = session_dir.join(&filename);

    // Serialize to JSON
    let json_string = serde_json::to_string_pretty(snapshot)?;

    // Write to file
    fs::write(&file_path, json_string)?;

    Ok(file_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{HyprClient, HyprWorkspace, HyprMonitor, HyprWorkspaceRef};
    use std::fs;

    fn create_dummy_snapshot() -> SessionSnapshot {
        SessionSnapshot {
            clients: vec![
                HyprClient {
                    address: "0xtest".to_string(),
                    at: [0, 0],
                    size: [100, 100],
                    workspace: HyprWorkspaceRef { id: 1, name: "1".to_string() },
                    class: "TestApp".to_string(),
                    title: "Test Title".to_string(),
                    initial_class: "TestApp".to_string(),
                    initial_title: "Test Title".to_string(),
                    floating: false,
                    pinned: false,
                    monitor: 0,
                    fullscreen: 0,
                    xwayland: false,
                    pid: 1000,
                }
            ],
            workspaces: vec![
                HyprWorkspace {
                    id: 1,
                    name: "1".to_string(),
                    monitor: "HDMI-A-1".to_string(),
                    windows: 1,
                    hasfullscreen: false,
                    lastwindow: "0xtest".to_string(),
                    lastwindowtitle: "Test Title".to_string(),
                }
            ],
            monitors: vec![
                HyprMonitor {
                    id: 0,
                    name: "HDMI-A-1".to_string(),
                    width: 1920,
                    height: 1080,
                    refresh_rate: 60.0,
                    x: 0,
                    y: 0,
                    active_workspace: HyprWorkspaceRef { id: 1, name: "1".to_string() },
                }
            ],
        }
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = create_dummy_snapshot();
        let json = serde_json::to_string_pretty(&snapshot).expect("Failed to serialize");
        
        assert!(json.contains("TestApp"));
        assert!(json.contains("HDMI-A-1"));
        assert!(json.contains("1920"));
    }

    #[test]
    fn test_save_session_creates_file() {
        let snapshot = create_dummy_snapshot();
        
        // This will create a file in the real sessions directory
        // We should clean it up after
        let result = save_session_to_file(&snapshot);
        assert!(result.is_ok());
        
        let path_str = result.unwrap();
        let path = Path::new(&path_str);
        
        assert!(path.exists());
        assert!(path.extension().unwrap() == "json");
        
        // Read content to verify
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("TestApp"));
        
        // Cleanup
        let _ = fs::remove_file(path);
    }
}

