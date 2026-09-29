#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "Starting Theater Seat Booking Dev Stack..."
echo "=========================================="

# 1. Start Docker Compose infrastructure services in the background
echo "Starting Docker Compose services (Postgres, S3/RustFS, Adminer, Stripe CLI)..."
docker compose up -d

# 2. Wait for PostgreSQL to be ready before launching backend and worker
echo "Waiting for PostgreSQL to accept connections..."
until docker compose exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 0.5
done
echo "PostgreSQL is ready."

# 3. Track background processes and trap signals for clean shutdown
pids=()

cleanup() {
    trap - SIGINT SIGTERM EXIT
    echo ""
    echo "Shutting down development services..."
    for pid in "${pids[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            pkill -P "$pid" 2>/dev/null || true
            kill "$pid" 2>/dev/null || true
        fi
    done
    wait "${pids[@]}" 2>/dev/null || true
    echo "All development services stopped."
}

trap cleanup SIGINT SIGTERM EXIT

# 4. Start watchers in parallel
echo "Starting Backend API..."
./dev-backend.sh &
pids+=($!)

echo "Starting Queue Worker..."
./dev-queue-worker.sh &
pids+=($!)

echo "Starting Frontend Dev Server..."
./dev-frontend.sh &
pids+=($!)

echo "=========================================="
echo "Services running in parallel:"
echo "  - Backend API:       http://localhost:3000"
echo "  - Frontend:          http://localhost:5173"
echo "  - Adminer (DB):      http://localhost:8081"
echo "  - Object Storage:    http://localhost:9000 (Console: :9001)"
echo ""
echo "Press Ctrl+C to stop all services."
echo "=========================================="

# Wait for background processes
wait "${pids[@]}"
