#!/bin/sh
# Test simple recursion

count() {
    if [ $1 -eq 0 ]; then
        return 0
    else
        true
    fi
}

count 0
echo Recursion_simple_test
