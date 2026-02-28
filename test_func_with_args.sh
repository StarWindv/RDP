#!/bin/sh
# Function with arguments but no parameter expansion

myfunc() {
    echo got args
}

echo before
myfunc hello world
echo after
