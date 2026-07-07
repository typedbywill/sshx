#!/bin/bash
# ==============================================================================
# SSHX Development & Automation Helper Script
# ==============================================================================
# This script provides easy shortcuts for building, installing, running,
# testing, and formatting the SSHX project.
# ==============================================================================

set -e

# --- 1. Cargo Detection ---
if command -v cargo >/dev/null 2>&1; then
    CARGO="cargo"
elif [ -f "$HOME/.cargo/bin/cargo" ]; then
    CARGO="$HOME/.cargo/bin/cargo"
else
    echo -e "\x1b[31mError: Cargo (Rust package manager) not found in PATH or ~/.cargo/bin/cargo\x1b[0m"
    echo "Please install Rust and Cargo from: https://rustup.rs"
    exit 1
fi

# --- 2. Helper Functions ---
show_help() {
    echo "SSHX Developer CLI Helper"
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build         Build SSHX in Release mode and copy to repository root"
    echo "  install       Build in Release and install locally to ~/.local/bin/sshx"
    echo "  update        Pull latest commits from git, rebuild and reinstall"
    echo "  run [args]    Run development build with the specified arguments"
    echo "  test          Run unit tests"
    echo "  fmt           Format code and run clippy check"
    echo "  help          Show this help menu"
    echo ""
}

build_release() {
    echo -e "\x1b[34m[Building] Building SSHX in Release mode...\x1b[0m"
    "$CARGO" build --release
    cp target/release/sshx ./sshx
    echo -e "\x1b[32m[Success] Binary built and copied to ./sshx\x1b[0m"
}

install_local() {
    build_release
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    
    echo -e "\x1b[34m[Installing] Installing to ${INSTALL_DIR}/sshx...\x1b[0m"
    cp target/release/sshx "${INSTALL_DIR}/sshx"
    echo -e "\x1b[32m[Success] SSHX installed to ${INSTALL_DIR}/sshx\x1b[0m"
    
    # Check if PATH contains install directory
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo -e "\x1b[33mWarning: ${INSTALL_DIR} is not in your PATH.\x1b[0m"
        echo "Add this to your shell configuration (e.g. ~/.bashrc or ~/.zshrc):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

update_project() {
    echo -e "\x1b[34m[Updating] Pulling latest code from Git...\x1b[0m"
    git pull
    install_local
}

run_dev() {
    "$CARGO" build
    echo -e "\x1b[34m[Running] target/debug/sshx $*\x1b[0m"
    target/debug/sshx "$@"
}

run_tests() {
    echo -e "\x1b[34m[Testing] Running Cargo tests...\x1b[0m"
    "$CARGO" test
}

run_fmt() {
    echo -e "\x1b[34m[Formatting] Formatting codebase with cargo fmt...\x1b[0m"
    if "$CARGO" fmt -- --check >/dev/null 2>&1; then
        echo "Code format is already clean."
    else
        "$CARGO" fmt
        echo "Code reformatted successfully."
    fi

    echo -e "\x1b[34m[Linting] Running cargo clippy...\x1b[0m"
    "$CARGO" clippy --all-targets
}

# --- 3. Command Routing ---
CMD="${1:-help}"
case "$CMD" in
    build)
        build_release
        ;;
    install)
        install_local
        ;;
    update)
        update_project
        ;;
    run)
        shift
        run_dev "$@"
        ;;
    test)
        run_tests
        ;;
    fmt)
        run_fmt
        ;;
    help|-h|--help)
        show_help
        ;;
    *)
        echo -e "\x1b[31mUnknown command: $CMD\x1b[0m"
        show_help
        exit 1
        ;;
esac
