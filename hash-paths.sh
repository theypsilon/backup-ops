#!/usr/bin/env bash

set -xeuo pipefail

cargo run --release -p hash-paths -- \
    --input filtered_cds.csv --output hashed_cds.csv \
    --error-log errors_hash.log

wc -l filtered_cds.csv
wc -l hashed_cds.csv
wc -l errors_hash.log || true