#!/usr/bin/env bash
set -euo pipefail

# Configuration parameters with override support
APP_NAME="theater-seat-booking"
APP_DIR="/app/${APP_NAME}"
RUN_USER="theater"
RUN_GROUP="theater"
GITHUB_REPO="${GITHUB_REPO:-dreamoutbox/theater-seat-booking}"
RELEASE_TAG="${RELEASE_TAG:-latest}"
PORT="${PORT:-3000}"
DATABASE_URL="${DATABASE_URL:-postgres://postgres:example@localhost:5432/postgres}"
S3_ENDPOINT="${S3_ENDPOINT:-http://127.0.0.1:9000}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-miniominio}"
S3_SECRET_KEY="${S3_SECRET_KEY:-miniominio}"
S3_BUCKET="${S3_BUCKET:-theater-documents}"
S3_REGION="${S3_REGION:-us-east-1}"
JWT_SECRET="${JWT_SECRET:-theater_booking_poc_jwt_secret_key_1234567890_super_secure}"

# Asserts that the executing user has root privileges.
#
# Arguments:
#   None
#
# Example usage:
#   check_root
check_root() {
  if [ "$(id -u)" -ne 0 ]; then
    echo "Error: This script must be executed as root." >&2
    exit 1
  fi
}

# Installs required base operating system packages.
#
# Arguments:
#   None
#
# Example usage:
#   install_prerequisites
install_prerequisites() {
  echo "Installing system prerequisites..."
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq \
    curl \
    ca-certificates \
    gnupg \
    lsb-release \
    tar \
    jq \
    sudo
}

# Configures the PostgreSQL PGDG repository and installs PostgreSQL 18 on bare metal.
#
# Arguments:
#   None
#
# Example usage:
#   setup_postgresql_18
setup_postgresql_18() {
  echo "Setting up PostgreSQL 18..."
  install -d -m 0755 /etc/apt/keyrings

  if [ ! -f /etc/apt/keyrings/postgresql.gpg ]; then
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /etc/apt/keyrings/postgresql.gpg
  fi

  local codename
  codename=$(lsb_release -cs)
  echo "deb [signed-by=/etc/apt/keyrings/postgresql.gpg] http://apt.postgresql.org/pub/repos/apt ${codename}-pgdg main" \
    > /etc/apt/sources.list.d/pgdg.list

  apt-get update -qq
  apt-get install -y -qq postgresql-18 postgresql-client-18

  systemctl enable postgresql
  systemctl start postgresql

  echo "Configuring PostgreSQL user and database..."
  sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'example';"
  sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname = 'postgres'" | grep -q 1 || \
    sudo -u postgres psql -c "CREATE DATABASE postgres;"

  local pg_hba_conf="/etc/postgresql/18/main/pg_hba.conf"
  if [ -f "$pg_hba_conf" ]; then
    if ! grep -q "host    all             postgres        127.0.0.1/32            scram-sha-256" "$pg_hba_conf"; then
      sed -i '1s/^/host    all             postgres        127.0.0.1\/32            scram-sha-256\n/' "$pg_hba_conf"
      systemctl reload postgresql
    fi
  fi
}

# Downloads and installs the SeaweedFS server binary and configures S3 service.
#
# Arguments:
#   None
#
# Example usage:
#   setup_seaweedfs
setup_seaweedfs() {
  echo "Setting up SeaweedFS S3 server..."
  local arch
  arch=$(uname -m)
  local weed_arch
  case "$arch" in
    x86_64) weed_arch="linux_amd64" ;;
    aarch64|arm64) weed_arch="linux_arm64" ;;
    *)
      echo "Error: Unsupported architecture '$arch' for SeaweedFS." >&2
      exit 1
      ;;
  esac

  if [ ! -f /usr/local/bin/weed ]; then
    local download_url="https://github.com/seaweedfs/seaweedfs/releases/latest/download/${weed_arch}.tar.gz"
    echo "Downloading SeaweedFS from $download_url..."
    curl -fsSL "$download_url" -o /tmp/seaweedfs.tar.gz
    tar -xzf /tmp/seaweedfs.tar.gz -C /usr/local/bin weed
    chmod +x /usr/local/bin/weed
    rm -f /tmp/seaweedfs.tar.gz
  fi

  if ! id -u seaweedfs >/dev/null 2>&1; then
    useradd --system --no-create-home --shell /usr/sbin/nologin seaweedfs
  fi

  mkdir -p /var/lib/seaweedfs /etc/seaweedfs
  cat <<EOF > /etc/seaweedfs/s3.json
{
  "identities": [
    {
      "name": "theater",
      "credentials": [
        {
          "accessKey": "${S3_ACCESS_KEY}",
          "secretKey": "${S3_SECRET_KEY}"
        }
      ],
      "actions": [
        "Read",
        "Write",
        "List",
        "Tagging",
        "Admin"
      ]
    }
  ]
}
EOF
  chown -R seaweedfs:seaweedfs /var/lib/seaweedfs /etc/seaweedfs

  cat <<EOF > /etc/systemd/system/seaweedfs.service
