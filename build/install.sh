#!/bin/sh
set -eu

REPO_TARBALL_URL="https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz"
RUSTUP_URL="https://sh.rustup.rs"

MODE=""
NO_AUDIO=0
OS=""
PKG_MGR=""
WORK_DIR=""

MISSING_CURL=0
MISSING_TAR=0
MISSING_RUST=0
MISSING_CC=0
MISSING_PKG_CONFIG=0
MISSING_ALSA=0
MISSING_COUNT=0
SOUND_ONLY_MISSING=0

cleanup() {
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        rm -rf "$WORK_DIR"
    fi
}

trap cleanup EXIT INT TERM

usage() {
    cat <<'EOF'
Usage: install.sh [MODE] [--no-audio] [--help]

Modes:
  --check      Check dependencies and stop at the first problem (non-zero exit).
  --list       List every missing dependency and the command to install it.
  --install    Install missing dependencies (after asking), re-check, then build
               and install oxysol from the release branch into ~/.cargo/bin.

Options:
  --no-audio   Skip ALSA/pkg-config checks and build with --no-default-features.
  -h, --help   Show this help message.

Without a mode flag, an interactive terminal asks which mode to run;
a non-interactive run behaves like --list.
EOF
}

parse_args() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --check)
                set_mode "check"
                ;;
            --list)
                set_mode "list"
                ;;
            --install)
                set_mode "install"
                ;;
            --no-audio)
                NO_AUDIO=1
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                printf 'Unknown option: %s\n' "$1" >&2
                usage >&2
                exit 2
                ;;
        esac
        shift
    done
}

set_mode() {
    if [ -n "$MODE" ] && [ "$MODE" != "$1" ]; then
        printf 'Conflicting mode flags: --%s and --%s\n' "$MODE" "$1" >&2
        exit 2
    fi
    MODE="$1"
}

detect_platform() {
    os_name="$(uname -s)"
    case "$os_name" in
        Linux)
            OS="linux"
            if command -v apt-get >/dev/null 2>&1; then
                PKG_MGR="apt"
            elif command -v dnf >/dev/null 2>&1; then
                PKG_MGR="dnf"
            elif command -v pacman >/dev/null 2>&1; then
                PKG_MGR="pacman"
            elif command -v zypper >/dev/null 2>&1; then
                PKG_MGR="zypper"
            else
                PKG_MGR="unknown"
            fi
            ;;
        Darwin)
            OS="macos"
            if command -v brew >/dev/null 2>&1; then
                PKG_MGR="brew"
            else
                PKG_MGR="none"
            fi
            ;;
        *)
            printf 'Unsupported operating system: %s\n' "$os_name" >&2
            exit 1
            ;;
    esac
}

ensure_cargo_in_path() {
    cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"
    if [ -d "$cargo_bin" ]; then
        case ":$PATH:" in
            *":$cargo_bin:"*) ;;
            *) PATH="$cargo_bin:$PATH"; export PATH ;;
        esac
    fi
}

