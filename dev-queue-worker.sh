#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "⚡ Starting Queue Worker & Hold Sweeper..."
echo "=========================================="

export RUST_LOG="${RUST_LOG:-info,theater_seat_booking=debug,queue_worker=debug}"

if command -v cargo-watch &> /dev/null; then
    echo "⚡ cargo-watch detected! Running worker with hot reload..."
    exec cargo watch -x "run --bin queue_worker"
else
    echo "Running worker with cargo run..."
    exec cargo run --bin queue_worker
fi
