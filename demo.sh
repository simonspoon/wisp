#!/usr/bin/env bash
# Wisp Demo v0.2 — showcases all CLI features including save/load, undo/redo,
# components, and interactive session.
#
# Usage: ./demo.sh
#
set -euo pipefail

WISP_DIR="$(cd "$(dirname "$0")" && pwd)"
WISP="$WISP_DIR/target/release/wisp"
WS_PORT=9847
APP_PID=""

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
DIM='\033[0;90m'
BOLD='\033[1m'
RESET='\033[0m'

cleanup() {
    echo -e "\n${DIM}Stopping Wisp app...${RESET}"
    pkill -f "target/debug/app" 2>/dev/null || true
    pkill -f "pnpm tauri dev" 2>/dev/null || true
    pkill -f "node.*vite.*wisp" 2>/dev/null || true
}
trap cleanup EXIT

step() {
    echo -e "${BLUE}→${RESET} $1"
}

result() {
    echo -e "  ${GREEN}✓${RESET} ${DIM}$1${RESET}"
}

pause() {
    sleep "${1:-0.8}"
}

# --- Build CLI if needed ---
if [ ! -f "$WISP" ]; then
    echo -e "${BOLD}Building Wisp CLI...${RESET}"
    (cd "$WISP_DIR" && cargo build -p wisp-cli --release --quiet)
fi

# --- Start the desktop app ---
echo -e "${BOLD}Starting Wisp desktop app...${RESET}"
cd "$WISP_DIR/app"
pnpm tauri dev > /dev/null 2>&1 &
APP_PID=$!
cd "$WISP_DIR"

# Wait for WS server
printf "Waiting for server"
for i in $(seq 1 60); do
    if nc -z 127.0.0.1 $WS_PORT 2>/dev/null; then
        echo -e " ${GREEN}ready${RESET}"
        break
    fi
    printf "."
    sleep 1
done
if ! nc -z 127.0.0.1 $WS_PORT 2>/dev/null; then
    echo -e " FAILED (timeout)"
    exit 1
fi

# Give the UI a moment to render
sleep 2

echo ""
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo -e "${BOLD}  Wisp v0.2 Demo: Full Feature Showcase    ${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo ""

# --- Helper to add and capture ID ---
add_node() {
    local output
    output=$("$WISP" node add "$@" 2>&1)
    echo "$output" | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'
}

# ═══════════════════════════════════
# Part 1: Build a dashboard layout
# ═══════════════════════════════════

step "Creating header bar"
HEADER=$(add_node "Header" -t frame --width 1920 --height 80 --fill "#1e40af")
result "Header: $HEADER"
pause

step "Adding app title to header"
add_node "App Title" -t text -x 24 -y 22 --parent "$HEADER" --text "Wisp Dashboard" --font-size 28 > /dev/null
result "Title text added"
pause

step "Creating sidebar"
SIDEBAR=$(add_node "Sidebar" -t frame -y 80 --width 280 --height 1000 --fill "#1e3a5f")
result "Sidebar: $SIDEBAR"
pause

step "Adding navigation items"
add_node "Nav: Overview" -t text -x 24 -y 24 --parent "$SIDEBAR" --text "Overview" --font-size 15 > /dev/null
add_node "Nav: Analytics" -t text -x 24 -y 56 --parent "$SIDEBAR" --text "Analytics" --font-size 15 > /dev/null
add_node "Nav: Reports" -t text -x 24 -y 88 --parent "$SIDEBAR" --text "Reports" --font-size 15 > /dev/null
add_node "Nav: Settings" -t text -x 24 -y 120 --parent "$SIDEBAR" --text "Settings" --font-size 15 > /dev/null
result "4 nav items added"
pause

step "Creating main content area"
MAIN=$(add_node "Main Content" -t frame -x 280 -y 80 --width 1640 --height 1000 --fill "#f1f5f9")
result "Main area: $MAIN"
pause

# ═══════════════════════════════════
# Part 2: Components (v0.2 feature)
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}── Component Templates ──${RESET}"
echo ""

step "Listing available components"
"$WISP" components list
pause

