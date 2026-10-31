#!/usr/bin/env bash
set -e
DR="$(cd "$(dirname "$0")" && pwd)"
cd "$DR"
export TERM="${TERM:-xterm-256color}"

TARGET_DIR="/home/foxter/.cache/speedtest-tui/target"

if [ ! -x "./speedtui" ] || [ "src/main.rs" -nt "./speedtui" ] || [ "src/dsp.rs" -nt "./speedtui" ] || [ "src/engn.rs" -nt "./speedtui" ] || [ "src/grph.rs" -nt "./speedtui" ]; then
    cargo build --release -q
    cp --remove-destination "${TARGET_DIR}/release/speedtui" ./speedtui
fi

exec "./speedtui" "$@"
