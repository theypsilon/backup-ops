#!/usr/bin/env bash

set -euo pipefail

INPUT="${1:-/mnt/c/}"
OUT="$(pwd)/out/${2:-music}"

mkdir -p ${OUT}

cargo build --all --release
echo
./target/release/gather-paths \
    --input ${INPUT} --output ${OUT}/all_paths.csv \
    --error-log ${OUT}/errors_all_paths.log \
    --recursive
echo
./target/release/filter-paths \
    --input ${OUT}/all_paths.csv --output ${OUT}/all_pics_1.csv \
    --error-log ${OUT}/errors_all_pics_1.log \
    --whitelist-path-ends ":case-insensitive:!.mp3" ":case-insensitive:!.ogg"
echo
./target/release/filter-paths \
    --input ${OUT}/all_pics_1.csv --output ${OUT}/filtered_1.csv \
    --error-log ${OUT}/errors_filtered_1.log \
    --exclude-unique-sizes
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_1.csv --output ${OUT}/mini_hash.csv \
    --bytes 10000 \
    --error-log ${OUT}/errors_mini_hash.log
echo
./target/release/filter-paths \
    --input ${OUT}/mini_hash.csv --output ${OUT}/filtered_2.csv \
    --error-log ${OUT}/errors_filter_2.log \
    --exclude-unique-hashes
echo
./target/release/hash-paths \
    --input ${OUT}/filtered_2.csv --output ${OUT}/full_hash.csv \
    --error-log ${OUT}/errors_full_hash.log
echo
./target/release/detect-dups \
    --input ${OUT}/full_hash.csv --output ${OUT}/dups.json \
    --error-log ${OUT}/errors_dups.log
echo
./target/release/unique-paths \
    --input-dups ${OUT}/dups.json --input-paths ${OUT}/all_pics_1.csv --output ${OUT}/unique.csv \
    --error-log ${OUT}/error_unique.log
echo
./target/release/copy-files \
    --input ${OUT}/unique.csv --output ${OUT}/mp3/ \
    --error-log ${OUT}/error_copy_files.log \
    --flatten-output
echo
echo "DONE!"