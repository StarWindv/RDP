#!/bin/sh
# Simple test script for rs-dash-pro

echo "Testing rs-dash-pro..."
echo "Current directory: $(pwd)"
echo "PATH: $PATH"

# Test basic echo
echo "Hello from rs-dash-pro"

# Test variable expansion
VAR="test variable"
echo "VAR=$VAR"

# Test exit status
true
echo "Exit status of true: $?"
false
echo "Exit status of false: $?"

# Test command substitution
echo "Current date: $(date)"

# Test simple arithmetic
echo "2 + 3 = $((2 + 3))"

echo "Test completed."