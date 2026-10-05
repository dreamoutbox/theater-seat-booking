#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "🎭 Resetting Database & Running E2E Tests"
echo "=========================================="

# 1. Reset DB & seed fresh demo data
./dev-reset-db.sh
./dev-reset-s3.sh

# 2. Run Playwright tests in frontend
echo ""
echo "=========================================="
echo "🚀 Executing Playwright E2E Tests..."
echo "=========================================="

WORKER=1

cd frontend

pnpm exec playwright test --workers=$WORKER "$@"
