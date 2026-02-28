#!/bin/sh
# 自定义函数 - 基础测试

# 标准POSIX函数定义方式1
myfunc() {
    echo "这是myfunc"
}

# 调用函数
myfunc

# 带参数的函数
show_args() {
    echo "第一个参数: $1"
    echo "第二个参数: $2"
}

show_args hello world

echo "完成"
