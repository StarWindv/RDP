#!/bin/sh
# Simple function parameter test

myfunc() {
    echo "arg1: $1"
    echo "arg2: $2"
    echo "total: $#"
}

myfunc hello world
