#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

BACKEND_IMAGE="${BACKEND_IMAGE:-dreamoutbox/theater-seat-booking:latest}"
FRONTEND_IMAGE="${FRONTEND_IMAGE:-dreamoutbox/theater-seat-booking-frontend:latest}"
LOADTEST_IMAGE="${LOADTEST_IMAGE:-dreamoutbox/theater-seat-booking-loadtest:latest}"

echo "Building Backend & Worker Docker Image: $BACKEND_IMAGE"
docker build -t "$BACKEND_IMAGE" -f Dockerfile .

echo "Building Frontend Docker Image: $FRONTEND_IMAGE"
docker build -t "$FRONTEND_IMAGE" -f frontend/Dockerfile frontend

echo "Building Loadtest Docker Image: $LOADTEST_IMAGE"
docker build -t "$LOADTEST_IMAGE" -f loadtest/Dockerfile loadtest

echo "Build completed successfully for $BACKEND_IMAGE, $FRONTEND_IMAGE, and $LOADTEST_IMAGE"