[Unit]
Description=SeaweedFS Server and S3 API
After=network.target

[Service]
Type=simple
User=seaweedfs
Group=seaweedfs
ExecStart=/usr/local/bin/weed server -s3 -s3.port=9000 -s3.config=/etc/seaweedfs/s3.json -dir=/var/lib/seaweedfs
Restart=always
RestartSec=3
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

  systemctl daemon-reload
  systemctl enable --now seaweedfs.service

  echo "Waiting for SeaweedFS S3 service to accept connections..."
  local attempts=0
  while ! curl -s "http://127.0.0.1:9000/" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge 30 ]; then
      echo "Error: SeaweedFS S3 did not start in time." >&2
      exit 1
    fi
    sleep 1
  done
}

# Creates the target S3 bucket in SeaweedFS.
#
# Arguments:
#   $1 - Name of the bucket to create (e.g., "theater-documents")
#
# Example usage:
#   setup_s3_bucket "theater-documents"
setup_s3_bucket() {
  local bucket_name="$1"
  echo "Ensuring S3 bucket '$bucket_name' exists..."
  mkdir -p "/var/lib/seaweedfs/buckets/${bucket_name}"
  chown -R seaweedfs:seaweedfs "/var/lib/seaweedfs/buckets"
}

# Fetches and extracts the latest application release from GitHub.
#
# Arguments:
#   $1 - Destination installation directory (e.g., "/app/theater-seat-booking")
#
# Example usage:
#   fetch_and_extract_release "/app/theater-seat-booking"
fetch_and_extract_release() {
  local target_dir="$1"
  echo "Fetching release for repository ${GITHUB_REPO} (tag: ${RELEASE_TAG})..."

  local api_url
  if [ "$RELEASE_TAG" = "latest" ]; then
    api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
  else
    api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/tags/${RELEASE_TAG}"
  fi

  local auth_header=()
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    auth_header=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
  fi

  local release_json
  release_json=$(curl -fsSL "${auth_header[@]}" -H "Accept: application/vnd.github.v3+json" "$api_url" || true)

  if [ -z "$release_json" ] || echo "$release_json" | grep -q '"message": "Not Found"'; then
    echo "Error: Release not found at $api_url." >&2
    echo "Ensure a release has been published or provide BACKEND_ARCHIVE_PATH and FRONTEND_ARCHIVE_PATH." >&2
    exit 1
  fi

  local backend_url
  backend_url=$(echo "$release_json" | jq -r '.assets[] | select(.name | test("linux-x86_64\\.tar\\.gz$")) | .browser_download_url' | head -n 1)

  local frontend_url
  frontend_url=$(echo "$release_json" | jq -r '.assets[] | select(.name | test("frontend.*\\.tar\\.gz$")) | .browser_download_url' | head -n 1)

  if [ -z "$backend_url" ] || [ "$backend_url" = "null" ]; then
    echo "Error: Could not locate backend archive in release assets." >&2
    exit 1
  fi

  if [ -z "$frontend_url" ] || [ "$frontend_url" = "null" ]; then
    echo "Error: Could not locate frontend archive in release assets." >&2
    exit 1
  fi

  mkdir -p "$target_dir"
  mkdir -p "${target_dir}/frontend/dist"

  echo "Downloading backend assets from $backend_url..."
  curl -fsSL -L "$backend_url" -o /tmp/backend-release.tar.gz
  tar -xzf /tmp/backend-release.tar.gz -C "$target_dir" --strip-components=1
  rm -f /tmp/backend-release.tar.gz

  echo "Downloading frontend assets from $frontend_url..."
  curl -fsSL -L "$frontend_url" -o /tmp/frontend-release.tar.gz
  tar -xzf /tmp/frontend-release.tar.gz -C "${target_dir}/frontend/dist"
  rm -f /tmp/frontend-release.tar.gz

  chmod +x "${target_dir}/theater-api" "${target_dir}/queue_worker" "${target_dir}/seed"
}

