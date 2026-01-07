#!/bin/bash
# Helper script to properly start ChromeDriver

echo "🔧 ChromeDriver Manager"
echo "======================="
echo ""

# Check if ChromeDriver is already running
if pgrep -f chromedriver > /dev/null; then
    echo "⚠️  ChromeDriver is already running"
    echo ""
    echo "Options:"
    echo "  1) Kill existing and restart"
    echo "  2) Keep existing (exit)"
    echo ""
    read -p "Choose (1/2): " choice
    
    if [ "$choice" = "1" ]; then
        echo "🔪 Killing existing ChromeDriver..."
        pkill -9 -f chromedriver
        sleep 2
        
        # Verify it's dead
        if pgrep -f chromedriver > /dev/null; then
            echo "❌ Failed to kill ChromeDriver. Please run: sudo pkill -9 chromedriver"
            exit 1
        fi
        echo "✓ ChromeDriver stopped"
    else
        echo "✓ Keeping existing ChromeDriver"
        exit 0
    fi
fi

echo ""
echo "🚀 Starting ChromeDriver on port 9515..."
echo ""

# Start ChromeDriver in background
chromedriver --port=9515 > /dev/null 2>&1 &
CHROMEDRIVER_PID=$!

# Wait a moment for it to start
sleep 2

# Verify it started
if ps -p $CHROMEDRIVER_PID > /dev/null 2>&1; then
    echo "✅ ChromeDriver started successfully!"
    echo "   PID: $CHROMEDRIVER_PID"
    echo "   Port: 9515"
    echo ""
    echo "To stop ChromeDriver later, run:"
    echo "   pkill -f chromedriver"
    echo ""
else
    echo "❌ Failed to start ChromeDriver"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check if port 9515 is in use: lsof -i :9515"
    echo "  2. Try killing any existing process: pkill -9 chromedriver"
    echo "  3. Check if chromedriver is installed: which chromedriver"
    exit 1
fi
