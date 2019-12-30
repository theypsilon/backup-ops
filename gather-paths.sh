#!/usr/bin/env bash

set -xeuo pipefail

cargo run --release -p gather-paths -- \
    -i /mnt/c/Users/Jose/Documents/Old_CDs\ 1/ /mnt/c/Users/Jose/Documents/Old_CDs\ 2/ \
    -o old_cds.csv \
    -e errors_gather.log \
    -rl

wc -l old_cds.csv
wc -l errors_gather.log || true