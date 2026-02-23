#!/bin/bash
# Test script for rs-dash-pro

echo "Testing basic commands..."
echo "hello world"

echo "Testing variable assignment..."
MYVAR="test value"
echo "Variable value: $MYVAR"

echo "Testing simple pipeline..."
echo "line1\nline2\nline3" | grep "line2"

echo "Testing logical operators..."
true && echo "AND operator works"
false || echo "OR operator works"

echo "Testing if statement..."
if true; then
    echo "If statement works"
fi

echo "Testing while loop..."
counter=0
while [ $counter -lt 3 ]; do
    echo "Counter: $counter"
    counter=$((counter + 1))
done

echo "Testing for loop..."
for i in 1 2 3; do
    echo "Number: $i"
done

echo "All tests completed!"