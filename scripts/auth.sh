#!/bin/bash
# ================================================================
# Novelist — Grant Accessibility Permissions for GUI Test Automation
#
# Required for: scripts/test-scroll-click.sh (cliclick + osascript)
#
# Usage:
#   sudo ./scripts/auth.sh
#
# What it does:
#   1. Grants Terminal.app accessibility access (for osascript keystrokes)
#   2. Grants cliclick accessibility access (for mouse simulation)
#   3. Installs cliclick via Homebrew if not present
#
# Notes:
#   - Requires sudo (writes to system TCC database)
#   - If SIP is enabled and TCC.db write fails, falls back to
#     tccutil reset which triggers a system prompt on next launch
#   - After running, restart Terminal for changes to take effect
# ================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

log()  { echo -e "${YELLOW}[auth]${NC} $1"; }
ok()   { echo -e "${GREEN}[ok]${NC} $1"; }
err()  { echo -e "${RED}[error]${NC} $1"; }

# Must be root
if [ "$EUID" -ne 0 ]; then
  echo "Usage: sudo $0"
  exit 1
fi

TCC_DB="/Library/Application Support/com.apple.TCC/TCC.db"

# ======================================================
# Step 1: Install cliclick if missing
# ======================================================
if ! command -v cliclick &>/dev/null; then
  log "Installing cliclick via Homebrew..."
  # Run brew as the real user (not root)
  REAL_USER=$(stat -f '%Su' /dev/console)
  sudo -u "$REAL_USER" brew install cliclick
  ok "cliclick installed"
else
  ok "cliclick already installed: $(which cliclick)"
fi

# ======================================================
# Step 2: Grant accessibility permissions
# ======================================================
grant_access() {
  local client=$1
  local client_type=$2  # 0 = bundle ID, 1 = path
  local label=$3

  log "Granting accessibility to ${label}..."

  # Try direct TCC.db write first
  if sqlite3 "$TCC_DB" \
    "INSERT OR REPLACE INTO access (service, client, client_type, auth_value, auth_reason, auth_version) VALUES ('kTCCServiceAccessibility', '${client}', ${client_type}, 2, 0, 1);" 2>/dev/null; then
    ok "  ${label} — granted via TCC.db"
    return 0
  fi

  # Fallback: tccutil reset (will prompt user on next launch)
  log "  TCC.db write failed (SIP?). Trying tccutil reset..."
  if [ "$client_type" -eq 0 ]; then
    tccutil reset Accessibility "$client" 2>/dev/null && \
      ok "  ${label} — reset via tccutil (system will prompt on next launch)" || \
      err "  ${label} — failed. Please grant manually in System Settings → Privacy → Accessibility"
  else
    err "  ${label} — cannot use tccutil for path-based apps. Grant manually in System Settings → Privacy → Accessibility"
  fi
}

# Terminal.app (bundle ID)
grant_access "com.apple.Terminal" 0 "Terminal.app"

# iTerm2 (if present)
if [ -d "/Applications/iTerm.app" ]; then
  grant_access "com.googlecode.iterm2" 0 "iTerm2"
fi

# Alacritty (if present)
if [ -d "/Applications/Alacritty.app" ]; then
  grant_access "org.alacritty" 0 "Alacritty"
fi

# WezTerm (if present)
if [ -d "/Applications/WezTerm.app" ]; then
  grant_access "com.github.wez.wezterm" 0 "WezTerm"
fi

# cliclick (path-based)
CLICLICK_PATH=$(which cliclick 2>/dev/null || echo "")
if [ -n "$CLICLICK_PATH" ]; then
  grant_access "$CLICLICK_PATH" 1 "cliclick"
fi

# ======================================================
# Step 3: Verify
# ======================================================
echo ""
log "Current accessibility grants:"
sqlite3 "$TCC_DB" \
  "SELECT client, CASE auth_value WHEN 2 THEN 'ALLOWED' WHEN 0 THEN 'DENIED' ELSE auth_value END FROM access WHERE service='kTCCServiceAccessibility';" 2>/dev/null | \
  while IFS='|' read -r client status; do
    echo "  ${status}  ${client}"
  done || log "(Could not read TCC.db — SIP may be blocking reads)"

echo ""
ok "Done. Restart your terminal, then run:"
echo "  ./scripts/test-scroll-click.sh"
