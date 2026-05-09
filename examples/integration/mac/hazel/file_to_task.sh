#!/bin/bash
# Hazel integration — Create a Things 3 task when a file matches a rule
#
# Setup in Hazel (noodlesoft.com):
#   1. Add a folder to watch (e.g., ~/Downloads)
#   2. Add a rule with your conditions (e.g., Kind is PDF, Name contains "receipt")
#   3. Add action: Run Shell Script
#   4. Set to: Embedded Script, Shell: /bin/bash
#   5. Paste this script into the editor
#
# Hazel passes the matched file path as the first argument ($1).
# The script creates a Things 3 task referencing the file.

set -euo pipefail
export RUST_LOG=off  # suppress things3 log output

FILE_PATH="${1:-}"

if [ -z "$FILE_PATH" ]; then
    echo "Usage: $0 <file_path>" >&2
    exit 1
fi

FILE_NAME=$(basename "$FILE_PATH")
DATE=$(date +"%Y-%m-%d")

# Build the task title from the filename
TITLE="Process: ${FILE_NAME}"

# Build notes with file metadata
FILE_SIZE=$(du -sh "$FILE_PATH" 2>/dev/null | cut -f1 || echo "unknown")
NOTES="File: ${FILE_PATH}\nSize: ${FILE_SIZE}\nMatched on: ${DATE}"

# URL-encode values for the Things URL scheme
encode() {
    python3 -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1]))" "$1"
}

ENCODED_TITLE=$(encode "$TITLE")
ENCODED_NOTES=$(encode "$(printf '%b' "$NOTES")")

# Open Things 3 URL scheme to create the task
# Add tags and deadline as needed; see: https://culturedcode.com/things/support/articles/2803573/
URL="things:///add?title=${ENCODED_TITLE}&notes=${ENCODED_NOTES}&tags=hazel"
open "$URL"

echo "Created task: $TITLE"
