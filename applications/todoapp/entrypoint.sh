#!/bin/bash

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
until pg_isready -d $DATABASE_URL; do
  sleep 2
done

# Run database migrations or initialization (if necessary)
echo "Running database migrations..."
psql $DATABASE_URL -f /docker-entrypoint-initdb.d/init.sql

# Start the Rust app
echo "Starting the Rust app..."
./todoapp
