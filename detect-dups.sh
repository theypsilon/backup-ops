#!/usr/bin/env bash

set -euo pipefail

cargo run --release -p detect-dups -- \
    c.csv dups.log \
    --error-log errors.log \
    --size-min 1000 \
    --exclude-path-contents "Jose/Documents/Workspace/" "/home/jose/workspace/" \
    --exclude-path-starts "/mnt/c/Windows/" "/mnt/c/octave"
