#!/bin/sh
set -e
sqlx migrate run --database-url "$DATABASE_URL" --source /app/migrations
exec neoland "$@"
