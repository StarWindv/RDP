#!/bin/sh
# Test function parameter passing

# Test 1: Simple parameter access
test_simple_params() {
    echo "Test 1: Simple parameter access"
    
    func1() {
        echo "Function received: arg1=$1, arg2=$2, arg3=$3"
        echo "Total args: $#"
        echo "All args (*): $*"
        echo "All args (@): $@"
    }
    
    func1 "hello" "world" "test"
}

# Test 2: $@ vs $*
test_at_vs_star() {
    echo "Test 2: \$@ vs \$*"
    
    func2() {
        echo "Using \$*:"
        for arg in $*; do
            echo "  - '$arg'"
        done
        
        echo "Using \$@:"
        for arg in "$@"; do
            echo "  - '$arg'"
        done
    }
    
    func2 "one two" "three" "four"
}

# Test 3: Shift command
test_shift() {
    echo "Test 3: shift command"
    
    func3() {
        echo "Before shift: \$1=$1, \$#=$#"
        shift
        echo "After shift: \$1=$1, \$#=$#"
        shift
        echo "After 2nd shift: \$1=$1, \$#=$#"
    }
    
    func3 "first" "second" "third"
}

# Run tests
test_simple_params
echo ""
test_at_vs_star
echo ""
test_shift
