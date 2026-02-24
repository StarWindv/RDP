#!/bin/bash

echo "=== Testing POSIX Special Builtins ==="
echo

# Test : (colon) - should always succeed
echo "Testing ':' (colon):"
:
echo "Exit status: $?"
echo

# Test . (dot) - source a file
echo "Testing '.' (dot) - creating test script:"
cat > test_source.sh << 'EOF'
echo "Sourced file executed"
MY_VAR="from sourced file"
EOF
. ./test_source.sh
echo "MY_VAR = $MY_VAR"
echo "Exit status: $?"
echo

# Test break/continue - in a loop context
echo "Testing break/continue (context needed):"
for i in 1 2 3 4 5; do
    if [ $i -eq 3 ]; then
        break
    fi
    echo "Loop iteration: $i"
done
echo "Exit status: $?"
echo

# Test eval
echo "Testing eval:"
eval "echo 'Hello from eval'"
echo "Exit status: $?"
echo

# Test exec - we'll use a simple command that won't replace shell
echo "Testing exec (simple command):"
exec echo "Hello from exec"
echo "This should not print if exec worked"
echo

# Test export
echo "Testing export:"
export TEST_VAR="exported value"
echo "TEST_VAR = $TEST_VAR"
env | grep TEST_VAR
echo "Exit status: $?"
echo

# Test readonly
echo "Testing readonly:"
readonly READONLY_VAR="readonly value"
echo "READONLY_VAR = $READONLY_VAR"
READONLY_VAR="try to change" 2>/dev/null && echo "ERROR: Should not be able to change"
echo "Exit status: $?"
echo

# Test set
echo "Testing set options:"
set -e  # errexit
echo "set -e enabled"
false && echo "This should not print"
echo "Exit status: $?"
set +e
echo "set +e disabled"
false && echo "This should print"
echo "Exit status: $?"
echo

# Test shift
echo "Testing shift:"
set -- arg1 arg2 arg3 arg4
echo "Original: \$1=$1, \$2=$2, \$3=$3, \$4=$4"
shift
echo "After shift: \$1=$1, \$2=$2, \$3=$3"
shift 2
echo "After shift 2: \$1=$1"
echo "Exit status: $?"
echo

# Test times
echo "Testing times:"
times
echo "Exit status: $?"
echo

# Test trap
echo "Testing trap:"
trap 'echo "SIGINT received"' INT
echo "Trap set for SIGINT"
# We won't actually send a signal in test
echo "Exit status: $?"
echo

# Test unset
echo "Testing unset:"
UNSET_VAR="to be unset"
echo "Before unset: UNSET_VAR = $UNSET_VAR"
unset UNSET_VAR
echo "After unset: UNSET_VAR = ${UNSET_VAR:-unset}"
echo "Exit status: $?"
echo

# Clean up
rm -f test_source.sh

echo "=== Builtin tests completed ==="