#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "🎬 Starting Theater Seat Booking API..."
echo "=========================================="

export RUST_LOG="${RUST_LOG:-info,theater_seat_booking=debug}"
export EMBEDDED_WORKER="${EMBEDDED_WORKER:-false}"

if command -v cargo-watch &> /dev/null; then
    echo "⚡ cargo-watch detected! Running with hot reload..."
    exec cargo watch -x "run --bin theater-api"
else
    echo "Running backend API with cargo run..."
    exec cargo run --bin theater-api
fi
