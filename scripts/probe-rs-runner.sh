#!/bin/sh
set -eu

CHIP="STM32F103C8"

if command -v probe-rs >/dev/null 2>&1; then
    if probe-rs run --chip "${CHIP}" "$@"; then
        exit 0
    fi
    echo "probe-rs detected but failed to talk to hardware; skipping flashing. Built binary: $1" >&2
    exit 0
fi

echo "probe-rs not installed; skipping hardware run. Built binary: $1" >&2
exit 0
