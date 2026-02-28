#!/bin/sh
# Simple local variable test - no parameter expansion

test_local_simple() {
    local X="local"
    echo X_local
}

Y="global"
echo Y_global
test_local_simple
echo Y_global_after
