#!/bin/bash
cd /home/null/Desktop/Stripeify
echo "y" | timeout 300 ./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file test_3cards.txt \
  --output test_results_final.json
