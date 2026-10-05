#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "Resetting MinIO S3 Object Storage..."
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

reset_s3_bucket() {
    if docker compose -f docker-compose.base.yml exec -T minio which mc > /dev/null 2>&1; then
        docker compose -f docker-compose.base.yml exec -T minio mc alias set local http://127.0.0.1:9000 "${S3_ACCESS_KEY}" "${S3_SECRET_KEY}" > /dev/null 2>&1 || true
        docker compose -f docker-compose.base.yml exec -T minio mc rb --force "local/${S3_BUCKET}" > /dev/null 2>&1 || true
        docker compose -f docker-compose.base.yml exec -T minio mc mb "local/${S3_BUCKET}" > /dev/null 2>&1 || true
    else
        docker compose -f docker-compose.base.yml exec -T minio rm -rf "/data/${S3_BUCKET}" > /dev/null 2>&1 || true
        docker compose -f docker-compose.base.yml exec -T minio mkdir -p "/data/${S3_BUCKET}" > /dev/null 2>&1 || true
    fi
}

# 1. Check if MinIO container is running via docker compose
if docker compose -f docker-compose.base.yml ps minio --status running --format json 2>/dev/null | grep -q "minio"; then
    echo "Wiping and recreating S3 bucket '${S3_BUCKET}' inside S3 container..."
    reset_s3_bucket
    echo "S3 bucket '${S3_BUCKET}' recreated successfully."
else
    echo "S3 container not running via docker compose"
    echo "Attempting to start S3 service..."
    docker compose -f docker-compose.base.yml up -d minio
    sleep 2
    reset_s3_bucket
    echo "S3 service started and bucket '${S3_BUCKET}' created."
fi

echo "=========================================="
echo "S3 storage reset complete!"
echo "=========================================="