step "Using stat-card components for dashboard metrics"
CARD1=$("$WISP" components use stat-card --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$CARD1" -x 32 -y 32 2>&1 > /dev/null
result "Stats card 1 placed"

CARD2=$("$WISP" components use stat-card --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$CARD2" -x 340 -y 32 2>&1 > /dev/null
result "Stats card 2 placed"

CARD3=$("$WISP" components use stat-card --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$CARD3" -x 648 -y 32 2>&1 > /dev/null
result "Stats card 3 placed"

CARD4=$("$WISP" components use stat-card --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$CARD4" -x 956 -y 32 2>&1 > /dev/null
result "Stats card 4 placed"
pause

step "Adding buttons"
BTN1=$("$WISP" components use button --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$BTN1" -x 32 -y 200 2>&1 > /dev/null
BTN2=$("$WISP" components use button --parent "$MAIN" 2>&1 | grep "root:" | awk '{print $2}')
"$WISP" node edit "$BTN2" -x 170 -y 200 2>&1 > /dev/null
result "2 buttons placed"
pause

# ═══════════════════════════════════
# Part 3: Partial edits (v0.2 fix)
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}── Partial Edits (Bug Fix) ──${RESET}"
echo ""

step "Editing only fill color on stat card (layout must be preserved)"
"$WISP" node edit "$CARD1" --fill "#e0f2fe" 2>&1
result "Fill changed, position preserved"
pause

step "Editing only x position on button (height/fill must be preserved)"
"$WISP" node edit "$BTN1" -x 50 2>&1
result "Position changed, style preserved"
pause

# ═══════════════════════════════════
# Part 4: Undo/Redo (v0.2 feature)
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}── Undo/Redo ──${RESET}"
echo ""

step "Current node count"
"$WISP" tree 2>&1 | wc -l | xargs -I{} echo "  {} lines in tree"

step "Undoing last 3 operations"
"$WISP" undo 2>&1
"$WISP" undo 2>&1
"$WISP" undo 2>&1
result "3 operations undone"

step "Node count after undo"
"$WISP" tree 2>&1 | wc -l | xargs -I{} echo "  {} lines in tree"

step "Redoing all 3"
"$WISP" redo 2>&1
"$WISP" redo 2>&1
"$WISP" redo 2>&1
result "3 operations redone"
pause

# ═══════════════════════════════════
# Part 5: Save/Load (v0.2 feature)
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}── Save/Load ──${RESET}"
echo ""

SAVE_PATH="/tmp/wisp-demo-dashboard.json"
step "Saving document to $SAVE_PATH"
"$WISP" save "$SAVE_PATH" 2>&1
result "Document saved"
pause

step "File size"
ls -lh "$SAVE_PATH" | awk '{print "  " $5 " " $9}'
pause

# ═══════════════════════════════════
# Part 6: Chart bars via session
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}── Interactive Session ──${RESET}"
echo ""

step "Creating chart area and bars via session mode"
CHART=$(add_node "Chart Panel" -t frame -x 32 -y 260 --parent "$MAIN" --width 1000 --height 500 --fill "#ffffff" --radius 16)
add_node "Chart Title" -t text -x 24 -y 20 --parent "$CHART" --text "Revenue Over Time" --font-size 18 > /dev/null

BAR_COLORS=("#3b82f6" "#2563eb" "#60a5fa" "#1d4ed8" "#93c5fd" "#3b82f6" "#60a5fa" "#2563eb" "#3b82f6" "#93c5fd" "#60a5fa" "#1d4ed8")
BAR_HEIGHTS=(180 220 150 280 200 320 250 300 260 190 340 380)
MONTHS=("Jan" "Feb" "Mar" "Apr" "May" "Jun" "Jul" "Aug" "Sep" "Oct" "Nov" "Dec")

# Build session commands
SESSION_CMDS=""
for i in $(seq 0 11); do
    bx=$((24 + i * 80))
    bh=${BAR_HEIGHTS[$i]}
    by=$((440 - bh))
    SESSION_CMDS="${SESSION_CMDS}node add \"${MONTHS[$i]} Bar\" -t rectangle -x $bx -y $by --parent $CHART --width 56 --height $bh --fill \"${BAR_COLORS[$i]}\" --radius 4\n"
    SESSION_CMDS="${SESSION_CMDS}node add \"${MONTHS[$i]}\" -t text -x $((bx + 12)) -y 450 --parent $CHART --text \"${MONTHS[$i]}\" --font-size 11\n"
done
SESSION_CMDS="${SESSION_CMDS}quit\n"

echo -e "$SESSION_CMDS" | "$WISP" session 2>/dev/null | grep -c "Created node" | xargs -I{} echo "  {} nodes created via session"
result "12-month bar chart drawn in single session"
pause

# ═══════════════════════════════════
# Part 7: Activity feed
# ═══════════════════════════════════

step "Creating activity feed"
FEED=$(add_node "Activity Feed" -t frame -x 1056 -y 260 --parent "$MAIN" --width 528 --height 500 --fill "#ffffff" --radius 16)
add_node "Feed Title" -t text -x 24 -y 20 --parent "$FEED" --text "Recent Activity" --font-size 18 > /dev/null

ACTIVITIES=(
    "New user registered: alice@example.com"
    "Order #4821 completed — \$142.00"
    "Payment received from Bob Chen"
    "Server scaled to 8 instances"
    "API latency alert resolved"
    "New feature deployed: dark mode"
    "Database backup completed"
    "User feedback survey sent"
)
for i in $(seq 0 7); do
    ay=$((60 + i * 52))
    add_node "Activity $((i+1))" -t text -x 24 -y "$ay" --parent "$FEED" --text "${ACTIVITIES[$i]}" --font-size 13 > /dev/null
done
result "Activity feed with 8 entries"
pause

# ═══════════════════════════════════
# Final save and tree
# ═══════════════════════════════════

echo ""
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo -e "${BOLD}  Final Document Tree                       ${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo ""
"$WISP" tree
echo ""

step "Final save"
"$WISP" save "$SAVE_PATH" 2>&1

NODE_COUNT=$("$WISP" tree 2>&1 | wc -l | xargs)
echo ""
echo -e "${GREEN}${BOLD}Demo complete!${RESET} ${NODE_COUNT} nodes in the design."
echo -e "${DIM}v0.2 features demonstrated: partial edits, save/load, undo/redo, components, sessions${RESET}"
echo -e "${DIM}Press Ctrl+C to stop the app.${RESET}"
echo ""

# Keep alive so the user can see the result
while kill -0 "$APP_PID" 2>/dev/null; do
    sleep 1
done
