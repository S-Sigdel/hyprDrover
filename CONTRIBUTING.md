# Contributing to hyprDrover

Thank you for your interest in contributing to hyprDrover! We welcome contributions from the community to help improve this project.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/YOUR_USERNAME/hyprDrover.git
    cd hyprDrover
    ```
3.  **Create a new branch** for your feature or bug fix:
    ```bash
    git checkout -b feature/my-new-feature
    ```

## Development Workflow

-   **Rust Toolchain**: Ensure you are using the latest stable version of Rust.
-   **Formatting**: We use `rustfmt` to maintain a consistent code style. Please run `cargo fmt` before committing your changes.
-   **Linting**: We use `clippy` to catch common mistakes. Please run `cargo clippy` and address any warnings.
-   **Testing**: Run the test suite to ensure no regressions:
    ```bash
    cargo test
    ```

## Submitting Changes

1.  **Commit your changes** with clear and descriptive commit messages.
2.  **Push to your fork**:
    ```bash
    git push origin feature/my-new-feature
    ```
3.  **Open a Pull Request** against the `main` branch of the original repository.
4.  Provide a clear description of the changes and the problem they solve.

## Reporting Bugs

If you encounter any issues, please open an issue on the GitHub repository. Include as much detail as possible, such as:

-   Steps to reproduce the issue.
-   Expected behavior vs. actual behavior.
-   Your Hyprland version (`hyprctl version`).
-   Logs or error messages.

## Code of Conduct

Please be respectful and professional in all interactions. We strive to maintain a welcoming and inclusive community.
