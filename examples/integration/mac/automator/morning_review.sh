#!/bin/bash
# Automator integration — Morning Review
#
# Setup in Automator:
#   1. Create a new Workflow or Application
#   2. Add action: Utilities > Run Shell Script
#   3. Set Shell: /bin/bash
#   4. Set Pass input: to stdin (or ignore input)
#   5. Paste this script into the script field
#
# The workflow fetches your inbox tasks and displays a summary notification.

set -euo pipefail
export RUST_LOG=off  # suppress things3 log output

THINGS3=$(command -v things3 || echo "${HOME}/.cargo/bin/things3")

if [ ! -x "$THINGS3" ]; then
    osascript -e 'display notification "things3 not found. Install with: brew install garthdb/tap/things3" with title "Morning Review"'
    exit 1
fi

# Fetch inbox tasks
INBOX_JSON=$("$THINGS3" inbox)
INBOX_COUNT=$(echo "$INBOX_JSON" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)

# Format a quick summary to display
SUMMARY="Inbox: ${INBOX_COUNT} task(s)"

# Show a system notification
osascript -e "display notification \"${SUMMARY}\" with title \"Things 3 Morning Review\" sound name \"Glass\""

# Also print to stdout (useful when run from Terminal or Automator log)
echo "=== Morning Review ==="
echo "$SUMMARY"
echo ""

if [ "$INBOX_COUNT" -gt 0 ]; then
    echo "Inbox tasks:"
    echo "$INBOX_JSON" | python3 -c "
import json, sys
tasks = json.load(sys.stdin)
for t in tasks[:10]:
    print(f'  • {t[\"title\"]}')
if len(tasks) > 10:
    print(f'  ... and {len(tasks) - 10} more')
"
fi
