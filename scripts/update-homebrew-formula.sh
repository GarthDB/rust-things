#!/bin/bash
# Updates the Homebrew formula in homebrew-tap/ and pushes to GarthDB/homebrew-tap.
# Usage: ./scripts/update-homebrew-formula.sh <version>
# Example: ./scripts/update-homebrew-formula.sh 2.0.0

set -euo pipefail

VERSION=${1:-""}
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>  (e.g. $0 2.0.0)"
    exit 1
fi

REPO="GarthDB/rust-things3"
TARBALL_URL="https://github.com/${REPO}/archive/v${VERSION}.tar.gz"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TAP_DIR="${PROJECT_ROOT}/homebrew-tap"
TAP_FORMULA="${TAP_DIR}/Formula/things3-cli.rb"
LOCAL_FORMULA="${PROJECT_ROOT}/Formula/things3-cli.rb"
TMPFILE="/tmp/rust-things3-${VERSION}.tar.gz"

# Verify the release exists on GitHub before doing any work.
echo "Verifying GitHub release v${VERSION}..."
if ! gh release view "v${VERSION}" --repo "${REPO}" &>/dev/null; then
    echo "Error: GitHub release v${VERSION} not found. Publish it first."
    exit 1
fi

# Ensure the tap remote uses SSH (avoids HTTPS credential prompts).
git -C "${TAP_DIR}" remote set-url origin git@github.com:GarthDB/homebrew-tap.git

# Pull latest tap state to avoid push conflicts.
echo "Pulling latest homebrew-tap..."
git -C "${TAP_DIR}" pull --ff-only origin main

# Download the source tarball and compute SHA256.
echo "Downloading ${TARBALL_URL}..."
curl -fsSL -o "${TMPFILE}" "${TARBALL_URL}"

echo "Computing SHA256..."
if command -v sha256sum &>/dev/null; then
    SHA256=$(sha256sum "${TMPFILE}" | cut -d' ' -f1)
else
    SHA256=$(shasum -a 256 "${TMPFILE}" | cut -d' ' -f1)
fi
rm "${TMPFILE}"
echo "SHA256: ${SHA256}"

# Update formula helper — rewrites url + sha256 lines in place.
update_formula() {
    local file="$1"
    sed -i.bak \
        -e "s|url \".*\"|url \"${TARBALL_URL}\"|" \
        -e "s|sha256 \"[^\"]*\"|sha256 \"${SHA256}\"|" \
        "${file}"
    rm "${file}.bak"
}

echo "Updating tap formula: ${TAP_FORMULA}"
update_formula "${TAP_FORMULA}"

echo "Updating local reference formula: ${LOCAL_FORMULA}"
update_formula "${LOCAL_FORMULA}"

# Commit and push the tap.
echo "Committing and pushing homebrew-tap..."
git -C "${TAP_DIR}" add Formula/things3-cli.rb
git -C "${TAP_DIR}" commit -m "Update things3-cli to v${VERSION}"
git -C "${TAP_DIR}" push origin main

echo ""
echo "Done. Install with:"
echo "  brew tap GarthDB/tap"
echo "  brew install GarthDB/tap/things3-cli"
