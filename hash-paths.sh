#!/usr/bin/env bash

set -xeuo pipefail

cargo run --release -p hash-paths -- "$@"