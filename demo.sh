#!/usr/bin/env bash
# Wisp Demo — builds a dashboard design using only CLI commands
# The desktop app shows the design appearing in real-time.
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
echo -e "${BOLD}  Wisp Demo: Building a Dashboard via CLI  ${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo ""

# --- Helper to add and capture ID ---
add_node() {
    local output
    output=$("$WISP" node add "$@" 2>&1)
    echo "$output" | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'
}

# === Step 1: Header bar ===
step "Creating header bar"
HEADER=$(add_node "Header" -t frame --width 1920 --height 80 --fill "#1e40af")
result "Header: $HEADER"
pause

# === Step 2: Header text ===
step "Adding app title to header"
add_node "App Title" -t text -x 24 -y 22 --parent "$HEADER" --text "Wisp Dashboard" --font-size 28 > /dev/null
result "Title text added"
pause

# === Step 3: Sidebar ===
step "Creating sidebar"
SIDEBAR=$(add_node "Sidebar" -t frame -y 80 --width 280 --height 1000 --fill "#1e3a5f")
result "Sidebar: $SIDEBAR"
pause

# === Step 4: Sidebar nav items ===
step "Adding navigation items"
add_node "Nav: Overview" -t text -x 24 -y 24 --parent "$SIDEBAR" --text "Overview" --font-size 15 > /dev/null
add_node "Nav: Analytics" -t text -x 24 -y 56 --parent "$SIDEBAR" --text "Analytics" --font-size 15 > /dev/null
add_node "Nav: Reports" -t text -x 24 -y 88 --parent "$SIDEBAR" --text "Reports" --font-size 15 > /dev/null
add_node "Nav: Settings" -t text -x 24 -y 120 --parent "$SIDEBAR" --text "Settings" --font-size 15 > /dev/null
result "4 nav items added"
pause

# === Step 5: Main content area ===
step "Creating main content area"
MAIN=$(add_node "Main Content" -t frame -x 280 -y 80 --width 1640 --height 1000 --fill "#f1f5f9")
result "Main area: $MAIN"
pause

# === Step 6: Stats cards row ===
step "Adding stats cards"
CARD1=$(add_node "Users Card" -t frame -x 32 -y 32 --parent "$MAIN" --width 370 --height 160 --fill "#ffffff" --radius 16)
CARD2=$(add_node "Revenue Card" -t frame -x 426 -y 32 --parent "$MAIN" --width 370 --height 160 --fill "#ffffff" --radius 16)
CARD3=$(add_node "Orders Card" -t frame -x 820 -y 32 --parent "$MAIN" --width 370 --height 160 --fill "#ffffff" --radius 16)
CARD4=$(add_node "Growth Card" -t frame -x 1214 -y 32 --parent "$MAIN" --width 370 --height 160 --fill "#ffffff" --radius 16)
result "4 stat cards created"
pause

# === Step 7: Card labels ===
step "Adding card content"
add_node "Label" -t text -x 24 -y 20 --parent "$CARD1" --text "Total Users" --font-size 13 > /dev/null
add_node "Value" -t text -x 24 -y 60 --parent "$CARD1" --text "12,847" --font-size 36 > /dev/null
add_node "Change" -t text -x 24 -y 115 --parent "$CARD1" --text "+14.2% from last month" --font-size 12 > /dev/null

add_node "Label" -t text -x 24 -y 20 --parent "$CARD2" --text "Revenue" --font-size 13 > /dev/null
add_node "Value" -t text -x 24 -y 60 --parent "$CARD2" --text '$84,254' --font-size 36 > /dev/null
add_node "Change" -t text -x 24 -y 115 --parent "$CARD2" --text "+8.1% from last month" --font-size 12 > /dev/null

add_node "Label" -t text -x 24 -y 20 --parent "$CARD3" --text "Orders" --font-size 13 > /dev/null
add_node "Value" -t text -x 24 -y 60 --parent "$CARD3" --text "3,621" --font-size 36 > /dev/null
add_node "Change" -t text -x 24 -y 115 --parent "$CARD3" --text "+23.5% from last month" --font-size 12 > /dev/null

add_node "Label" -t text -x 24 -y 20 --parent "$CARD4" --text "Growth" --font-size 13 > /dev/null
add_node "Value" -t text -x 24 -y 60 --parent "$CARD4" --text "18.2%" --font-size 36 > /dev/null
add_node "Change" -t text -x 24 -y 115 --parent "$CARD4" --text "+2.4% from last quarter" --font-size 12 > /dev/null
result "Stats content added to all 4 cards"
pause

# === Step 8: Chart area ===
step "Creating chart section"
CHART=$(add_node "Chart Panel" -t frame -x 32 -y 224 --parent "$MAIN" --width 1000 --height 500 --fill "#ffffff" --radius 16)
add_node "Chart Title" -t text -x 24 -y 20 --parent "$CHART" --text "Revenue Over Time" --font-size 18 > /dev/null
result "Chart panel with title"
pause

# === Step 9: Chart bars (simulated bar chart) ===
step "Drawing chart bars"
BAR_COLORS=("#3b82f6" "#2563eb" "#60a5fa" "#1d4ed8" "#93c5fd" "#3b82f6" "#60a5fa" "#2563eb" "#3b82f6" "#93c5fd" "#60a5fa" "#1d4ed8")
BAR_HEIGHTS=(180 220 150 280 200 320 250 300 260 190 340 380)
MONTHS=("Jan" "Feb" "Mar" "Apr" "May" "Jun" "Jul" "Aug" "Sep" "Oct" "Nov" "Dec")
for i in $(seq 0 11); do
    bx=$((24 + i * 80))
    bh=${BAR_HEIGHTS[$i]}
    by=$((440 - bh))
    add_node "${MONTHS[$i]} Bar" -t rectangle -x "$bx" -y "$by" --parent "$CHART" --width 56 --height "$bh" --fill "${BAR_COLORS[$i]}" --radius 4 > /dev/null
    add_node "${MONTHS[$i]}" -t text -x "$((bx + 12))" -y 450 --parent "$CHART" --text "${MONTHS[$i]}" --font-size 11 > /dev/null
done
result "12-month bar chart drawn"
pause

# === Step 10: Activity feed ===
step "Creating activity feed"
FEED=$(add_node "Activity Feed" -t frame -x 1056 -y 224 --parent "$MAIN" --width 528 --height 500 --fill "#ffffff" --radius 16)
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

# === Final tree ===
echo ""
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo -e "${BOLD}  Final Document Tree                       ${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
echo ""
"$WISP" tree
echo ""

echo -e "${GREEN}${BOLD}Demo complete!${RESET} The dashboard is visible in the Wisp app."
echo -e "${DIM}Press Ctrl+C to stop the app.${RESET}"
echo ""

# Keep alive so the user can see the result
while kill -0 "$APP_PID" 2>/dev/null; do
    sleep 1
done
