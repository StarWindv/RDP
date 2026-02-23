#!/bin/bash

echo "Testing rs-dash-pro basic functionality..."

# Test 1: Simple echo command
echo "Test 1: Simple echo command"
./target/debug/rs-dash-pro.exe -c "echo hello world"

# Test 2: Multiple commands
echo -e "\nTest 2: Multiple commands"
./target/debug/rs-dash-pro.exe -c "echo first; echo second"

# Test 3: Builtin commands
echo -e "\nTest 3: Builtin commands"
./target/debug/rs-dash-pro.exe -c "pwd"

# Test 4: Variable assignment
echo -e "\nTest 4: Variable assignment"
./target/debug/rs-dash-pro.exe -c "TEST=value; echo \$TEST"

# Test 5: External command (if available)
echo -e "\nTest 5: External command"
./target/debug/rs-dash-pro.exe -c "whoami"