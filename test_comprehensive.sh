#!/bin/bash

echo "=== Testing rs-dash-pro ==="
echo ""

# Build the project
echo "Building rs-dash-pro..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "=== Basic Command Tests ==="
echo ""

# Test 1: Simple echo command
echo "Test 1: Simple echo command"
./target/release/rs-dash-pro.exe -c "echo hello world"
echo "Expected: hello world"
echo ""

# Test 2: Multiple commands with semicolon
echo "Test 2: Multiple commands with semicolon"
./target/release/rs-dash-pro.exe -c "echo first; echo second"
echo "Expected: first (newline) second"
echo ""

# Test 3: Builtin pwd command
echo "Test 3: Builtin pwd command"
./target/release/rs-dash-pro.exe -c "pwd"
echo "Expected: current directory"
echo ""

# Test 4: Variable assignment and expansion
echo "Test 4: Variable assignment and expansion"
./target/release/rs-dash-pro.exe -c "TESTVAR=hello; echo \$TESTVAR"
echo "Expected: hello"
echo ""

# Test 5: Logical AND
echo "Test 5: Logical AND"
./target/release/rs-dash-pro.exe -c "true && echo success"
echo "Expected: success"
echo ""

# Test 6: Logical OR
echo "Test 6: Logical OR"
./target/release/rs-dash-pro.exe -c "false || echo failed"
echo "Expected: failed"
echo ""

# Test 7: Command with arguments
echo "Test 7: Command with arguments"
./target/release/rs-dash-pro.exe -c "echo arg1 arg2 arg3"
echo "Expected: arg1 arg2 arg3"
echo ""

# Test 8: External command (if available)
echo "Test 8: External command"
./target/release/rs-dash-pro.exe -c "whoami"
echo "Expected: current username"
echo ""

# Test 9: Pipeline (basic)
echo "Test 9: Pipeline (basic)"
./target/release/rs-dash-pro.exe -c "echo hello | cat"
echo "Expected: hello"
echo ""

# Test 10: Redirection (output)
echo "Test 10: Redirection (output)"
./target/release/rs-dash-pro.exe -c "echo test > test_output.txt"
cat test_output.txt
rm test_output.txt
echo "Expected: test"
echo ""

# Test 11: Compound commands
echo "Test 11: Compound commands"
./target/release/rs-dash-pro.exe -c "{ echo first; echo second; }"
echo "Expected: first (newline) second"
echo ""

# Test 12: If statement
echo "Test 12: If statement"
./target/release/ds-dash-pro.exe -c "if true; then echo if_works; fi"
echo "Expected: if_works"
echo ""

# Test 13: While loop
echo "Test 13: While loop"
./target/release/rs-dash-pro.exe -c "count=0; while [ \$count -lt 3 ]; do echo \$count; count=\$((count+1)); done"
echo "Expected: 0 1 2 (each on new line)"
echo ""

# Test 14: For loop
echo "Test 14: For loop"
./target/release/rs-dash-pro.exe -c "for i in 1 2 3; do echo \$i; done"
echo "Expected: 1 2 3 (each on new line)"
echo ""

echo "=== All tests completed ==="