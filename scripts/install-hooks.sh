#!/bin/bash
# scripts/install-hooks.sh
#
# Configures git to use .githooks/ as the hooks directory and makes
# all hook files executable.
#
# Run once after cloning:
#   ./scripts/install-hooks.sh
#
# How it works:
#   Sets core.hooksPath = .githooks/ (no file copying needed).
#   Hooks stay in sync automatically via git pull.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="${REPO_ROOT}/.githooks"

# Colour output (TTY only)
if [ -t 1 ]; then
    G='\033[0;32m'; Y='\033[1;33m'; R='\033[0;31m'; B='\033[1m'; N='\033[0m'
else
    G=''; Y=''; R=''; B=''; N=''
fi

ok()   { printf "${G}✓${N} %s\n" "$*"; }
warn() { printf "${Y}!${N} %s\n" "$*"; }
err()  { printf "${R}✗${N} %s\n" "$*" >&2; }
bold() { printf "${B}%s${N}\n" "$*"; }

echo ""
bold "Scarff — installing git hooks"
echo ""

# ── 1. Point git at .githooks/ ────────────────────────────────────────────────
git config --local core.hooksPath "${HOOKS_DIR}"
ok "core.hooksPath → .githooks/"

# ── 2. Make all hook files executable ────────────────────────────────────────
HOOKS=(
    commit-msg
    prepare-commit-msg
    pre-commit
    pre-merge-commit
    pre-push
    pre-rebase
    # post-commit
    # post-checkout
    # post-merge
)

MISSING=()
echo ""
for hook in "${HOOKS[@]}"; do
    f="${HOOKS_DIR}/${hook}"
    if [ -f "$f" ]; then
        chmod +x "$f"
        ok "  $hook"
    else
        MISSING+=("$hook")
        warn "  $hook  (missing — expected at .githooks/$hook)"
    fi
done

# ── 3. Check for cog ─────────────────────────────────────────────────────────
echo ""
if command -v cog >/dev/null 2>&1; then
    COG_VER=$(cog --version 2>/dev/null | head -1 || echo "unknown")
    ok "cocogitto found: $COG_VER"
else
    warn "cocogitto (cog) not installed — commit-msg validation will be skipped."
    echo "     Install: cargo install cocogitto"
fi

# ── 4. Result ─────────────────────────────────────────────────────────────────
echo ""
if [ ${#MISSING[@]} -gt 0 ]; then
    err "Missing hooks: ${MISSING[*]}"
    err "Ensure .githooks/ contains all hook scripts and re-run."
    exit 1
fi

bold "All hooks installed. What each one does:"
echo ""
echo "  Commit lifecycle:"
echo "    pre-commit          fmt check + secret scan + whitespace (staged only)"
echo "    prepare-commit-msg  auto-prefixes message from branch name (feat:, fix:, …)"
echo "    commit-msg          validates Conventional Commits via cog"
echo "    post-commit         prints commit summary + context hints"
echo ""
echo "  Merge:"
echo "    pre-merge-commit    validates merge message + blocks WIP commits"
echo "    post-merge          warns when Cargo/toolchain/hooks files changed"
echo ""
echo "  Push:"
echo "    pre-push            clippy + unit tests + cog history check"
echo ""
echo "  Rebase / checkout:"
echo "    pre-rebase          requires clean tree, guards main branch"
echo "    post-checkout       warns when Cargo/toolchain changed on branch switch"
echo ""
echo "  Emergency bypass (use sparingly):"
echo "    git commit --no-verify   skips pre-commit + commit-msg"
echo "    git push   --no-verify   skips pre-push"
echo ""
