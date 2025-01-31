#!/bin/sh
set -e

until nc -z -v -w30 db 3306; do
  echo "Waiting for MySQL to start..."
  sleep 5
done

echo "MySQL is up - running migrations..."
cargo sqlx migrate run

echo "Starting the app..."
exec ./target/release/lift-rust
