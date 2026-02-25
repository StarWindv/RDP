#!/bin/sh
# Test script for rs-dash-pro

echo "Testing if statement..."
if true; then
    echo "if statement works"
else
    echo "if statement failed"
fi

echo "Testing for loop..."
for i in 1 2 3; do
    echo "for loop iteration $i"
done

echo "Testing while loop..."
counter=0
while [ $counter -lt 3 ]; do
    echo "while loop iteration $counter"
    counter=$((counter + 1))
done

echo "Testing until loop..."
counter=0
until [ $counter -ge 3 ]; do
    echo "until loop iteration $counter"
    counter=$((counter + 1))
done

echo "Testing break and continue..."
for i in 1 2 3 4 5; do
    if [ $i -eq 3 ]; then
        continue
    fi
    if [ $i -eq 5 ]; then
        break
    fi
    echo "break/continue test: $i"
done

echo "All tests completed."