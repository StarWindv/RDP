# POSIX Shell SSA IR 设计文档

## 设计目标
1. 完整的POSIX Shell功能支持
2. 静态单赋值形式，便于优化
3. 显式控制流和数据流
4. 支持所有Shell语法结构

## SSA IR 核心概念

### 基本块 (BasicBlock)
- 线性指令序列
- 单一入口，单一出口
- 结尾是终止指令（分支、返回等）

### 值 (Value)
- 所有操作产生值
- 每个值有唯一名称（%1, %2, ...）
- 类型：字符串、整数、布尔、文件描述符、进程ID等

### 指令 (Instruction)
- 操作符 + 操作数 → 结果值
- 所有副作用显式表示

## IR 指令设计

### 1. 控制流指令
```
jump %block           // 无条件跳转
branch %cond %true_block %false_block  // 条件跳转
return %status        // 返回退出状态
```

### 2. 变量操作
```
alloc_var %name       // 分配变量
store %var %value     // 存储值到变量
load %var → %result   // 从变量加载值
```

### 3. 命令执行
```
call_builtin %name %args... → %status
call_external %cmd %args... → %status
```

### 4. 管道和重定向
```
create_pipe → %read_fd %write_fd
dup_fd %old_fd %new_fd
close_fd %fd
redirect %fd %target %mode   // mode: read, write, append, etc.
```

### 5. 进程操作
```
fork → %pid
exec %pid %cmd %args...
wait %pid → %status
```

### 6. 字符串操作
```
concat %str1 %str2 → %result
substr %str %start %len → %result
match %pattern %str → %bool
```

### 7. 算术运算
```
add %a %b → %result
sub %a %b → %result
mul %a %b → %result
div %a %b → %result
```

### 8. 逻辑运算
```
and %a %b → %result
or %a %b → %result
not %a → %result
cmp %a %b %op → %result   // op: eq, ne, lt, le, gt, ge
```

## SSA IR 示例

### 简单命令
```
%1 = alloc_var "VAR"
%2 = store %1 "value"
%3 = call_external "echo" "hello"
%4 = return %3
```

### 管道
```
%1 = create_pipe
%2 = fork
branch %2 0 %child %parent

.child:
%3 = dup_fd %1.write_fd 1
%4 = close_fd %1.read_fd
%5 = call_external "ls" "-l"
%6 = exit %5

.parent:
%7 = dup_fd %1.read_fd 0
%8 = close_fd %1.write_fd
%9 = call_external "grep" "pattern"
%10 = wait %2 → %status
%11 = return %9
```

### If语句
```
%1 = call_external "test" "-f" "file.txt"
%2 = cmp %1 0 eq
branch %2 %then %else

.then:
%3 = call_external "echo" "File exists"
jump %endif

.else:
%4 = call_external "echo" "File not found"
jump %endif

.endif:
%5 = phi [%then: %3] [%else: %4]
%6 = return 0
```

## 实现计划

### Phase 1: 基础IR结构
1. 定义Value、BasicBlock、Function
2. 实现核心指令集
3. 创建IR构建器

### Phase 2: 增强Lexer
1. 支持所有POSIX token
2. 处理heredoc、process substitution等

### Phase 3: 增强Parser
1. 完整语法分析
2. 生成AST

### Phase 4: IR生成
1. AST转SSA IR
2. 处理控制流
3. 插入phi节点

### Phase 5: 执行器
1. 解释执行SSA IR
2. 实现所有指令语义

### Phase 6: 优化器
1. 死代码消除
2. 常量传播
3. 公共子表达式消除