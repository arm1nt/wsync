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

MODE="release"
BUILD_TARGET_DIR="./build"
CLEAN=false
QUIET=false

VALID_BUILD_ARGS=$(getopt -o m:cq --long mode:,cleanup,quiet -- "$@")

eval set -- "$VALID_BUILD_ARGS"
while true; do
    case "$1" in
        -q | --quiet)
            QUIET=true
            shift
            ;;
        -c | --cleanup)
            CLEAN=true
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
        --) shift;
            break
            ;;
    esac
done

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
build_flags+=(--target-dir "$BUILD_TARGET_DIR")

if $CLEAN; then
    echo ">> cargo clean --target-dir \"$BUILD_TARGET_DIR\""
    cargo clean --target-dir "$BUILD_TARGET_DIR"
fi

for crate_directory in "${CRATES[@]}"; do
    manifest_path="$crate_directory/Cargo.toml"
    validate_file "$manifest_path"

    echo "============================================================"
    echo "Building: $crate_directory"
    echo "============================================================"


    echo ">> cargo build --manifest-path \"$manifest_path\" ${build_flags[*]}"
    cargo build --manifest-path "$manifest_path" "${build_flags[@]}"
done
