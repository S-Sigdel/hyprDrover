pub mod position;

use crate::ipc::{self, SessionSnapshot};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Orchestrates the restoration of a session
pub fn restore_session(snapshot: &SessionSnapshot) -> Result<(), Box<dyn Error>> {
    // 1. Get current state
    let current_state = ipc::capture_state()?;
    let mut available_clients = current_state.clients;

    // Baseline addresses to identify newly spawned windows after launching
    let baseline_addresses: HashSet<String> = available_clients
        .iter()
        .map(|c| c.address.clone())
        .collect();

    // Preserve the currently active workspace so restore doesn't leave you elsewhere.
    // This reflects the workspace on the currently focused monitor (where you ran the command).
    let original_workspace_id = ipc::get_active_workspace()
        .map(|ws| ws.id)
        .unwrap_or(1);

    // Track restored windows to avoid double-matching later
    let mut restored_addresses: HashSet<String> = HashSet::new();

    // 2. Restore per-workspace to allow deterministic tiling order reconstruction.
    let mut by_workspace: HashMap<i32, Vec<ipc::HyprClient>> = HashMap::new();
    for client in &snapshot.clients {
        by_workspace
            .entry(client.workspace.id)
            .or_default()
            .push(client.clone());
    }

    let mut workspace_ids: Vec<i32> = by_workspace.keys().copied().collect();
    workspace_ids.sort_unstable();

    for workspace_id in workspace_ids {
        let Some(saved_clients) = by_workspace.get(&workspace_id) else {
            continue;
        };

        // Move focus to the workspace we're restoring (best effort).
        let _ = ipc::dispatch(&format!("workspace {}", workspace_id));

        // Partition: tiling windows first (tree restore), floating/pinned after.
        let mut tiled: Vec<ipc::HyprClient> = Vec::new();
        let mut floating_or_pinned: Vec<ipc::HyprClient> = Vec::new();
        for c in saved_clients {
            if c.pinned || c.floating {
                floating_or_pinned.push(c.clone());
            } else {
                tiled.push(c.clone());
            }
        }

        if tiled.len() == 1 {
            // Single tiled window: just ensure it exists and is moved.
            let saved = &tiled[0];
            let _ = ensure_restored(
                saved,
                &mut available_clients,
                &baseline_addresses,
                &mut restored_addresses,
                Duration::from_secs(10),
            );
        } else if tiled.len() > 1 {
            // Build a balanced split tree from saved geometry and replay it using dwindle preselect.
            let rects: Vec<Rect> = tiled.iter().map(Rect::from_client).collect();
            let indices: Vec<usize> = (0..tiled.len()).collect();
            let tree = build_split_tree(&rects, &indices);
            if let Err(e) = restore_split_tree(
                &tree,
                workspace_id,
                &tiled,
                &mut available_clients,
                &baseline_addresses,
                &mut restored_addresses,
                Duration::from_secs(10),
            ) {
                eprintln!(
                    "   ⚠️ Failed to restore tiling order for workspace {}: {}",
                    workspace_id, e
                );
                // Best-effort fallback: restore remaining tiled windows without ordering.
                for saved in &tiled {
                    let _ = ensure_restored(
                        saved,
                        &mut available_clients,
                        &baseline_addresses,
                        &mut restored_addresses,
                        Duration::from_secs(10),
                    );
                }
            }
        }

        // Restore floating/pinned windows (pixel placement if floating).
        for saved in &floating_or_pinned {
            let _ = ensure_restored(
                saved,
                &mut available_clients,
                &baseline_addresses,
                &mut restored_addresses,
                Duration::from_secs(10),
            );
        }
    }

    // 3. Return to the original workspace (best effort).
    let _ = ipc::dispatch(&format!("workspace {}", original_workspace_id));

    Ok(())
}

