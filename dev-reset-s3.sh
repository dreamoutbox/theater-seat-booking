#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "🪣 Resetting MinIO S3 Object Storage..."
echo "=========================================="

# Load environment variables if .env exists
if [ -f .env ]; then
    set -a
    source .env
    set +a
fi

S3_ENDPOINT="${S3_ENDPOINT:-http://127.0.0.1:9000}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-miniominio}"
S3_SECRET_KEY="${S3_SECRET_KEY:-miniominio}"
S3_BUCKET="${S3_BUCKET:-theater-documents}"

echo "S3 Endpoint:   ${S3_ENDPOINT}"
echo "Target Bucket: ${S3_BUCKET}"

# 1. Check if MinIO container is running via docker compose
if docker compose ps minio --status running --format json 2>/dev/null | grep -q "minio"; then
    echo "🧹 Wiping and recreating S3 bucket '${S3_BUCKET}' inside MinIO container..."
    
    # Configure minio alias inside container
    docker compose exec -T minio mc alias set local http://127.0.0.1:9000 "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}" > /dev/null 2>&1 || true

    # Force remove all objects and bucket if exists
    docker compose exec -T minio mc rb --force "local/${S3_BUCKET}" > /dev/null 2>&1 || true

    # Recreate the fresh bucket
    docker compose exec -T minio mc mb "local/${S3_BUCKET}" > /dev/null 2>&1 || true
    
    echo "✅ MinIO bucket '${S3_BUCKET}' recreated successfully."
else
    echo "⚠️  MinIO container not running via docker compose."
    echo "Attempting to start MinIO service..."
    docker compose up -d minio
    sleep 2
    
    docker compose exec -T minio mc alias set local http://127.0.0.1:9000 "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}" > /dev/null 2>&1 || true
    docker compose exec -T minio mc rb --force "local/${S3_BUCKET}" > /dev/null 2>&1 || true
    docker compose exec -T minio mc mb "local/${S3_BUCKET}" > /dev/null 2>&1 || true
    
    echo "✅ MinIO service started and bucket '${S3_BUCKET}' created."
fi

echo "=========================================="
echo "✨ S3 storage reset complete!"
echo "=========================================="
