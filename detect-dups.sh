#!/usr/bin/env bash

set -euo pipefail

cargo run --release -p detect-dups -- \
    c.csv dups.log \
    --error-log errors.log \
    --size-min 10 \
    --hashing-check \
    --exclude-path-contents "/home/jose/workspace/" \
    --exclude-path-starts "/mnt/c/Windows/" "/mnt/c/octave" "/mnt/c/Users/Jose/Documents/Consoles/" \
    "/mnt/c/Users/Jose/Downloads/" "/mnt/c/Users/Jose/.rustup/" "/mnt/c/Users/Jose/.cargo/" \
    "/mnt/c/Users/Jose/AppData/" "/mnt/c/Users/Jose/Documents/Workspace/" \
    "/mnt/c/Program Files (x86)/Microsoft" "/mnt/c/Program Files (x86)/Windows"
