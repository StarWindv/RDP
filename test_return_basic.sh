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

# Test 1: Simple return
test_return_zero
echo After_test_return_zero

# Test 2: Return with value
test_return_nonzero
echo After_test_return_nonzero

# Test 3: Return with no arg
test_return_value
echo After_test_return_value
