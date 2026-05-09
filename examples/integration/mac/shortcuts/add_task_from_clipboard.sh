#!/bin/bash
# Shortcuts integration — "Add Task from Clipboard"
#
# Setup in Apple Shortcuts:
#   1. Add action: Scripting > Run Shell Script
#   2. Set Shell: /bin/bash
#   3. Set Input: Shortcut Input (passes clipboard text via stdin)
#   4. Paste this script into the script field
#
# The shortcut reads clipboard text and creates a Things 3 task from it.

set -euo pipefail

# Read task title from stdin (Shortcuts passes "Shortcut Input" here)
TITLE=$(cat)

if [ -z "$TITLE" ]; then
    echo "Error: No title provided. Copy text to clipboard first." >&2
    exit 1
fi

# Optional: read notes from the second line onward if multi-line input
FIRST_LINE=$(echo "$TITLE" | head -1)
REMAINING=$(echo "$TITLE" | tail -n +2)

# URL-encode the title and notes for the Things URL scheme
encode() {
    python3 -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1]))" "$1"
}

ENCODED_TITLE=$(encode "$FIRST_LINE")
URL="things:///add?title=${ENCODED_TITLE}"

if [ -n "$REMAINING" ]; then
    ENCODED_NOTES=$(encode "$REMAINING")
    URL="${URL}&notes=${ENCODED_NOTES}"
fi

# Open the Things URL scheme to create the task
open "$URL"

echo "Task created: $FIRST_LINE"
