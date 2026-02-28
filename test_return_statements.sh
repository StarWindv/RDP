#!/bin/sh
# Test return statements in functions

test_return_zero() {
    return 0
}

test_return_nonzero() {
    return 42
}

test_return_value() {
    true
    return
}

# Test 1
test_return_zero
if [ $? -eq 0 ]; then
    echo "Test 1 passed: return 0"
else
    echo "Test 1 failed: return 0 got $?"
fi

# Test 2
test_return_nonzero
if [ $? -eq 42 ]; then
    echo "Test 2 passed: return 42"
else
    echo "Test 2 failed: return 42 got $?"
fi

# Test 3
test_return_value
if [ $? -eq 0 ]; then
    echo "Test 3 passed: return (no arg)"
else
    echo "Test 3 failed: return (no arg) got $?"
fi
