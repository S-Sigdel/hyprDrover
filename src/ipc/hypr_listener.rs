use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Events emitted by Hyprland and to track
#[derive(Debug, Clone, PartialEq)]
pub enum HyprEvent {
    WorkspaceChanged {
        workspace_id: i32,
        workspace_name: String,
    },
    FocusedMonitor {
        monitor_name: String,
        workspace_name: String,
    },
    ActiveWindow {
        class: String,
        title: String,
    },
    WindowOpened {
        address: String,
        workspace: String,
        class: String,
        title: String,
    },
    WindowClosed {
        address: String,
    },
    WindowMoved {
        address: String,
        workspace: String,
    },
    WindowTitleChanged {
        address: String,
        title: String,
    },
    LayoutChanged {
        keyboard: String,
        layout: String,
    },
    SubMapChanged {
        submap: String,
    },
    Fullscreen {
        state: bool,
    },
    MonitorAdded {
        monitor_name: String,
    },
    MonitorRemoved {
        monitor_name: String,
    },
    CreateWorkspace {
        workspace_id: i32,
        workspace_name: String,
    },
    DestroyWorkspace {
        workspace_id: i32,
        workspace_name: String,
    },
    MoveWorkspace {
        workspace_id: i32,
        workspace_name: String,
        monitor: String,
    },
    Unknown {
        raw: String,
    },
}

impl HyprEvent {
    /// Parse a line from the Hyprland IPC socket into an event
    pub fn parse(line: &str) -> Self {
        // Events come in the format: "eventname>>data"
        let parts: Vec<&str> = line.splitn(2, ">>").collect();
        if parts.len() != 2 {
            return HyprEvent::Unknown {
                raw: line.to_string(),
            };
        }

        let event_name = parts[0];
        let data = parts[1];

        match event_name {
            "workspace" => {
                // Format: workspace>>WORKSPACEID,WORKSPACENAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    if let Ok(id) = fields[0].parse::<i32>() {
                        return HyprEvent::WorkspaceChanged {
                            workspace_id: id,
                            workspace_name: fields[1].to_string(),
                        };
                    }
                }
            }
            "focusedmon" => {
                // Format: focusedmon>>MONNAME,WORKSPACENAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    return HyprEvent::FocusedMonitor {
                        monitor_name: fields[0].to_string(),
                        workspace_name: fields[1].to_string(),
                    };
                }
            }
            "activewindow" => {
                // Format: activewindow>>WINDOWCLASS,WINDOWTITLE
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    return HyprEvent::ActiveWindow {
                        class: fields[0].to_string(),
                        title: fields[1].to_string(),
                    };
                }
            }
            "openwindow" => {
                // Format: openwindow>>ADDRESS,WORKSPACENAME,WINDOWCLASS,WINDOWTITLE
                let fields: Vec<&str> = data.splitn(4, ',').collect();
                if fields.len() == 4 {
                    return HyprEvent::WindowOpened {
                        address: fields[0].to_string(),
                        workspace: fields[1].to_string(),
                        class: fields[2].to_string(),
                        title: fields[3].to_string(),
                    };
                }
            }
            "closewindow" => {
                // Format: closewindow>>ADDRESS
                return HyprEvent::WindowClosed {
                    address: data.to_string(),
                };
            }
            "movewindow" => {
                // Format: movewindow>>ADDRESS,WORKSPACENAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    return HyprEvent::WindowMoved {
                        address: fields[0].to_string(),
                        workspace: fields[1].to_string(),
                    };
                }
            }
            "windowtitle" => {
                // Format: windowtitle>>ADDRESS
                return HyprEvent::WindowTitleChanged {
                    address: data.to_string(),
                    title: String::new(), // Title is not provided in event
                };
            }
            "activelayout" => {
                // Format: activelayout>>KEYBOARDNAME,LAYOUTNAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    return HyprEvent::LayoutChanged {
                        keyboard: fields[0].to_string(),
                        layout: fields[1].to_string(),
                    };
                }
            }
            "submap" => {
                // Format: submap>>SUBMAPNAME
                return HyprEvent::SubMapChanged {
                    submap: data.to_string(),
                };
            }
            "fullscreen" => {
                // Format: fullscreen>>0/1
                return HyprEvent::Fullscreen { state: data == "1" };
            }
            "monitoradded" => {
                return HyprEvent::MonitorAdded {
                    monitor_name: data.to_string(),
                };
            }
            "monitorremoved" => {
                return HyprEvent::MonitorRemoved {
                    monitor_name: data.to_string(),
                };
            }
            "createworkspace" => {
                // Format: createworkspace>>WORKSPACEID,WORKSPACENAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    if let Ok(id) = fields[0].parse::<i32>() {
                        return HyprEvent::CreateWorkspace {
                            workspace_id: id,
                            workspace_name: fields[1].to_string(),
                        };
                    }
                }
            }
            "destroyworkspace" => {
                // Format: destroyworkspace>>WORKSPACEID,WORKSPACENAME
                let fields: Vec<&str> = data.splitn(2, ',').collect();
                if fields.len() == 2 {
                    if let Ok(id) = fields[0].parse::<i32>() {
                        return HyprEvent::DestroyWorkspace {
                            workspace_id: id,
                            workspace_name: fields[1].to_string(),
                        };
                    }
                }
            }
            "moveworkspace" => {
                // Format: moveworkspace>>WORKSPACEID,WORKSPACENAME,MONNAME
                let fields: Vec<&str> = data.splitn(3, ',').collect();
                if fields.len() == 3 {
                    if let Ok(id) = fields[0].parse::<i32>() {
                        return HyprEvent::MoveWorkspace {
                            workspace_id: id,
                            workspace_name: fields[1].to_string(),
                            monitor: fields[2].to_string(),
                        };
                    }
                }
            }
            _ => {}
        }

        HyprEvent::Unknown {
            raw: line.to_string(),
        }
    }
}

