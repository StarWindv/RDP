#!/bin/sh
# Check if myfunc is being called

myfunc() {
    echo hello
}

echo before
myfunc
echo after
