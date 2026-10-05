#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

BACKEND_IMAGE="${BACKEND_IMAGE:-dreamoutbox/theater-seat-booking:latest}"
FRONTEND_IMAGE="${FRONTEND_IMAGE:-dreamoutbox/theater-seat-booking-frontend:latest}"

echo "Building Backend & Worker Docker Image: $BACKEND_IMAGE"
docker build -t "$BACKEND_IMAGE" -f Dockerfile .

echo "Building Frontend Docker Image: $FRONTEND_IMAGE"
docker build -t "$FRONTEND_IMAGE" -f frontend/Dockerfile frontend

echo "Build completed successfully for $BACKEND_IMAGE and $FRONTEND_IMAGE"
