#!/bin/sh
# Simple return test

test_return() {
    return 42
}

test_return
echo Return_value_is: $?
