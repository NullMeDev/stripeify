#!/bin/bash

echo "=========================================="
echo "Searching for Mady Gateway Configuration"
echo "=========================================="
echo ""

echo "1. Searching for other mady.py files..."
echo "----------------------------------------"
find ~/Desktop -name "mady*.py" -type f 2>/dev/null | while read file; do
    echo "Found: $file"
    # Check if it has a configured gateway
    if grep -q "ccfoundation" "$file" 2>/dev/null; then
        echo "  ✓ Contains 'ccfoundation' reference"
    fi
    if grep -q "YOUR_PAYMENT_API_URL" "$file" 2>/dev/null; then
        echo "  ⚠ Contains placeholder URL"
    else
        echo "  ✓ May have real gateway URL configured"
    fi
    echo ""
done

echo ""
echo "2. Searching for configuration files..."
echo "----------------------------------------"
find ~/Desktop -type f \( -name "*config*.json" -o -name "*config*.txt" -o -name "*.conf" \) 2>/dev/null | grep -i mady

echo ""
echo "3. Checking current directory for backups..."
echo "----------------------------------------"
ls -lah /home/null/Desktop/SKy/MadyOriginal/ | grep -E "\.py|\.json|\.bak|\.old"

echo ""
echo "4. Searching for Stripe API references..."
echo "----------------------------------------"
grep -r "stripe" ~/Desktop/SKy/ 2>/dev/null | grep -v ".git" | head -10

echo ""
echo "5. Searching for ccfoundation references..."
echo "----------------------------------------"
grep -r "ccfoundation" ~/Desktop/SKy/ 2>/dev/null | grep -v ".git" | head -10

echo ""
echo "6. Checking git history (if available)..."
echo "----------------------------------------"
cd /home/null/Desktop/SKy/MadyOriginal
if [ -d .git ]; then
    echo "Git repository found!"
    echo ""
    echo "Recent commits:"
    git log --oneline -10 2>/dev/null
    echo ""
    echo "Files changed in recent commits:"
    git log --name-only --oneline -5 2>/dev/null
else
    echo "Not a git repository"
fi

echo ""
echo "=========================================="
echo "Search Complete"
echo "=========================================="
