#!/bin/bash
# Alfred Script Filter — Search Things 3
#
# Setup in Alfred:
#   1. Open Alfred Preferences > Workflows > + > Blank Workflow
#   2. Add Input: Script Filter
#      - Language: /bin/bash
#      - Script: paste this file's contents
#      - Keyword: t3 (or your choice)
#      - Argument: Required
#   3. Add Output: Open URL  (set URL to: things:///show?id={query})
#      OR copy the UUID arg using a Copy to Clipboard action
#   4. Connect Script Filter → Open URL
#
# Alfred passes the search query as $1 (with "with input as argv").

set -euo pipefail
export RUST_LOG=off  # suppress things3 log output

THINGS3=$(command -v things3 || echo "${HOME}/.cargo/bin/things3")
QUERY="${1:-}"

if [ ! -x "$THINGS3" ]; then
    echo '{"items":[{"title":"things3 not found","subtitle":"Install with: brew install garthdb/tap/things3","valid":false}]}'
    exit 0
fi

if [ -z "$QUERY" ]; then
    echo '{"items":[{"title":"Search Things 3","subtitle":"Type to search your tasks","valid":false}]}'
    exit 0
fi

RESULTS=$("$THINGS3" search "$QUERY")

if [ -z "$RESULTS" ] || [ "$RESULTS" = "[]" ]; then
    echo "$QUERY" | python3 -c "
import json, sys
q = sys.stdin.read().strip()
print(json.dumps({'items': [{'title': f'No results for: {q}', 'valid': False}]}))
"
    exit 0
fi

# Convert things3 JSON output to Alfred Script Filter JSON
echo "$RESULTS" | python3 -c "
import json, sys

tasks = json.load(sys.stdin)
items = []
for t in tasks:
    icon = '✅ ' if t['status'] == 'completed' else ''
    items.append({
        'uid': t['uuid'],
        'title': icon + t['title'],
        'subtitle': (t.get('notes') or t['status'])[:80],
        'arg': t['uuid'],
        'autocomplete': t['title'],
    })
print(json.dumps({'items': items}))
"
