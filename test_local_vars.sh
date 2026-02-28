#!/bin/sh
# Test local variables in functions

# Global variable
GLOBAL="global_value"

test_local() {
    # Create a local variable with same name as global
    local GLOBAL="local_value"
    
    # Create a new local variable
    local LOCAL_VAR="inside_function"
    
    echo "Inside function:"
    echo "  GLOBAL=$GLOBAL"
    echo "  LOCAL_VAR=$LOCAL_VAR"
}

echo "Before function:"
echo "  GLOBAL=$GLOBAL"

test_local

echo "After function:"
echo "  GLOBAL=$GLOBAL"
# LOCAL_VAR should not exist outside function
echo "  LOCAL_VAR=$LOCAL_VAR"