fn resolve_command(class: &str) -> String {
    let lower = class.to_lowercase();
    match lower.as_str() {
        "brave-browser" => "brave".to_string(),
        "code" => "code".to_string(), // VS Code often has class "Code"
        "google-chrome" => "google-chrome-stable".to_string(),
        "com.mitchellh.ghostty" => "ghostty".to_string(),
        _ => lower,
    }
}

fn launched_window_matches(current: &ipc::HyprClient, saved: &ipc::HyprClient) -> bool {
    if let (Some(current_path), Some(saved_path)) = (&current.exe_path, &saved.exe_path) {
        if current_path == saved_path {
            return true;
        }
    }

    let saved_class = saved.class.to_lowercase();
    let saved_initial_class = saved.initial_class.to_lowercase();
    let current_class = current.class.to_lowercase();
    let current_initial_class = current.initial_class.to_lowercase();

    (!saved_class.is_empty()
        && (current_class == saved_class || current_initial_class == saved_class))
        || (!saved_initial_class.is_empty()
            && (current_class == saved_initial_class || current_initial_class == saved_initial_class))
}

#[derive(Debug, Clone, Copy)]
enum SplitAxis {
    X,
    Y,
}

#[derive(Debug, Clone)]
enum SplitTree {
    Leaf(usize),
    Node {
        axis: SplitAxis,
        first: Box<SplitTree>,
        second: Box<SplitTree>,
    },
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect {
    fn from_client(c: &ipc::HyprClient) -> Self {
        Self {
            x: c.at[0],
            y: c.at[1],
            w: c.size[0],
            h: c.size[1],
        }
    }

    fn center_x(&self) -> f32 {
        self.x as f32 + (self.w as f32 / 2.0)
    }

    fn center_y(&self) -> f32 {
        self.y as f32 + (self.h as f32 / 2.0)
    }
}

fn build_split_tree(rects: &[Rect], indices: &[usize]) -> SplitTree {
    if indices.len() == 1 {
        return SplitTree::Leaf(indices[0]);
    }

    let mut min_cx = f32::INFINITY;
    let mut max_cx = f32::NEG_INFINITY;
    let mut min_cy = f32::INFINITY;
    let mut max_cy = f32::NEG_INFINITY;
    for &idx in indices {
        let r = &rects[idx];
        let cx = r.center_x();
        let cy = r.center_y();
        min_cx = min_cx.min(cx);
        max_cx = max_cx.max(cx);
        min_cy = min_cy.min(cy);
        max_cy = max_cy.max(cy);
    }

    let range_x = max_cx - min_cx;
    let range_y = max_cy - min_cy;
    let axis = if range_x >= range_y {
        SplitAxis::X
    } else {
        SplitAxis::Y
    };

    let mut sorted = indices.to_vec();
    match axis {
        SplitAxis::X => sorted.sort_by(|&a, &b| rects[a].center_x().partial_cmp(&rects[b].center_x()).unwrap()),
        SplitAxis::Y => sorted.sort_by(|&a, &b| rects[a].center_y().partial_cmp(&rects[b].center_y()).unwrap()),
    }

    let mid = sorted.len() / 2;
    let (first, second) = sorted.split_at(mid);

    // Guarantee non-empty groups (should hold for len >= 2)
    if first.is_empty() {
        return build_split_tree(rects, second);
    }
    if second.is_empty() {
        return build_split_tree(rects, first);
    }

    SplitTree::Node {
        axis,
        first: Box::new(build_split_tree(rects, first)),
        second: Box::new(build_split_tree(rects, second)),
    }
}

fn restore_split_tree(
    tree: &SplitTree,
    workspace_id: i32,
    saved_clients: &[ipc::HyprClient],
    available_clients: &mut Vec<ipc::HyprClient>,
    baseline_addresses: &HashSet<String>,
    restored_addresses: &mut HashSet<String>,
    launch_timeout: Duration,
) -> Result<String, Box<dyn Error>> {
    match tree {
        SplitTree::Leaf(idx) => {
            let saved = &saved_clients[*idx];
            let current = ensure_restored(
                saved,
                available_clients,
                baseline_addresses,
                restored_addresses,
                launch_timeout,
            )?;
            Ok(current.address)
        }
        SplitTree::Node { axis, first, second } => {
            // Restore the first subtree; then use preselect to create the split and restore the second.
            let pivot_addr = restore_split_tree(
                first,
                workspace_id,
                saved_clients,
                available_clients,
                baseline_addresses,
                restored_addresses,
                launch_timeout,
            )?;

            // Focus pivot and preselect direction for the next window.
            let _ = ipc::dispatch(&format!("focuswindow address:{}", pivot_addr));
            let dir = match axis {
                SplitAxis::X => "r",
                SplitAxis::Y => "d",
            };
            // If user isn't on dwindle layout, this may fail; ignore and continue best-effort.
            let _ = ipc::dispatch(&format!("layoutmsg preselect {}", dir));

            let _second_addr = restore_split_tree(
                second,
                workspace_id,
                saved_clients,
                available_clients,
                baseline_addresses,
                restored_addresses,
                launch_timeout,
            )?;

            Ok(pivot_addr)
        }
    }
}

fn ensure_restored(
    saved_client: &ipc::HyprClient,
    available_clients: &mut Vec<ipc::HyprClient>,
    baseline_addresses: &HashSet<String>,
    restored_addresses: &mut HashSet<String>,
    timeout: Duration,
) -> Result<ipc::HyprClient, Box<dyn Error>> {
    // 1) Try to match an already-running client first.
    if let Some(index) = available_clients
        .iter()
        .position(|c| launched_window_matches(c, saved_client))
    {
        let current_client = available_clients.remove(index);
        println!(
            "   Restoring window: {} ({})",
            current_client.class, current_client.title
        );
        position::restore_window_position(&current_client, saved_client)?;
        restored_addresses.insert(current_client.address.clone());
        return Ok(current_client);
    }

    // 2) Launch missing app (target workspace is best-effort; we still explicitly move it).
    println!("   ⚠️ Window missing: {}", saved_client.class);
    let _ = std::process::Command::new("notify-send")
        .arg("Restoring Session")
        .arg(format!("Launching {}...", saved_client.class))
        .spawn();

    let command = if let Some(path) = &saved_client.exe_path {
        path.clone()
    } else {
        let raw_name = if !saved_client.initial_class.is_empty() {
            &saved_client.initial_class
        } else {
            &saved_client.class
        };
        resolve_command(raw_name)
    };

    println!("      -> Launching: {}", command);
    let exec_arg = format!(
        "[workspace {} silent] {}",
        saved_client.workspace.id, command
    );

    let output = std::process::Command::new("hyprctl")
        .arg("dispatch")
        .arg("exec")
        .arg(&exec_arg)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                return Err(format!(
                    "Failed to launch {}: {}",
                    command,
                    String::from_utf8_lossy(&out.stderr)
                )
                .into());
            }
        }
        Err(e) => return Err(format!("Failed to execute hyprctl: {}", e).into()),
    }

    // 3) Poll until the newly spawned window appears.
    let poll_interval = Duration::from_millis(250);
    let start = Instant::now();

    while start.elapsed() < timeout {
        let new_state = ipc::capture_state()?;
        if let Some(current_client) = new_state.clients.iter().find(|c| {
            !baseline_addresses.contains(&c.address)
                && !restored_addresses.contains(&c.address)
                && launched_window_matches(c, saved_client)
        }) {
            println!("   Positioning launched window: {}", saved_client.class);
            position::restore_window_position(current_client, saved_client)?;
            restored_addresses.insert(current_client.address.clone());
            return Ok(current_client.clone());
        }

        std::thread::sleep(poll_interval);
    }

    Err(format!(
        "Could not find launched window for positioning (timed out): {}",
        saved_client.class
    )
    .into())
}
