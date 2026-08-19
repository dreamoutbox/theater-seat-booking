#!/usr/bin/env bash
set -e

# Change to project root directory
cd "$(dirname "$0")"

echo "=========================================="
echo "🔄 Resetting Database & Running Migrations..."
echo "=========================================="

# Load environment variables if .env exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs -d '\n')
fi

DATABASE_URL="${DATABASE_URL:-postgres://postgres:example@localhost:5432/postgres}"

echo "Database URL: ${DATABASE_URL}"

# 1. Reset database schema (Drop and recreate public schema)
echo "🧹 Dropping and recreating public schema..."
if command -v psql &> /dev/null; then
    psql "$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO postgres; GRANT ALL ON SCHEMA public TO public;" > /dev/null 2>&1 || {
        echo "⚠️  Direct psql failed, trying docker compose..."
        docker compose exec -T postgres psql -U postgres -d postgres -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO postgres; GRANT ALL ON SCHEMA public TO public;"
    }
else
    echo "Using docker compose to reset schema..."
    docker compose exec -T postgres psql -U postgres -d postgres -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO postgres; GRANT ALL ON SCHEMA public TO public;"
fi

echo "✅ Database schema reset successfully."

# 2. Run SQLx migrations and seed sample data
echo "🌱 Running migrations and seeding sample data..."
cargo run --bin seed

echo "=========================================="
echo "✨ Database reset and migrations complete!"
echo "=========================================="
