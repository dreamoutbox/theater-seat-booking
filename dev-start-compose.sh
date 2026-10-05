#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "Starting Theater Seat Booking Compose Stack (Base + App)..."

if [ "$#" -eq 0 ]; then
    exec docker compose -f docker-compose.base.yml -f docker-compose.app.yml up -d
else
    exec docker compose -f docker-compose.base.yml -f docker-compose.app.yml "$@"
fi
