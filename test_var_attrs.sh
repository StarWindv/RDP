#!/bin/sh
# Test variable attributes

X=original
export X
set Y new_val
shift
unset X
echo done
