#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Search Things 3
# @raycast.mode fullOutput

# Optional parameters:
# @raycast.icon 🎯
# @raycast.packageName Things 3
# @raycast.description Search your Things 3 tasks and display results
# @raycast.argument1 { "type": "text", "placeholder": "Search query" }
# Setup:
#   1. Install Raycast (raycast.com)
#   2. Copy this file to ~/Documents/Raycast Scripts/ (or any Scripts dir)
#   3. In Raycast: Extensions > + > Add Script Directory > select your folder
#   4. Make executable: chmod +x things3_search.sh
#   5. Search "Search Things 3" in Raycast to run it

set -euo pipefail
export RUST_LOG=off  # suppress things3 log output

THINGS3=$(command -v things3 || echo "${HOME}/.cargo/bin/things3")
QUERY="${1:-}"

if [ ! -x "$THINGS3" ]; then
    echo "things3 not found. Install with: brew install garthdb/tap/things3"
    exit 1
fi

if [ -z "$QUERY" ]; then
    echo "Please provide a search query."
    exit 0
fi

RESULTS=$("$THINGS3" search "$QUERY")

if [ -z "$RESULTS" ] || [ "$RESULTS" = "[]" ]; then
    echo "No tasks found for: $QUERY"
    exit 0
fi

echo "$RESULTS" | python3 -c "
import json, sys
tasks = json.load(sys.stdin)
for t in tasks:
    icon = '✅' if t['status'] == 'completed' else '⬜'
    print(f\"{icon} {t['title']}\")
    if t.get('notes'):
        print(f\"   {t['notes'][:100]}\")
print()
print(f'Found {len(tasks)} task(s)')
"
