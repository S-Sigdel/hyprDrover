# hyprdrover

hyprdrover is a lightweight session manager for the Hyprland compositor. It allows users to snapshot their current window layout (workspaces, positions, and sizes) and restore it later. This is particularly useful for saving specific workflows and quickly switching between them.

## Features

-   **Session Snapshot**: Captures the state of all active windows, including their workspace, position, size, and monitor.
-   **Session Restoration**: Restores windows to their saved positions and workspaces.
-   **Application Launching**: Automatically attempts to launch applications that are missing from the current session during restoration.
-   **Smart Filtering**: Automatically ignores system overlays and background utilities (e.g., Rofi, Waybar, Dunst).
-   **CLI Interface**: Simple command-line interface for saving, loading, and listing sessions.
-   **JSON Storage**: Sessions are saved as human-readable JSON files.

## Installation

### Prerequisites

-   Rust (latest stable toolchain)
-   Hyprland
-   `hyprctl` (usually comes with Hyprland)

### Building from Source

1.  Clone the repository:
    ```bash
    git clone https://github.com/S-Sigdel/hyprDrover.git
    cd hyprDrover
    ```

2.  Build the project:
    ```bash
    cargo build --release
    ```

3.  Install the binary:
    You can use the built-in install command to copy the binary to `~/.local/bin/`:
    ```bash
    ./target/release/hyprdrover --install
    ```
    
    Alternatively, you can manually move it:
    ```bash
    sudo cp target/release/hyprdrover /usr/local/bin/
    ```

## Usage

hyprdrover is controlled entirely via the command line.

### Save a Session

To snapshot the current state of your Hyprland session:

```bash
hyprdrover --save
```

This will create a new JSON file in the `sessions/` directory (relative to where you run the command) with a timestamp.

### List Saved Sessions

To view all available snapshots:

```bash
hyprdrover --list
```

### Restore a Session

To restore the most recent session:

```bash
hyprdrover --load
```

To restore a specific session file:

```bash
hyprdrover --load sessions/session_YYYY-MM-DD_HH-MM-SS.json
```

### Install

To install the binary to your local bin directory (`~/.local/bin`):

```bash
hyprdrover --install
```

## Configuration

Currently, configuration is handled via defaults. The application automatically ignores the following window classes to prevent issues during restoration:

-   `rofi`
-   `waybar`
-   `dunst`
-   `hyprland-share-picker`
-   `polkit-gnome-authentication-agent-1`

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## Releases

For pre-built binaries and version history, please check the [Releases](https://github.com/S-Sigdel/hyprDrover/releases) tab.
