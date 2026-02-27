#!/bin/bash
# Simple test for break parsing

for i in 1 2 3; do
    if [ "$i" -eq 2 ]; then
        break
    fi
    echo "$i"
done
