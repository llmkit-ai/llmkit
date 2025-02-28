#!/bin/bash
set -e

# Create database directory if needed
DB_PATH="${DATABASE_URL#sqlite:}"
DB_DIR=$(dirname "$DB_PATH")
mkdir -p "$DB_DIR"

# Check if we have any command line arguments
if [ $# -gt 0 ]; then
    # If arguments were passed, run them instead of the default command
    exec "$@"
else
    # Run the application (it will handle DB creation and migrations)
    exec ./backend
fi