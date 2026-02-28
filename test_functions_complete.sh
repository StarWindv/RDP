#!/bin/sh
# 自定义函数系统完整演示

# 定义函数1：简单函数
greet() {
    echo "Hello from greet function"
}

# 定义函数2：带参数的函数
add() {
    echo "函数名: $0"
    echo "参数个数: $#"
    echo "第1个参数: $1"
    echo "第2个参数: $2"
    echo "所有参数: $*"
}

# 定义函数3：使用局部变量和返回值
compute() {
    local result
    result=$((10 + 20))
    echo "计算结果: $result"
    return 42
}

# 定义函数4：递归函数示例
countdown() {
    if [ $1 -le 0 ]; then
        echo "完成!"
        return 0
    fi
    echo "$1"
    return 0
}

# ============ 测试 ============

echo "=== 测试1: 简单函数调用 ==="
greet

echo ""
echo "=== 测试2: 带参数的函数 ==="
add apple banana

echo ""
echo "=== 测试3: 返回值测试 ==="
compute
echo "返回状态码: $?"

echo ""
echo "=== 测试4: 递归/循环调用 ==="
countdown 3

echo ""
echo "=== 所有测试完成 ==="
