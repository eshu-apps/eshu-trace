#!/bin/bash
# ESHU Trace Demo Recording Script
# Shows time-travel debugging to find breaking package changes

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Install required tools
if ! command -v asciinema &> /dev/null; then
    echo -e "${YELLOW}Installing asciinema...${NC}"
    sudo pacman -S asciinema --noconfirm || sudo apt install asciinema -y
fi

if ! command -v expect &> /dev/null; then
    echo -e "${YELLOW}Installing expect...${NC}"
    sudo pacman -S expect --noconfirm || sudo apt install expect -y
fi

# Create demo script
cat > /tmp/trace-demo.exp << 'EOF'
#!/usr/bin/expect -f

set timeout -1
set send_human {0.05 0.1 0.5 0.01 0.2}

spawn asciinema rec --overwrite /tmp/trace-demo.cast

sleep 2

# === SCENE 1: The Problem ===
send -h "# ESHU Trace: Time Travel Debugging for Packages\r"
sleep 2
send -h "\r"
send -h "# Scenario: Your app broke after recent updates\r"
sleep 2
send -h "# Which package caused it? Let's find out!\r"
sleep 2

# === SCENE 2: Check Status ===
send -h "\r"
send -h "eshu-trace status\r"
sleep 4
send -h "\r"
send -h "# Shows current packages and recent changes\r"
sleep 2

# === SCENE 3: Binary Search Through History ===
send -h "\r\r"
send -h "# Start time-travel debugging\r"
sleep 1
send -h "eshu-trace bisect start\r"
sleep 3

# Simulate bisect process
send -h "# Is the current state good or bad?\r"
sleep 2
send -h "eshu-trace bisect bad\r"
sleep 2

send -h "\r"
send -h "# Testing middle point in history...\r"
sleep 3
send -h "# Run your test: ./my-app --test\r"
sleep 2
send -h "eshu-trace bisect good\r"
sleep 2

send -h "\r"
send -h "# Binary search continues...\r"
sleep 3
send -h "eshu-trace bisect bad\r"
sleep 2

send -h "\r"
send -h "# 🎯 FOUND IT! Package 'libfoo' v2.3.1 broke your app\r"
sleep 3

# === SCENE 4: Rollback ===
send -h "\r\r"
send -h "# Rollback to working version\r"
sleep 1
send -h "eshu-trace rollback libfoo 2.3.0\r"
sleep 3
send -h "# ✅ Rolled back successfully!\r"
sleep 2

# === SCENE 5: History View ===
send -h "\r\r"
send -h "# View complete package history\r"
sleep 1
send -h "eshu-trace history python\r"
sleep 4

# === FINALE ===
send -h "\r\r"
send -h "# ESHU Trace: Never guess which update broke your system again!\r"
sleep 2
send -h "# Get it: paru -S eshu-trace\r"
sleep 2
send -h "# Visit: https://eshu-apps.com\r"
sleep 2
send -h "\r"

send "\x04"
sleep 2

EOF

chmod +x /tmp/trace-demo.exp

echo -e "${GREEN}Starting ESHU Trace demo recording...${NC}"
/tmp/trace-demo.exp

if command -v agg &> /dev/null; then
    echo -e "${GREEN}Converting to GIF...${NC}"
    agg /tmp/trace-demo.cast /tmp/trace-demo.gif --speed 1.5
    echo -e "${GREEN}GIF created: /tmp/trace-demo.gif${NC}"
fi

echo -e "${GREEN}Demo complete!${NC}"
echo -e "${BLUE}Recording: /tmp/trace-demo.cast${NC}"
echo -e "${BLUE}View: asciinema play /tmp/trace-demo.cast${NC}"
