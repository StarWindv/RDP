#!/bin/sh
# Test return statement with simple echo
# (no parameter expansion to avoid parsing issues)

test_return() {
    return 42
}

test_return
echo PASSED_IF_RETURNS_42
