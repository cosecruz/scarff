#!/bin/bash
# scripts/install-hooks.sh
# Run this after cloning to set up all hooks

HOOKS_DIR=".git/hooks"
SCRIPTS_DIR="scripts/hooks"

mkdir -p "$SCRIPTS_DIR"

# Create hook files
for hook in commit-msg pre-commit pre-push pre-rebase pre-merge-commit; do
    if [ -f "$SCRIPTS_DIR/$hook" ]; then
        cp "$SCRIPTS_DIR/$hook" "$HOOKS_DIR/$hook"
        chmod +x "$HOOKS_DIR/$hook"
        echo "Installed: $hook"
    fi
done

echo "All hooks installed. Use 'git config --local core.hooksPath .githooks' for custom path."
