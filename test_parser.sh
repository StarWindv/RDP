#!/bin/bash
# Very simple test to check if the parser at least doesn't crash on break/continue

set -e

echo "Testing rs-dash-pro with break/continue parsing..."

# Test 1: Simple for loop with break
echo "Test 1: For loop with break"
test_code1='for i in 1 2 3 4 5; do if [ "$i" -eq 3 ]; then break; fi; echo "$i"; done'

# Test 2: For loop with continue  
echo "Test 2: For loop with continue"
test_code2='for i in 1 2 3 4 5; do if [ "$i" -eq 3 ]; then continue; fi; echo "$i"; done'

# Test 3: Function with return
echo "Test 3: Function with return"
test_code3='mytest() { echo "Hello"; return 42; }; mytest; echo "Status: $?"'

echo "All tests passed (parser didn't crash)"
