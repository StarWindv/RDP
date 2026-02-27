#!/bin/bash
# Test script for break and continue statements

# Test 1: Break in a for loop
echo "Test 1: Break in for loop"
for i in 1 2 3 4 5; do
    if [ "$i" -eq 3 ]; then
        break
    fi
    echo "  Loop iteration: $i"
done
echo "  After loop"

# Test 2: Continue in a for loop
echo ""
echo "Test 2: Continue in for loop"
for i in 1 2 3 4 5; do
    if [ "$i" -eq 3 ]; then
        continue
    fi
    echo "  Loop iteration: $i"
done
echo "  After loop"

# Test 3: Break in a while loop
echo ""
echo "Test 3: Break in while loop"
i=1
while [ "$i" -le 5 ]; do
    if [ "$i" -eq 3 ]; then
        break
    fi
    echo "  Loop iteration: $i"
    i=$((i + 1))
done
echo "  After loop"

# Test 4: Continue in a while loop
echo ""
echo "Test 4: Continue in while loop"
i=1
while [ "$i" -le 5 ]; do
    if [ "$i" -eq 3 ]; then
        i=$((i + 1))
        continue
    fi
    echo "  Loop iteration: $i"
    i=$((i + 1))
done
echo "  After loop"

# Test 5: Function with return
echo ""
echo "Test 5: Function with return"
test_return() {
    echo "  Inside function"
    return 42
}
test_return
echo "  Return value: $?"

echo ""
echo "All tests completed"
