#!/usr/bin/env bash
# Automated bootstrap for the Freegin AI project.
# Installs required toolchain components (Rust, make, sqlx-cli) and installs the binary.

set -euo pipefail

INSTALL_PREFIX=""

log() {
    printf '\033[1;34m[bootstrap]\033[0m %s\n' "$1"
}

warn() {
    printf '\033[1;33m[bootstrap]\033[0m %s\n' "$1"
}

error() {
    printf '\033[1;31m[bootstrap]\033[0m %s\n' "$1" >&2
}

usage() {
    cat <<USAGE
Usage: ${0##*/} [--prefix DIR] [--system]

Options:
  --prefix DIR   Install under DIR (default: ~/.local)
  --system       Shorthand for --prefix /usr/local (requires sudo)
USAGE
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --prefix)
                shift
                INSTALL_PREFIX="${1:?--prefix requires a directory}";
                ;;
            --system)
                INSTALL_PREFIX="/usr/local"
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
        shift || true
    done

    if [[ -z "${INSTALL_PREFIX}" ]]; then
        INSTALL_PREFIX="${HOME}/.local"
    fi

}

ensure_rust() {
    if command_exists cargo; then
        log "Rust toolchain already present."
        return
    fi

    if ! command_exists rustup; then
        warn "Rust not found. Installing rustup (will modify ~/.cargo)."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    else
        warn "rustup found but cargo missing; running rustup self update."
        rustup self update
    fi

    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"

    if ! command_exists cargo; then
        error "cargo is still unavailable after installing rustup. Please inspect your PATH."
        exit 1
    fi
}

ensure_make() {
    if command_exists make; then
        log "GNU make already installed."
        return
    fi

    warn "GNU make not detected. Attempting to install."

    case "$(uname -s)" in
        Linux)
            if command_exists apt-get; then
                sudo apt-get update && sudo apt-get install -y make
            elif command_exists dnf; then
                sudo dnf install -y make
            elif command_exists pacman; then
                sudo pacman -Sy --needed make
            else
                warn "Unable to automatically install make. Please install GNU make manually."
            fi
            ;;
        Darwin)
            if command_exists brew; then
                brew install make
            else
                warn "Homebrew not found. Install Xcode Command Line Tools or Homebrew to get make."
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*)
            warn "On Windows, install make via MSYS2 or WSL."
            ;;
        *)
            warn "Unsupported platform for automatic make installation."
            ;;
    esac

    if ! command_exists make; then
        error "make is required but could not be installed automatically."
        exit 1
    fi
}

ensure_sqlx_cli() {
    if command_exists sqlx; then
        log "sqlx-cli already installed."
        return
    fi

    log "Installing sqlx-cli via cargo install..."
    cargo install sqlx-cli --locked
}

prepare_config() {
    local config_root="${XDG_CONFIG_HOME:-$HOME/.config}"
    local app_config_dir="${config_root}/freegin-ai"
    local target_config="${app_config_dir}/config.toml"

    if [[ ! -f "${target_config}" ]]; then
        log "Seeding configuration at ${target_config}";
        mkdir -p "${app_config_dir}"
        cp .config/template.toml "${target_config}"
        warn "Update API credentials in ${target_config} before running the service."
    else
        log "Configuration file already present at ${target_config}."
    fi
}

install_project() {
    log "Building and installing the project binary to ${INSTALL_PREFIX}."
    PREFIX="${INSTALL_PREFIX}" make install
}

main() {
    parse_args "$@"
    log "Starting bootstrap sequence."
    ensure_rust
    ensure_make
    ensure_sqlx_cli
    prepare_config
    install_project
    local bin_dir="${INSTALL_PREFIX}/bin"
    if ! printf '%s' ":$PATH:" | grep -q ":${bin_dir}:"; then
        warn "${bin_dir} is not on your PATH. Add 'export PATH=\"${bin_dir}:\$PATH\"' to your shell profile."
    fi
    local binary_path="${bin_dir}/freegin-ai"
    if [[ -x "${binary_path}" ]]; then
        log "Installed binary self-check (${binary_path} --help):"
        "${binary_path}" --help || warn "freegin-ai --help exited with status $?"
    else
        warn "Installed binary not found at ${binary_path}."
    fi
    log "Bootstrap complete. Installed freegin-ai to ${bin_dir}."
}

main "$@"
