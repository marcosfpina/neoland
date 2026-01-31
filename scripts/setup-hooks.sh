#!/usr/bin/env bash
# Setup Git Hooks for NEOLAND Development
#
# This script configures git to use custom hooks from .githooks/

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITHOOKS_DIR="$PROJECT_ROOT/.githooks"

echo "🔧 Setting up git hooks for NEOLAND..."

# Configure git to use .githooks directory
git config core.hooksPath "$GITHOOKS_DIR"

echo "✅ Git hooks configured!"
echo ""
echo "Installed hooks:"
ls -1 "$GITHOOKS_DIR"
echo ""
echo "The following checks will run before each commit:"
echo "  1. Format check (rustfmt)"
echo "  2. Lint check (clippy)"
echo "  3. Unit tests"
echo "  4. Compilation check"
echo ""
echo "To bypass hooks (not recommended):"
echo "  git commit --no-verify"
echo ""
echo "To disable hooks:"
echo "  git config --unset core.hooksPath"
echo ""
echo "🚀 Happy coding!"
