#!/bin/sh
# Test script for pipeline functionality

echo "=== Testing basic pipeline ==="
echo "hello" | cat
echo "Status: $?"

echo "=== Testing multi-pipeline ==="
echo "test1" | cat | grep "test"
echo "Status: $?"

echo "=== Testing output redirection ==="
echo "test output" > test_output.txt
cat test_output.txt
rm test_output.txt

echo "=== Testing input redirection ==="
echo "test input" > test_input.txt
cat < test_input.txt
rm test_input.txt

echo "=== Testing append redirection ==="
echo "line1" > test_append.txt
echo "line2" >> test_append.txt
cat test_append.txt
rm test_append.txt

echo "=== Testing error redirection ==="
ls /nonexistent 2> /dev/null
echo "Status: $?"

echo "=== Testing here document ==="
cat << EOF
Line 1
Line 2
Line 3
EOF

echo "=== Testing combined redirection ==="
echo "stdout" > combined.txt 2>&1
cat combined.txt
rm combined.txt

echo "=== Testing fd duplication ==="
echo "test" 2>&1
echo "Status: $?"

echo "=== All tests completed ==="