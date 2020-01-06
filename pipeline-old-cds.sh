#!/usr/bin/env bash

set -euo pipefail

OUT="out/old_cds"

mkdir -p ${OUT}

cargo build --all --release
echo
./target/release/gather-paths \
    --input /mnt/c/Users/Jose/Documents/Old_CDs\ 1/ /mnt/c/Users/Jose/Documents/Old_CDs\ 2/ --output ${OUT}/old_cds.csv \
    --error-log ${OUT}/errors_gather.log \
    --recursive
echo 
./target/release/filter-paths \
    --input ${OUT}/old_cds.csv --output ${OUT}/filtered_cds_1.csv \
    --error-log ${OUT}/errors_filter_1.log \
    --exclude-unique-sizes \
    --size-min 10000
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_cds_1.csv --output ${OUT}/hashed_cds_1.csv \
    --bytes 10000 \
    --error-log ${OUT}/errors_hash_1.log
echo
./target/release/filter-paths \
    --input ${OUT}/hashed_cds_1.csv --output ${OUT}/filtered_cds_2.csv \
    --error-log ${OUT}/errors_filter_2.log \
    --exclude-unique-hashes
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_cds_2.csv --output ${OUT}/hashed_cds_2.csv \
    --error-log ${OUT}/errors_hash_2.log
echo
./target/release/detect-dups \
    --input ${OUT}/hashed_cds_2.csv --output ${OUT}/dups.json \
    --error-log ${OUT}/errors_dups.log
echo
./target/release/unique-paths \
    --input-dups ${OUT}/dups.json --input-paths ${OUT}/old_cds.csv --output ${OUT}/unique_cds.csv \
    --error-log ${OUT}/error_unique.log
echo
echo "DONE!"