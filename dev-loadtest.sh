#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

LOADTEST_IMAGE="${LOADTEST_IMAGE:-dreamoutbox/theater-seat-booking-loadtest:latest}"
VUS="${VUS:-1000}"
DURATION="${DURATION:-30s}"
TARGET_URL="${TARGET_URL:-http://backend:3000/api}"
RESET=true

show_help() {
  cat << EOF
Usage: ./dev-loadtest.sh [OPTIONS] [-- K6_ARGS...]

Run the k6 load testing suite in Docker simulating 1000+ users big event ticket sale.

Options:
  -v, --vus <N>          Number of virtual users (default: 1000)
  -d, --duration <D>     Test duration, e.g. 30s, 1m (default: 30s)
  -u, --target-url <URL> API base URL (default: http://backend:3000/api)
  --no-reset             Skip resetting database and S3 storage
  -h, --help             Display this help message

Examples:
  ./dev-loadtest.sh
  ./dev-loadtest.sh --vus 50 --duration 15s
  ./dev-loadtest.sh --vus 1000 --duration 1m
EOF
}

# Parse CLI options
EXTRA_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--vus)
      VUS="$2"
      shift 2
      ;;
    -d|--duration)
      DURATION="$2"
      shift 2
      ;;
    -u|--target-url)
      TARGET_URL="$2"
      shift 2
      ;;
    --no-reset)
      RESET=false
      shift
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    --)
      shift
      EXTRA_ARGS+=("$@")
      break
      ;;
    *)
      EXTRA_ARGS+=("$1")
      shift
      ;;
  esac
done

echo "=========================================="
echo "Theater Seat Booking - k6 Load Testing"
echo "=========================================="
echo "Virtual Users: $VUS"
echo "Duration:      $DURATION"
echo "Target URL:    $TARGET_URL"
echo "Reset DB & S3: $RESET"
echo "=========================================="

# 1. Start Base services (Postgres, MinIO)
echo "Ensuring Base services are running..."
docker compose -f docker-compose.base.yml up -d

# 2. Reset Database and S3 storage
if [ "$RESET" = true ]; then
  echo "Resetting database and running initial migrations..."
  ./dev-reset-db.sh
  echo "Resetting MinIO S3 object storage..."
  ./dev-reset-s3.sh
  echo "Seeding high-capacity load test environment..."
  cargo run --bin seed_loadtest
elif [ ! -f loadtest/test-data.json ]; then
  echo "Test data not found; seeding high-capacity load test environment..."
  cargo run --bin seed_loadtest
fi

# 3. Start Base and App services before loadtest
echo "Starting Base and App compose services..."
docker compose -f docker-compose.base.yml -f docker-compose.app.yml up -d
docker compose -f docker-compose.base.yml -f docker-compose.app.yml restart backend queue-worker

# 4. Pre-build the loadtest Docker image (no build in docker compose)
echo "Pre-building loadtest container image: $LOADTEST_IMAGE..."
docker build -t "$LOADTEST_IMAGE" -f loadtest/Dockerfile loadtest

# 5. Run k6 load test container
echo "Running k6 load test..."
exec docker compose -f docker-compose.loadtest.yml run --rm \
  -e VUS="$VUS" \
  -e DURATION="$DURATION" \
  -e TARGET_URL="$TARGET_URL" \
  loadtest "${EXTRA_ARGS[@]}"
