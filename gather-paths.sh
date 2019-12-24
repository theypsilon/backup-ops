#!/usr/bin/env bash

set -euo pipefail

cargo run --release -p gather-paths -- \
    /mnt/c/ c.csv \
    --error-log errors.log \
    --recursive \
    --lengths
