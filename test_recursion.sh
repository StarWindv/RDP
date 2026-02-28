#!/bin/sh
# Test recursive function

factorial() {
    if [ $1 -le 1 ]; then
        return 1
    else
        N=$1
        PREV=$((N-1))
        factorial $PREV
        return N
    fi
}

factorial 5
echo Recursion_test_completed