/// A listener for Hyprland IPC events with callback support
pub struct IpcEventListener {
    stream: UnixStream,
}

impl IpcEventListener {
    /// Get the default Hyprland event socket path
    pub fn get_socket_path() -> Result<PathBuf, String> {
        // Hyprland stores its socket at $XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock
        let runtime_dir =
            env::var("XDG_RUNTIME_DIR").map_err(|_| "XDG_RUNTIME_DIR not set".to_string())?;

        let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;

        Ok(PathBuf::from(runtime_dir)
            .join("hypr")
            .join(signature)
            .join(".socket2.sock"))
    }

    /// Connect to the Hyprland event socket
    pub fn connect() -> std::io::Result<Self> {
        let socket_path = Self::get_socket_path()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;

        let stream = UnixStream::connect(socket_path)?;
        Ok(Self { stream })
    }

    /// Connect to a custom socket path
    pub fn connect_to(socket_path: &str) -> std::io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        Ok(Self { stream })
    }

    /// Listen for events and call the provided callback for each one
    pub fn listen<F>(&mut self, mut callback: F) -> std::io::Result<()>
    where
        F: FnMut(HyprEvent),
    {
        let reader = BufReader::new(&self.stream);

        for line in reader.lines() {
            let line = line?;
            let event = HyprEvent::parse(&line);
            callback(event);
        }

        Ok(())
    }

    /// Listen for events with a filter - only events matching the filter are passed to callback
    pub fn listen_filtered<F, P>(
        &mut self,
        mut callback: F,
        mut predicate: P,
    ) -> std::io::Result<()>
    where
        F: FnMut(HyprEvent),
        P: FnMut(&HyprEvent) -> bool,
    {
        let reader = BufReader::new(&self.stream);

        for line in reader.lines() {
            let line = line?;
            let event = HyprEvent::parse(&line);

            if predicate(&event) {
                callback(event);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace_event() {
        let event = HyprEvent::parse("workspace>>1,main");
        assert_eq!(
            event,
            HyprEvent::WorkspaceChanged {
                workspace_id: 1,
                workspace_name: "main".to_string()
            }
        );
    }

    #[test]
    fn test_parse_active_window() {
        let event = HyprEvent::parse("activewindow>>firefox,Mozilla Firefox");
        assert_eq!(
            event,
            HyprEvent::ActiveWindow {
                class: "firefox".to_string(),
                title: "Mozilla Firefox".to_string()
            }
        );
    }

    #[test]
    fn test_parse_window_opened() {
        let event = HyprEvent::parse("openwindow>>0x12345,1,kitty,Terminal");
        assert_eq!(
            event,
            HyprEvent::WindowOpened {
                address: "0x12345".to_string(),
                workspace: "1".to_string(),
                class: "kitty".to_string(),
                title: "Terminal".to_string()
            }
        );
    }

    #[test]
    fn test_parse_unknown_event() {
        let event = HyprEvent::parse("customevent>>somedata");
        match event {
            HyprEvent::Unknown { raw } => assert_eq!(raw, "customevent>>somedata"),
            _ => panic!("Expected Unknown event"),
        }
    }
}
