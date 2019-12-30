#!/usr/bin/env bash

set -euo pipefail

echo "START"

./gather-paths.sh
./filter-paths.sh
./hash-paths.sh
./detect-dups.sh

echo "DONE!"