#!/usr/bin/env bash

set -xeuo pipefail

cargo run --release -p detect-dups -- \
    -i hashed_cds.csv -o dups.json \
    --error-log errors_dups.log

wc -l hashed_cds.csv
wc -l dups.json
wc -l errors_dups.log || true