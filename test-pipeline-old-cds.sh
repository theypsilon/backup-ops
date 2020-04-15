#!/usr/bin/env bash

set -euo pipefail

OUT="out/old_cds_merged_check_7"

mkdir -p ${OUT}

cargo build --all --release
echo
./target/release/filter-paths \
    --input ${OUT}/test_hash.csv --output ${OUT}/test_filter.csv \
    --error-log ${OUT}/test_errors_filter_2.log \
    --exclude-unique-hashes
echo
echo "DONE!"