# Creates the system execution user for systemd.
#
# Arguments:
#   $1 - User name to create (e.g., "theater")
#
# Example usage:
#   setup_run_user "theater"
setup_run_user() {
  local user_name="$1"
  if ! id -u "$user_name" >/dev/null 2>&1; then
    echo "Creating system execution user '$user_name'..."
    useradd --system --no-create-home --shell /usr/sbin/nologin "$user_name"
  fi
}

# Writes the production environment configuration file.
#
# Arguments:
#   $1 - Directory where .env should be written (e.g., "/app/theater-seat-booking")
#
# Example usage:
#   setup_environment_config "/app/theater-seat-booking"
setup_environment_config() {
  local target_dir="$1"
  echo "Writing environment configuration to ${target_dir}/.env..."

  cat <<EOF > "${target_dir}/.env"
RUST_LOG=info,theater_seat_booking=debug
PORT=${PORT}
DATABASE_URL=${DATABASE_URL}
JWT_SECRET=${JWT_SECRET}
HOLD_TTL_SECONDS=600
POLL_INTERVAL_MS=1000
MAX_BOOKINGS_PER_USER=4
STRIPE_SKIP_PAYMENT=true
STRIPE_SECRET_KEY=sk_test_mock
STRIPE_WEBHOOK_SECRET=whsec_mock
STRIPE_CURRENCY=thb
S3_ENDPOINT=${S3_ENDPOINT}
S3_ACCESS_KEY=${S3_ACCESS_KEY}
S3_SECRET_KEY=${S3_SECRET_KEY}
S3_BUCKET=${S3_BUCKET}
S3_REGION=${S3_REGION}
EOF

  chown "${RUN_USER}:${RUN_GROUP}" "${target_dir}/.env"
  chmod 0600 "${target_dir}/.env"
}

# Configures and starts the systemd service with automatic restart.
#
# Arguments:
#   $1 - Target installation directory (e.g., "/app/theater-seat-booking")
#
# Example usage:
#   setup_systemd_service "/app/theater-seat-booking"
setup_systemd_service() {
  local target_dir="$1"
  echo "Configuring systemd service for ${APP_NAME}..."

  cat <<EOF > /etc/systemd/system/${APP_NAME}.service
[Unit]
Description=Theater Seat Booking API & Web Server
After=network.target postgresql.service seaweedfs.service
Wants=postgresql.service seaweedfs.service

[Service]
Type=simple
User=${RUN_USER}
Group=${RUN_GROUP}
WorkingDirectory=${target_dir}
EnvironmentFile=${target_dir}/.env
ExecStart=${target_dir}/theater-api
Restart=always
RestartSec=3
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

  systemctl daemon-reload
  systemctl enable --now "${APP_NAME}.service"
  echo "Service ${APP_NAME} enabled and started."
}

# Executes the database migrations and demo data seeding.
#
# Arguments:
#   $1 - Installation directory containing seed executable (e.g., "/app/theater-seat-booking")
#
# Example usage:
#   run_migrations_and_seed "/app/theater-seat-booking"
run_migrations_and_seed() {
  local target_dir="$1"
  echo "Running database schema migrations and seeding mock data..."
  (
    cd "$target_dir"
    sudo -u "$RUN_USER" env \
      DATABASE_URL="$DATABASE_URL" \
      JWT_SECRET="$JWT_SECRET" \
      ./seed
  )
}

main() {
  check_root
  install_prerequisites
  setup_postgresql_18
  setup_seaweedfs
  setup_s3_bucket "$S3_BUCKET"
  setup_run_user "$RUN_USER"
  fetch_and_extract_release "$APP_DIR"
  setup_environment_config "$APP_DIR"

  chown -R "${RUN_USER}:${RUN_GROUP}" "$APP_DIR"

  run_migrations_and_seed "$APP_DIR"
  setup_systemd_service "$APP_DIR"

  echo ""
  echo "========================================================"
  echo "Theater Seat Booking demo setup completed successfully."
  echo "Application is running on: http://localhost:${PORT}"
  echo "SeaweedFS S3 is running on: ${S3_ENDPOINT}"
  echo "Database is running on: PostgreSQL 18"
  echo "========================================================"
}

main "$@"
