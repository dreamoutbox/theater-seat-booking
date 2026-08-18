#!/usr/bin/env bash
set -e

# Change to frontend directory
cd "$(dirname "$0")/frontend"

echo "=========================================="
echo "🎨 Starting Vue 3 Frontend Dev Server..."
echo "=========================================="

exec npm run dev
