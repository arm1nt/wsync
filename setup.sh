#!/bin/bash

set -e

validate_file() {
  local file=$1

  if [ ! -f "$file" ]; then
    echo "[ERROR] File '$file' does not exist";
    exit 1;
  fi

  if [ ! -r "$file" ]; then
    echo "[ERROR] File '$file' is not readable";
    exit 1;
  fi
}

validate_directory() {
  local directory=$1

  if [ ! -d "$directory" ]; then
    echo "[ERROR] Directory '$directory' does not exist";
    exit 1;
  fi
}

check_environment() {

  if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERROR] cargo not found in PATH.";
    exit 1;
  fi

  if ! command -v rsync >/dev/null 2>&1; then
    echo "[ERROR] rsync not found in PATH.";
    exit 1;
  fi
}

prepare_environment() {

  if ! mkdir -p "$HOME/.wsync"; then
    echo "[ERROR] Failed to create the '$HOME/.wsync' directory!"
    exit 1;
  fi

  if ! mkdir -p "$HOME/.wsync/log"; then
    echo "[ERROR] Failed to create the '$HOME/.wsync/log' directory!"
    exit 1;
  fi
}

# Check that all required dependencies are installed on the system
check_environment

prepare_environment

MODE="release"
TARGET_DIR="./build"
LOG_DIRECTORY="$HOME/.wsync/log"
WS_CONFIG_DIR="$HOME/.wsync"
DAEMON_CMD_SOCKET_DIR="/tmp"
CLEANUP=false
QUIET=false

VALID_ARGS=$(getopt -o m:cq --long mode:,cleanup,quiet,ws_config_dir:,log_directory:,daemon_cmd_socket_dir: -- "$@")
if [[ $? -ne 0 ]]; then
  echo "[ERROR] Error setting VALID_ARGS";
  exit 1;
fi

eval set -- "$VALID_ARGS"
while true; do

  case "$1" in
    -q | --quiet)
      QUIET=true
      shift
      ;;
    -c | --cleanup)
      CLEANUP=true
      shift
      ;;
    -m | --mode)

      if [[ "$2" != "debug" && "$2" != "release" ]]; then
        echo "[ERROR] The value of 'mode' must either be 'debug' or 'release'";
        exit 1;
      fi
      MODE="$2"
      shift 2
      ;;
    --ws_config_dir)
      validate_directory "$2"
      WS_CONFIG_DIR="$2"
      shift 2
      ;;
    --log_directory)
      validate_directory "$2"
      LOG_DIRECTORY="$2"
      shift 2
      ;;
    --daemon_cmd_socket_dir)
      validate_directory "$2"
      DAEMON_CMD_SOCKET_DIR="$2"
      shift 2
      ;;
    --) shift;
      break
      ;;
  esac
done

# Build all crates

CRATES=(
  "./lib/daemon-client"
  "./lib/daemon-interface"
  "./lib/wsync-config"
  "./src/client"
  "./src/monitor"
  "./src/daemon"
)

declare -a build_flags
[[ "$MODE" == "release" ]] && build_flags+=(--release)
[[ "$QUIET" ]] && build_flags+=(--quiet)
build_flags+=(--target-dir "$TARGET_DIR")

for crate_dir in "${CRATES[@]}"; do
  manifest_path="$crate_dir/Cargo.toml"

  validate_file "$manifest_path"

  echo "============================================================"
  echo "Building: $crate_dir"
  echo "============================================================"

  if $CLEANUP; then
    echo ">> cargo clean --manifest_path \"$manifest_path\" --target-dir \"$TARGET_DIR\""
    cargo clean --manifest-path "$manifest_path" --target-dir "$TARGET_DIR"
  fi

  echo ">> cargo build --manifest-path \"$manifest_path\" ${build_flags[*]}"
  cargo build --manifest-path "$manifest_path" "${build_flags[@]}"
done

# Check that we can find the monitor executable
PROJECT_PATH=$(pwd)
MONITOR_EXECUTABLE="$PROJECT_PATH/$TARGET_DIR/$MODE/monitor"

if [[ ! -f "$MONITOR_EXECUTABLE" ]]; then
  echo "[ERROR] Cannot find the monitor executable file at '$MONITOR_EXECUTABLE'";
  exit 1;
fi

WSYNC_CONFIG_FILE="$HOME/.wsync/wsync.config"
touch "$WSYNC_CONFIG_FILE"
# Truncate file to prevent duplicate entries
: > "$WSYNC_CONFIG_FILE"

{
  echo "WorkspaceConfigFilePath=$WS_CONFIG_DIR/wsync-ws-config.json";
  echo "MonitorExecutablePath=$MONITOR_EXECUTABLE";
  echo "DaemonCommandSocketPath=$DAEMON_CMD_SOCKET_DIR/wsync-daemon-cmd.socket";
  echo "LogDirectory=$LOG_DIRECTORY";
} >>  "$WSYNC_CONFIG_FILE"


EXECUTABLES_DIR_PATH="$PROJECT_PATH/$TARGET_DIR/$MODE"
validate_directory "$EXECUTABLES_DIR_PATH"

echo "================================================================"
echo "For ease of use, add the client to the PATH by appending the following string to your shell configuration file:"
echo ">> export PATH=\"$EXECUTABLES_DIR_PATH:\$PATH\""