has_alsa_headers() {
    if command -v pkg-config >/dev/null 2>&1; then
        pkg-config --exists alsa >/dev/null 2>&1
        return $?
    fi
    if [ -f /usr/include/alsa/asoundlib.h ] || [ -f /usr/local/include/alsa/asoundlib.h ]; then
        return 0
    fi
    for header in /usr/include/*/alsa/asoundlib.h; do
        if [ -f "$header" ]; then
            return 0
        fi
    done
    return 1
}

check_dependencies() {
    ensure_cargo_in_path

    MISSING_CURL=0
    MISSING_TAR=0
    MISSING_RUST=0
    MISSING_CC=0
    MISSING_PKG_CONFIG=0
    MISSING_ALSA=0
    MISSING_COUNT=0
    SOUND_ONLY_MISSING=0

    if ! command -v curl >/dev/null 2>&1; then
        MISSING_CURL=1
        MISSING_COUNT=$((MISSING_COUNT + 1))
    fi

    if ! command -v tar >/dev/null 2>&1; then
        MISSING_TAR=1
        MISSING_COUNT=$((MISSING_COUNT + 1))
    fi

    if ! command -v rustup >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        MISSING_RUST=1
        MISSING_COUNT=$((MISSING_COUNT + 1))
    fi

    if [ "$OS" = "linux" ]; then
        if ! command -v cc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
            MISSING_CC=1
            MISSING_COUNT=$((MISSING_COUNT + 1))
        fi
        if [ "$NO_AUDIO" -eq 0 ]; then
            if ! command -v pkg-config >/dev/null 2>&1; then
                MISSING_PKG_CONFIG=1
                MISSING_COUNT=$((MISSING_COUNT + 1))
            fi
            if ! has_alsa_headers; then
                MISSING_ALSA=1
                MISSING_COUNT=$((MISSING_COUNT + 1))
            fi
        fi
    elif [ "$OS" = "macos" ]; then
        if ! xcode-select -p >/dev/null 2>&1 || { ! command -v cc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; }; then
            MISSING_CC=1
            MISSING_COUNT=$((MISSING_COUNT + 1))
        fi
    fi

    if [ "$OS" = "linux" ] && [ "$NO_AUDIO" -eq 0 ] && [ "$MISSING_CURL" -eq 0 ] && [ "$MISSING_TAR" -eq 0 ] && [ "$MISSING_RUST" -eq 0 ] && [ "$MISSING_CC" -eq 0 ]; then
        if [ "$MISSING_PKG_CONFIG" -eq 1 ] || [ "$MISSING_ALSA" -eq 1 ]; then
            SOUND_ONLY_MISSING=1
        fi
    fi
}

linux_system_packages() {
    pkgs=""
    case "$PKG_MGR" in
        apt)
            [ "$MISSING_CURL" -eq 1 ] && pkgs="$pkgs curl"
            [ "$MISSING_TAR" -eq 1 ] && pkgs="$pkgs tar"
            [ "$MISSING_CC" -eq 1 ] && pkgs="$pkgs build-essential"
            [ "$MISSING_PKG_CONFIG" -eq 1 ] && pkgs="$pkgs pkg-config"
            [ "$MISSING_ALSA" -eq 1 ] && pkgs="$pkgs libasound2-dev"
            ;;
        dnf)
            [ "$MISSING_CURL" -eq 1 ] && pkgs="$pkgs curl"
            [ "$MISSING_TAR" -eq 1 ] && pkgs="$pkgs tar"
            [ "$MISSING_CC" -eq 1 ] && pkgs="$pkgs gcc"
            [ "$MISSING_PKG_CONFIG" -eq 1 ] && pkgs="$pkgs pkgconf-pkg-config"
            [ "$MISSING_ALSA" -eq 1 ] && pkgs="$pkgs alsa-lib-devel"
            ;;
        pacman)
            [ "$MISSING_CURL" -eq 1 ] && pkgs="$pkgs curl"
            [ "$MISSING_TAR" -eq 1 ] && pkgs="$pkgs tar"
            [ "$MISSING_CC" -eq 1 ] && pkgs="$pkgs base-devel"
            [ "$MISSING_PKG_CONFIG" -eq 1 ] && pkgs="$pkgs pkgconf"
            [ "$MISSING_ALSA" -eq 1 ] && pkgs="$pkgs alsa-lib"
            ;;
        zypper)
            [ "$MISSING_CURL" -eq 1 ] && pkgs="$pkgs curl"
            [ "$MISSING_TAR" -eq 1 ] && pkgs="$pkgs tar"
            [ "$MISSING_CC" -eq 1 ] && pkgs="$pkgs gcc"
            [ "$MISSING_PKG_CONFIG" -eq 1 ] && pkgs="$pkgs pkg-config"
            [ "$MISSING_ALSA" -eq 1 ] && pkgs="$pkgs alsa-devel"
            ;;
    esac
    printf '%s' "${pkgs# }"
}

linux_install_command() {
    pkgs="$(linux_system_packages)"
    if [ -z "$pkgs" ]; then
        return
    fi
    case "$PKG_MGR" in
        apt)
            printf 'sudo apt-get update && sudo apt-get install -y %s' "$pkgs"
            ;;
        dnf)
            printf 'sudo dnf install -y %s' "$pkgs"
            ;;
        pacman)
            printf 'sudo pacman -S --needed %s' "$pkgs"
            ;;
        zypper)
            printf 'sudo zypper install -y %s' "$pkgs"
            ;;
    esac
}

run_check_mode() {
    if [ "$MISSING_CURL" -eq 1 ]; then
        printf 'Missing dependency: curl\n' >&2
        exit 1
    fi
    if [ "$MISSING_TAR" -eq 1 ]; then
        printf 'Missing dependency: tar\n' >&2
        exit 1
    fi
    if [ "$MISSING_CC" -eq 1 ]; then
        if [ "$OS" = "macos" ]; then
            printf 'Missing dependency: Xcode Command Line Tools (xcode-select --install)\n' >&2
        else
            printf 'Missing dependency: C compiler (cc or gcc)\n' >&2
        fi
        exit 1
    fi
    if [ "$MISSING_PKG_CONFIG" -eq 1 ]; then
        printf 'Missing dependency: pkg-config\n' >&2
        exit 1
    fi
    if [ "$MISSING_ALSA" -eq 1 ]; then
        printf 'Missing dependency: ALSA development headers (alsa.pc)\n' >&2
        exit 1
    fi
    if [ "$MISSING_RUST" -eq 1 ]; then
        printf 'Missing dependency: Rust toolchain (rustup and cargo)\n' >&2
        exit 1
    fi
    printf 'All build dependencies are present.\n'
}

run_list_mode() {
    if [ "$MISSING_COUNT" -eq 0 ]; then
        printf 'All build dependencies are present.\n'
        return 0
    fi

    printf 'Missing build dependencies:\n'
    [ "$MISSING_CURL" -eq 1 ] && printf '  - curl (download tool)\n'
    [ "$MISSING_TAR" -eq 1 ] && printf '  - tar (archive extractor)\n'
    if [ "$MISSING_CC" -eq 1 ]; then
        if [ "$OS" = "macos" ]; then
            printf '  - Xcode Command Line Tools (C toolchain)\n'
        else
            printf '  - C compiler (cc / gcc)\n'
        fi
    fi
    [ "$MISSING_PKG_CONFIG" -eq 1 ] && printf '  - pkg-config\n'
    [ "$MISSING_ALSA" -eq 1 ] && printf '  - ALSA development headers (alsa.pc)\n'
    [ "$MISSING_RUST" -eq 1 ] && printf '  - Rust (rustup / cargo)\n'

    printf '\nInstall commands:\n'
    if [ "$OS" = "linux" ]; then
        if [ "$PKG_MGR" = "unknown" ]; then
            printf 'Unknown Linux package manager. Install these packages with your distribution package manager:\n'
            [ "$MISSING_CURL" -eq 1 ] && printf '  - curl\n'
            [ "$MISSING_TAR" -eq 1 ] && printf '  - tar\n'
            [ "$MISSING_CC" -eq 1 ] && printf '  - gcc (or build-essential / base-devel)\n'
            [ "$MISSING_PKG_CONFIG" -eq 1 ] && printf '  - pkg-config (or pkgconf)\n'
            [ "$MISSING_ALSA" -eq 1 ] && printf '  - libasound2-dev (Debian/Ubuntu), alsa-lib-devel (Fedora), alsa-lib (Arch), or alsa-devel (openSUSE)\n'
        else
            cmd="$(linux_install_command)"
            if [ -n "$cmd" ]; then
                printf '  %s\n' "$cmd"
            fi
        fi
    elif [ "$OS" = "macos" ]; then
        if [ "$MISSING_CC" -eq 1 ]; then
            printf '  xcode-select --install\n'
        fi
        if [ "$MISSING_CURL" -eq 1 ] || [ "$MISSING_TAR" -eq 1 ]; then
            if [ "$PKG_MGR" = "brew" ]; then
                brew_pkgs=""
                [ "$MISSING_CURL" -eq 1 ] && brew_pkgs="$brew_pkgs curl"
                [ "$MISSING_TAR" -eq 1 ] && brew_pkgs="$brew_pkgs gnu-tar"
                printf '  brew install%s\n' "$brew_pkgs"
            else
                printf '  Install curl and tar (or install Homebrew from https://brew.sh)\n'
            fi
        fi
    fi

    if [ "$MISSING_RUST" -eq 1 ]; then
        printf '  curl --proto '\''=https'\'' --tlsv1.2 -sSf %s | sh\n' "$RUSTUP_URL"
    fi

    if [ "$SOUND_ONLY_MISSING" -eq 1 ]; then
        printf '\nAlternative without sound: re-run with --no-audio (builds with --no-default-features).\n'
    fi
}

ask_yes_no() {
    prompt="$1"
    printf '%s [y/N]: ' "$prompt"
    read -r reply || return 1
    case "$reply" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

run_privileged() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    else
        sudo "$@"
    fi
}

install_linux_packages() {
    pkgs="$(linux_system_packages)"
    if [ -z "$pkgs" ]; then
        return 0
    fi
    # Word splitting of $pkgs is intentional: package names are space-separated tokens.
    # shellcheck disable=SC2086
    case "$PKG_MGR" in
        apt)
            run_privileged apt-get update
            run_privileged apt-get install -y $pkgs
            ;;
        dnf)
            run_privileged dnf install -y $pkgs
            ;;
        pacman)
            run_privileged pacman -S --needed $pkgs
            ;;
        zypper)
            run_privileged zypper install -y $pkgs
            ;;
    esac
}

install_macos_packages() {
    if [ "$MISSING_CC" -eq 1 ]; then
        printf 'Starting Xcode Command Line Tools installer...\n'
        xcode-select --install || true
        printf 'Finish the Xcode Command Line Tools installation, then re-run this script.\n'
        exit 1
    fi
    if [ "$MISSING_CURL" -eq 1 ] || [ "$MISSING_TAR" -eq 1 ]; then
        if [ "$PKG_MGR" = "brew" ]; then
            [ "$MISSING_CURL" -eq 1 ] && brew install curl
            [ "$MISSING_TAR" -eq 1 ] && brew install gnu-tar
        else
            printf 'Missing curl or tar and Homebrew is not installed.\n' >&2
            exit 1
        fi
    fi
}

install_rust() {
    if [ "$MISSING_RUST" -eq 0 ]; then
        return 0
    fi
    if ! command -v curl >/dev/null 2>&1; then
        printf 'curl is required to install rustup.\n' >&2
        exit 1
    fi
    curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_URL" | sh -s -- -y
    ensure_cargo_in_path
}

build_and_install() {
    if [ "$NO_AUDIO" -eq 1 ]; then
        printf 'Building and installing oxysol without audio (--no-default-features)...\n'
    else
        printf 'Building and installing oxysol...\n'
    fi

    WORK_DIR="$(mktemp -d)"
    tarball="$WORK_DIR/oxysol-source.tar.gz"
    curl -fsSL -o "$tarball" "$REPO_TARBALL_URL"
    tar -xzf "$tarball" -C "$WORK_DIR"
    src_dir="$WORK_DIR/oXyFlow.Solitaire-release"
    if [ ! -d "$src_dir" ]; then
        printf 'Expected source directory not found: %s\n' "$src_dir" >&2
        exit 1
    fi

    if [ "$NO_AUDIO" -eq 1 ]; then
        cargo install --path "$src_dir" --locked --no-default-features
    else
        cargo install --path "$src_dir" --locked
    fi

    printf 'oxysol installed to %s/oxysol\n' "${CARGO_HOME:-$HOME/.cargo}/bin"
}

run_install_mode() {
    if [ "$MISSING_COUNT" -gt 0 ]; then
        if [ "$OS" = "linux" ] && [ "$PKG_MGR" = "unknown" ] && { [ "$MISSING_CURL" -eq 1 ] || [ "$MISSING_TAR" -eq 1 ] || [ "$MISSING_CC" -eq 1 ] || [ "$MISSING_PKG_CONFIG" -eq 1 ] || [ "$MISSING_ALSA" -eq 1 ]; }; then
            run_list_mode
            exit 1
        fi

        if [ "$SOUND_ONLY_MISSING" -eq 1 ] && [ -t 0 ]; then
            printf 'Only sound dependencies are missing (pkg-config / ALSA headers).\n'
            printf '  1) Install sound dependencies and build with audio\n'
            printf '  2) Skip sound dependencies and build without audio (--no-audio)\n'
            printf '  3) Cancel\n'
            printf 'Choose [1-3]: '
            read -r audio_choice || exit 1
            case "$audio_choice" in
                1) ;;
                2)
                    NO_AUDIO=1
                    check_dependencies
                    ;;
                *)
                    printf 'Cancelled.\n'
                    exit 0
                    ;;
            esac
        fi

        if [ "$MISSING_COUNT" -gt 0 ]; then
            run_list_mode
            if [ ! -t 0 ]; then
                printf 'Non-interactive session: refusing to install without confirmation.\n' >&2
                exit 1
            fi
            if ! ask_yes_no "Install the missing dependencies listed above?"; then
                printf 'Cancelled.\n'
                exit 1
            fi

            if [ "$OS" = "linux" ]; then
                install_linux_packages
            elif [ "$OS" = "macos" ]; then
                install_macos_packages
            fi
            install_rust

            check_dependencies
            if [ "$MISSING_COUNT" -gt 0 ]; then
                printf 'Dependencies are still missing after installation:\n' >&2
                run_list_mode >&2
                exit 1
            fi
        fi
    fi

    build_and_install
}

choose_interactive_mode() {
    printf 'oXySol dependency check and installer\n'
    printf '  1) Check dependencies (--check)\n'
    printf '  2) List missing dependencies and install commands (--list)\n'
    printf '  3) Install missing dependencies, then build and install (--install)\n'
    printf 'Choose [1-3] (default 2): '
    read -r choice || choice="2"
    case "$choice" in
        1) MODE="check" ;;
        ""|2) MODE="list" ;;
        3) MODE="install" ;;
        *)
            printf 'Invalid choice: %s\n' "$choice" >&2
            exit 2
            ;;
    esac
}

main() {
    parse_args "$@"
    detect_platform

    if [ -z "$MODE" ]; then
        if [ -t 0 ]; then
            choose_interactive_mode
        else
            MODE="list"
        fi
    fi

    check_dependencies

    case "$MODE" in
        check) run_check_mode ;;
        list) run_list_mode ;;
        install) run_install_mode ;;
    esac
}

main "$@"
