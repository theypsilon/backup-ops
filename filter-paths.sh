#!/usr/bin/env bash

set -xeuo pipefail

cargo run --release -p filter-paths -- \
    --input old_cds.csv --output filtered_cds.csv \
    --error-log errors_filter.log \
    --size-min 1250000 \
    --size-max 1500000

wc -l old_cds.csv
wc -l filtered_cds.csv
wc -l errors_filter.log || true