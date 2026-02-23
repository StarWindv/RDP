#!/bin/bash

echo "=== Testing rs-dash-pro Basic Features ==="
echo ""

# Build the project
echo "Building rs-dash-pro..."
cargo build --release

echo ""
echo "=== Running Tests ==="
echo ""

tests_passed=0
tests_total=0

run_test() {
    local test_name="$1"
    local command="$2"
    local expected="$3"
    
    tests_total=$((tests_total + 1))
    
    echo "Test $tests_total: $test_name"
    echo "Command: $command"
    echo -n "Output: "
    
    output=$(./target/release/rs-dash-pro.exe -c "$command" 2>/dev/null)
    echo "$output"
    
    if [ "$output" = "$expected" ]; then
        echo "✓ PASSED"
        tests_passed=$((tests_passed + 1))
    else
        echo "✗ FAILED (Expected: '$expected')"
    fi
    echo ""
}

# Test 1: Simple echo
run_test "Simple echo" "echo hello" "hello"

# Test 2: Echo with multiple arguments
run_test "Echo with multiple arguments" "echo hello world" "hello world"

# Test 3: Multiple commands
run_test "Multiple commands" "echo first; echo second" "first
second"

# Test 4: Builtin pwd (just check it runs without error)
tests_total=$((tests_total + 1))
echo "Test $tests_total: Builtin pwd"
echo "Command: pwd"
output=$(./target/release/rs-dash-pro.exe -c "pwd" 2>/dev/null)
if [ $? -eq 0 ] && [ -n "$output" ]; then
    echo "✓ PASSED (Output: $output)"
    tests_passed=$((tests_passed + 1))
else
    echo "✗ FAILED"
fi
echo ""

# Test 5: Variable assignment
run_test "Variable assignment" "MYVAR=test; echo \$MYVAR" "test"

# Test 6: Logical AND
run_test "Logical AND" "true && echo success" "success"

# Test 7: Logical OR  
run_test "Logical OR" "false || echo failed" "failed"

# Test 8: External command (whoami)
tests_total=$((tests_total + 1))
echo "Test $tests_total: External command"
echo "Command: whoami"
output=$(./target/release/rs-dash-pro.exe -c "whoami" 2>/dev/null)
if [ $? -eq 0 ] && [ -n "$output" ]; then
    echo "✓ PASSED (Output: $output)"
    tests_passed=$((tests_passed + 1))
else
    echo "✗ FAILED"
fi
echo ""

# Test 9: Command with quotes
run_test "Command with quotes" "echo 'hello world'" "hello world"

# Test 10: Command with double quotes
run_test "Command with double quotes" 'echo "hello world"' "hello world"

echo "=== Test Summary ==="
echo "Passed: $tests_passed/$tests_total"
echo ""

if [ $tests_passed -eq $tests_total ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi