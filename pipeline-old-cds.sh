#!/usr/bin/env bash

set -euo pipefail

OUT="out/old_cds"

mkdir -p ${OUT}

cargo build --all --release
echo
./target/release/gather-paths \
    --input /mnt/c/Users/Jose/Documents/Old_CDs/ --output ${OUT}/old_cds.csv \
    --error-log ${OUT}/errors_gather.log \
    --recursive \
    | tee -a ${OUT}/shell.log
echo
./target/release/filter-paths \
    --input ${OUT}/old_cds.csv --output ${OUT}/filtered_cds_1.csv \
    --error-log ${OUT}/errors_filter_1.log \
    --exclude-unique-sizes \
    | tee -a ${OUT}/shell.log
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_cds_1.csv --output ${OUT}/hashed_cds_1.csv \
    --bytes 10000 \
    --show-progression \
    --error-log ${OUT}/errors_hash_1.log \
    | tee -a ${OUT}/shell.log
echo
./target/release/filter-paths \
    --input ${OUT}/hashed_cds_1.csv --output ${OUT}/filtered_cds_2.csv \
    --error-log ${OUT}/errors_filter_2.log \
    --exclude-unique-hashes \
    | tee -a ${OUT}/shell.log
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_cds_2.csv --output ${OUT}/hashed_cds_2.csv \
    --show-progression \
    --error-log ${OUT}/errors_hash_2.log \
    | tee -a ${OUT}/shell.log
echo
./target/release/detect-dups \
    --input ${OUT}/hashed_cds_2.csv --output ${OUT}/dups.json \
    --error-log ${OUT}/errors_dups.log \
    | tee -a ${OUT}/shell.log
echo
./target/release/unique-paths \
    --input-dups ${OUT}/dups.json --input-paths ${OUT}/old_cds.csv --output ${OUT}/unique_cds.csv \
    --error-log ${OUT}/error_unique.log \
    | tee -a ${OUT}/shell.log
echo
./target/debug/copy-files \
    --input ${OUT}/paths.csv --output ${OUT}/files/ \
    --show-progression \
    --error-log ${OUT}/error_copy_files.log \
   | tee -a ${OUT}/shell.log
echo
echo "DONE!"