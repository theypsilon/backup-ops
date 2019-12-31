#!/usr/bin/env bash

set -euo pipefail

cargo build --all --release
echo
./target/release/gather-paths \
    --input /mnt/c/Users/Jose/Documents/Old_CDs\ 1/ /mnt/c/Users/Jose/Documents/Old_CDs\ 2/ --output old_cds.csv \
    --error-log errors_gather.log \
    --recursive
echo
./target/release/filter-paths \
    --input old_cds.csv --output filtered_cds_1.csv \
    --error-log errors_filter_1.log \
    --exclude-unique-sizes \
    --size-min 10000
echo
./target/release/hash-paths \
    --input filtered_cds_1.csv --output hashed_cds_1.csv \
    --bytes 10000 \
    --error-log errors_hash_1.log
echo
./target/release/filter-paths \
    --input hashed_cds_1.csv --output filtered_cds_2.csv \
    --error-log errors_filter_2.log \
    --exclude-unique-hashes
echo
./target/release/hash-paths \
    --input filtered_cds_2.csv --output hashed_cds_2.csv \
    --error-log errors_hash_2.log
echo
./target/release/detect-dups \
    --input hashed_cds_2.csv --output dups.json \
    --error-log errors_dups.log
echo "DONE!"