# rs-dash-pro 实现计划：从现代架构走向POSIX兼容

## 执行总结 (2026-02-28 重构)

本计划基于对rs-dash-pro当前状态（SSA架构、部分POSIX内置命令实现）和POSIX Shell标准的深度分析。**关键改进**: 基于用户反馈，从根本上重构架构，确保前端完全处理语言语义，后端（Executor）作为纯虚拟机。

**核心哲学**:
- **前端职责**（Lexer → Parser → SSA）: 完全处理语言语义，生成完美的IR。所有字符串处理（引号移除、参数展开、命令替换）在此阶段完成。
- **后端职责**（Executor）: 纯虚拟机，信任前端，只执行IR和处理运行时错误。
- **平台支持**: Windows使用原生API（winapi），Unix使用nix库，ProcessManager作为唯一适配层。

**关键策略**:

- **第1优先级 (P0)**: 前端修复 (Parser/Lexer去引号) → SSA完善 (参数展开、命令替换) → Windows进程管理
- **第2优先级 (P1)**: 管道和重定向 (基于新的ProcessManager)
- **第3优先级 (P2)**: 交互式功能放最后
- 每实现一个小功能就进行一次中文commit
- 保持SSA架构优势（当前优化器NOP，但预留未来优化空间）
- 使用Rust生态库替代libc（如os_pipe替代管道系统调用，winapi替代shell调用）
- 增量开发，通过ProcessManager抽象层实现平台无关

---

## 第1阶段：核心脚本执行（Phase 1: Core Script Execution）

**目标**: 使rs-dash-pro可以执行基本的POSIX shell脚本  
**预期结果**: 支持运行大多数控制结构脚本、基本作业控制、函数系统

### 1.1 完整控制结构执行 (Control Structures)

- **现状**: Parser解析了if/for/while/case语句，but执行不完整；break/continue/return作为内置命令而非语法结构
- **架构决策**: 采用现代语言设计范式
  - **Break/Continue/Return** 作为**语言级别的语句**，而非内置命令
  - 在Parser中作为关键词处理，生成专用SSA IR指令
  - 在Executor中维护循环/函数上下文栈
  - 这符合Rust/Go等现代语言的设计，更有利于后续优化
- **任务**:
  - [x] 实现if-elif-else完整执行流
  - [x] 实现for循环的迭代和变量绑定
  - [x] 实现while/until循环的条件判断（含条件式循环SSA-IR）
  - [ ] 实现**break语句**（语法级，非命令）：Parser关键词 → SSA Break指令 → Executor循环上下文跳转
  - [ ] 实现**continue语句**（语法级，非命令）：Parser关键词 → SSA Continue指令 → Executor循环上下文跳转
  - [ ] 实现case语句的模式匹配和分支
  - [ ] 单元测试：覆盖所有控制结构组合

**实现位置**: 

- Parser: `src/modules/parser.rs` (break/continue/return关键词)
- SSA IR: `src/modules/ssa_ir.rs` (Break/Continue/Return指令)
- Generator: `src/modules/ssa_ir_generator.rs` (生成控制流指令)
- Executor: `src/modules/ssa_executor.rs` (循环/函数上下文栈)

**关键架构**: 

- Executor维护 `loop_context_stack: Vec<LoopContext>` 记录循环块信息
- Break指令: 跳转到当前循环的exit块
- Continue指令: 跳转到当前循环的update块
- Return指令: 清除函数栈帧，返回指定状态码

**验收标准**: 

- `./run_tests.sh` 脚本可正确执行
- 嵌套if/for/while脚本可执行
- break/continue正确退出/继续循环
- nested loop中break只影响最内层循环

### 1.2 函数系统完整实现 (Function System)

- **现状**: 函数定义和基本调用已有框架，但缺少参数传递、局部变量、返回值
- **架构决策**: Return作为语言级语句，不是内置命令
  - **Return语句**: 语法级结构，Parser关键词 → SSA Return指令 → Executor函数栈帧返回
  - **Local变量**: 可保留为内置命令，或在语法中实现（如 `local VAR=value`）
- **任务**:
  - [ ] 实现函数参数传递（$1, $2, ..., $@, $#）
  - [ ] 实现local命令或语句（局部变量作用域）
  - [ ] 实现**return语句**（语法级，非命令）：带返回状态码
  - [ ] 实现函数递归调用
  - [ ] 单元测试：参数传递、递归、作用域

**实现位置**: 

- Parser: `src/modules/parser.rs` (return关键词)
- SSA IR: `src/modules/ssa_ir.rs` (Return指令)
- Executor: `src/modules/ssa_executor.rs` (函数栈帧管理)
- Builtins: `src/modules/builtins/local.rs` (局部变量作用域)

**关键架构**: 

- Executor维护 `function_context_stack: Vec<FunctionContext>` 记录函数栈帧
- Return指令: 弹出函数栈帧，设置返回状态码，跳转到调用处
- Local变量: 在当前函数栈帧中创建作用域隔离的变量

**验收标准**: 

- 递归阶乘函数正确运行
- 局部变量不污染全局环境
- 参数扩展正确处理

### 1.3 作业控制系统基础 (Basic Job Control)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现后台执行操作符 `&`
  - [ ] 实现进程组管理（setpgrp, getpgrp）
  - [ ] 实现SIGINT (Ctrl+C), SIGTSTP (Ctrl+Z), SIGCONT信号处理
  - [ ] 实现wait命令（等待所有后台进程）
  - [ ] 实现jobs命令（列出后台进程）
  - [ ] 实现fg命令（前台恢复）
  - [ ] 实现bg命令（后台恢复）
  - [ ] 单元测试：后台运行、信号处理、进程状态

**实现位置**: `src/modules/job_control_enhanced.rs`, `src/modules/ssa_executor.rs`  
**关键依赖**: 需要完成管道执行（1.5）
**技术栈**: 

- 使用nix crate替代libc（已在Cargo.toml）
- os_pipe用于管道创建
- 信号处理通过nix::signal
  **验收标准**: 
- `sleep 10 &` 后台运行成功
- `Ctrl+Z` 暂停进程
- `fg` 恢复前台
- `jobs` 显示所有后台进程

### 1.4 完整变量属性系统 (Variable Attributes)

- **现状**: 基本的变量存储，缺少属性（export、readonly）和作用域
- **任务**:
  - [ ] 实现export内置命令（变量导出到子进程）
  - [ ] 实现readonly内置命令（只读变量保护）
  - [ ] 实现变量属性查询（printenv等）
  - [ ] 实现位置参数设置 `set -- arg1 arg2`
  - [ ] 实现shift命令（位置参数左移）
  - [ ] 实现unset命令的完整功能
  - [ ] 单元测试：属性管理、继承、保护

**实现位置**: `src/modules/variables.rs`, `src/modules/env.rs`, `src/modules/builtins/{export,readonly,set,shift}.rs`  
**关键挑战**: 

- 子进程环境变量继承
- readonly变量的运行时保护
- 变量属性与SSA存储的映射
  **验收标准**: 
- `export VAR=value` 后，子进程可见
- `readonly VAR` 后修改失败
- `set -- a b c` 设置$1, $2, $3正确

### 1.5 完整管道和重定向系统 (Pipes & Redirections)

- **现状**: 基础管道框架，缺少完整的fd操作和Here文档
- **任务**:
  - [ ] 实现多命令管道执行 (`cmd1 | cmd2 | cmd3`)
  - [ ] 实现标准输出重定向 (`>`, `>>`)
  - [ ] 实现标准输入重定向 (`<`)
  - [ ] 实现错误输出重定向 (`2>`, `2>>`)
  - [ ] 实现fd复制 (`n>&m`, `n<&m`)
  - [ ] 实现fd关闭 (`>&-`, `<&-`)
  - [ ] 实现Here文档 (`<< EOF`, `<<- EOF`, `<< "EOF"`)
  - [ ] 实现重定向顺序评估 (先打开文件再fork)
  - [ ] 使用os_pipe库替代系统调用
  - [ ] 单元测试：所有重定向组合

**实现位置**: `src/modules/redirection.rs`, `src/modules/pipeline.rs`, `src/modules/here_doc.rs`  
**技术栈**: 

- os_pipe crate用于管道创建和管理
- nix::fcntl用于fd操作
- duplex实现管道的双向通信
  **关键挑战**: 
- 重定向评估顺序（POSIX指定的）
- Here文档的字符串收集和扩展
- 管道中的fd关闭时序
  **验收标准**: 
- `cat file.txt | grep pattern | wc -l` 正确输出
- `echo hello > file.txt` 创建文件
- `<< EOF` Here文档正确收集多行输入
- 错误输出正确重定向到文件

### 1.6 Shell选项系统 (Shell Options)

- **现状**: set命令框架存在，但选项处理不完整
- **任务**:
  - [ ] 实现set -e (errexit): 错误时退出
  - [ ] 实现set -u (nounset): 未定义变量报错
  - [ ] 实现set -x (xtrace): 调试输出
  - [ ] 实现set -n (noexec): 解析不执行
  - [ ] 实现set -v (verbose): 输出原始命令
  - [ ] 实现set -o选项形式
  - [ ] 实现选项继承到子shell
  - [ ] 单元测试：选项组合、脚本调试

**实现位置**: `src/modules/options.rs`, `src/modules/options_enhanced.rs`  
**关键挑战**: 

- 选项状态在SSA执行器中的维护
- errexit的精确语义（哪些错误触发）
- 选项继承机制
  **验收标准**: 
- `set -e; false; echo never` 不输出never
- `set -u; echo $UNDEFINED` 报错
- `set -x` 后命令执行前输出

### 1.7 完整POSIX内置命令 (POSIX Builtins)

- **现状**: 部分内置命令已实现（cd, export, readonly等）
- **任务**:
  - [ ] 补充缺失的特殊内置命令：
    - `eval` - 完整实现（已有框架）
    - `exec` - 替换shell进程（已有框架）
    - `command` - 执行简单命令（绕过别名/函数）
    - `kill` - 发送信号到进程
    - `read` - 从stdin读取行
    - `type` - 显示命令类型
    - `umask` - 文件创建掩码
    - `printf` - 格式化输出
    - `test`/`[` - 测试表达式
    - `trap` - 信号处理（改进实现）
  - [ ] 完整测试每个命令

**实现位置**: `src/modules/builtins/{command,kill,read,type,umask,printf,test,trap}.rs`  
**关键依赖**: 1.3完成后（作业控制），kill/trap才能工作
**验收标准**: 

- `command ls` 执行external ls
- `read var < file` 读取第一行
- `type ls` 显示"ls is /bin/ls"
- `kill -TERM $PID` 发送信号
- `trap 'echo INTERRUPT' INT` 处理中断

---

## 第2阶段：交互式使用（Phase 2: Interactive Shell）

⚠️ **注意**: 本阶段优先级调整为最后（在Phase 1/3/4之后）

**目标**: 使rs-dash-pro可作为交互式shell使用  
**依赖**: Phase 1完成、Phase 3/4基本完成

### 2.1 高级参数扩展 (Advanced Parameter Expansion)

- **现状**: 基础 `$VAR` 和 `${VAR:-default}` 支持
- **任务**:
  - [ ] 实现 `${VAR#pattern}`, `${VAR##pattern}` (前缀删除)
  - [ ] 实现 `${VAR%pattern}`, `${VAR%%pattern}` (后缀删除)
  - [ ] 实现 `${VAR/pattern/replace}` (全局替换)
  - [ ] 实现 `${VAR:offset:length}` (子字符串)
  - [ ] 实现 `${#VAR}` (字符串长度)
  - [ ] 实现 `${VAR:+word}` (存在时替换)
  - [ ] 实现 `${VAR:?message}` (不存在则报错)
  - [ ] 单元测试：所有参数扩展形式

**实现位置**: `src/modules/param_expand.rs`, `src/modules/expansion.rs`  
**关键挑战**: 模式匹配的POSIX语义（glob vs正则）

### 2.2 完整算术扩展 (Arithmetic Expansion)

- **现状**: 基础 `$((1+2))` 支持
- **任务**:
  - [ ] 实现所有二元运算符: +, -, *, /, %, **, <<, >>, &, |, ^
  - [ ] 实现所有一元运算符: -, +, ~, !
  - [ ] 实现逻辑运算符: &&, ||
  - [ ] 实现赋值运算符: =, +=, -=, *=, /=, %=, <<=, >>=, &=, |=, ^=
  - [ ] 实现三元运算符: ? :
  - [ ] 实现逗号运算符
  - [ ] 实现基数转换: `2#1010`, `16#FF`
  - [ ] 实现变量副作用: `$((i++))`
  - [ ] 单元测试：所有运算符组合

**实现位置**: `src/modules/arithmetic.rs`  
**关键挑战**: 

- 运算符优先级和结合性
- 变量副作用的SSA表示

### 2.3 路径名扩展 (Pathname Expansion)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现通配符 `*` (任意字符)
  - [ ] 实现通配符 `?` (单个字符)
  - [ ] 实现字符类 `[abc]`, `[a-z]`, `[!abc]`
  - [ ] 实现模式排序
  - [ ] 实现noglob选项支持
  - [ ] 单元测试：所有模式形式

**实现位置**: `src/modules/expansion.rs` (新增pathname模块)  
**技术栈**: 

- glob crate用于glob模式匹配
- 或自己实现POSIX glob语义

### 2.4 波浪号扩展 (Tilde Expansion)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现 `~` 扩展到$HOME
  - [ ] 实现 `~user` 扩展到用户主目录
  - [ ] 实现在适当上下文的扩展
  - [ ] 单元测试：路径中的波浪号

**实现位置**: `src/modules/expansion.rs` (新增tilde模块)  
**技术栈**: users crate用于用户信息查询

### 2.5 行编辑和历史 (Line Editing & History)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现基本行编辑：退格、Ctrl+U清行、Ctrl+D退出
  - [ ] 实现历史存储（内存中）
  - [ ] 实现上下箭头导航历史
  - [ ] 实现历史搜索 (Ctrl+R)
  - [ ] 可选：Tab补全基础
  - [ ] 单元测试：交互模式测试

**实现位置**: 新建 `src/modules/line_edit.rs`  
**技术栈**: 

- rustyline crate用于行编辑
- 或自己实现简单行编辑逻辑

### 2.6 数组变量 (Array Variables)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现数组赋值 `array=(a b c)`
  - [ ] 实现数组访问 `${array[i]}`
  - [ ] 实现数组展开 `${array[@]}`, `${array[*]}`
  - [ ] 实现数组长度 `${#array[@]}`
  - [ ] 单元测试：数组操作

**实现位置**: `src/modules/variables.rs` (扩展)  
**关键挑战**: 

- SSA中的数组表示
- 数组在环境变量中的编码

---

## 第3阶段：完全POSIX兼容（Phase 3: Full POSIX Compliance）

**目标**: 通过POSIX测试套件，与dash行为一致  
**依赖**: Phase 1完全完成

### 3.1 进程替换 (Process Substitution)

- **现状**: 词法分析器支持，执行不完整
- **任务**:
  - [ ] 实现 `<(command)` 为读管道
  - [ ] 实现 `>(command)` 为写管道
  - [ ] 实现FIFO创建和管理
  - [ ] 单元测试：进程替换管道

**实现位置**: `src/modules/process_substitution.rs`

### 3.2 Subshell完整支持 (Complete Subshell Support)

- **现状**: 基础框架存在
- **任务**:
  - [ ] 实现 `(...)` 在独立进程中执行
  - [ ] 实现变量作用域隔离
  - [ ] 实现 `{...}` 复合命令（同进程）
  - [ ] 实现 `{ ... } > file` 重定向整体
  - [ ] 单元测试：subshell隔离、变量作用域

### 3.3 命令替换完整性 (Complete Command Substitution)

- **现状**: `$(command)` 基本支持
- **任务**:
  - [ ] 完整支持嵌套命令替换
  - [ ] 支持错误传播
  - [ ] 支持老语法 `` `command` ``
  - [ ] 单元测试：嵌套、错误处理

### 3.4 POSIX兼容性测试 (POSIX Compliance Testing)

- **现状**: 无测试套件
- **任务**:
  - [ ] 建立与dash的对比测试框架
  - [ ] 编写1000+个POSIX测试用例
  - [ ] 针对差异修复
  - [ ] 边缘情况处理

**实现位置**: `tests/posix_compliance.rs`

### 3.5 错误处理完善 (Error Handling Refinement)

- **现状**: 基础错误处理
- **任务**:
  - [ ] 精确的语法错误报告（行号）
  - [ ] 与dash错误消息一致
  - [ ] 交互模式错误恢复
  - [ ] 单元测试：各种错误场景

### 3.6 性能基准测试 (Performance Benchmarking)

- **现状**: 无性能数据
- **任务**:
  - [ ] 建立性能基准
  - [ ] 与dash对比
  - [ ] 识别性能瓶颈

---

## 第4阶段：优化和完善（Phase 4: Optimization & Polish）

**目标**: SSA优化实现，性能达到dash水平  
**依赖**: Phase 1和Phase 3基本完成

### 4.1 SSA优化器实现 (SSA Optimizer)

- **现状**: NOP实现
- **任务**:
  - [ ] 死代码消除 (Dead Code Elimination)
  - [ ] 常量传播 (Constant Propagation)
  - [ ] 公共子表达式消除 (Common Subexpression Elimination)
  - [ ] 控制流优化
  - [ ] 单元测试：优化正确性验证

**实现位置**: `src/modules/optimize/optimizer.rs`

### 4.2 命令哈希系统 (Command Hashing)

- **现状**: 完全缺失
- **任务**:
  - [ ] 实现hash内置命令
  - [ ] 缓存PATH搜索结果
  - [ ] 快速路径执行

**实现位置**: `src/modules/builtins/hash.rs`

### 4.3 性能优化 (Performance Tuning)

- **现状**: 无特殊优化
- **任务**:
  - [ ] 字符串驻留优化
  - [ ] 内存分配优化
  - [ ] 热路径优化
  - [ ] 基准测试验证

---

## 技术决策和约束

### 0. 语言级控制流结构（新增 - 现代化架构）

为了拥抱现代语言设计范式，而非墨守成规地实现成古老shell的内置命令模式：

**语言级语句**（Parser关键词 → SSA IR指令 → Executor上下文跳转）：

- **break** - 退出当前循环，跳转到loop exit块
- **continue** - 继续当前循环，跳转到loop update块
- **return** - 从函数返回，弹出栈帧，返回状态码

**内置命令**（保持为特殊内置命令或函数库）：

- `cd`, `echo`, `export`, `readonly`, `local`, `eval`, `exec`, `command`, `kill`, `read`, `type`, `umask`, `printf`, `test`, `trap` 等

**好处**：

- 更清晰的控制流语义，便于优化器分析
- 类似Rust/Go/Python等现代语言的做法
- 编译期可进行更多的控制流分析和验证
- 嵌套控制流的处理更直观

### 1. SSA架构维护

- 当前阶段优化器为NOP，但架构预留
- 所有功能实现都需要在SSA IR中表示清晰
- 保持Lexer → Parser → SSA IRGenerator → Executor流程
- **控制流指令**: Break/Continue/Return在SSA IR中是一等指令，不是特殊处理

### 2. 库的选择

- **禁用**: `libc::` 直接系统调用
- **推荐**:
  - `os_pipe`: 管道创建
  - `nix`: POSIX系统接口（进程、信号、fd操作）
  - `glob`: 路径名模式匹配
  - `users`: 用户信息查询
  - `rustyline`: 交互式行编辑
  - `tempfile`: 测试临时文件

### 3. 开发流程

- 每个小功能完成一次中文commit
- 中文commit消息格式: `[功能点] 实现/改进xxx`
  - 例: `[控制结构] 实现if-elif-else完整执行`
  - 例: `[参数扩展] 支持${VAR#pattern}模式删除`
- 保证每次提交都能 `cargo build` 和 `cargo test` 通过

### 4. 平台支持

- **优先**: Linux (WSL2 Ubuntu 24.04)
- **支持**: Windows (语法和命令工作，实现可能不同)
- 使用条件编译: `#[cfg(unix)]` / `#[cfg(windows)]`

### 5. 测试策略

- 单元测试：内联 `#[cfg(test)]` 模块
- 集成测试：`tests/` 目录
- 对比测试：与标准dash行为一致
- 自动化: `cargo test` 全通过

---

## 实现优先级和依赖关系

```
🎯 新的执行顺序: Phase 1 → Phase 3 → Phase 4 → Phase 2 (最后)

Phase 1 (脚本执行可用) ← 优先级 1️⃣ :
├─ 1.1 控制结构 (独立) ✓
├─ 1.2 函数系统 (独立) ✓
├─ 1.3 作业控制 (依赖: 1.5)
├─ 1.4 变量属性 (独立) ✓
├─ 1.5 管道重定向 (依赖: 1.1, 1.2) ✓
├─ 1.6 Shell选项 (独立) ✓
└─ 1.7 POSIX内置 (依赖: 1.3, 1.5)

Phase 3 (POSIX兼容) ← 优先级 2️⃣ (需要 Phase 1 完成):
├─ 3.1 进程替换 (依赖: 1.5)
├─ 3.2 Subshell支持 (依赖: 1.1, 1.2)
├─ 3.3 命令替换 (独立)
├─ 3.4 POSIX测试 (依赖: 1-3)
├─ 3.5 错误处理 (独立)
└─ 3.6 性能基准 (独立)

Phase 4 (优化) ← 优先级 3️⃣ (需要 Phase 3 测试通过):
├─ 4.1 SSA优化器 (依赖: Phase 3 POSIX测试)
├─ 4.2 命令哈希 (依赖: 1.7)
└─ 4.3 性能优化 (依赖: 4.1, 4.2)

Phase 2 (交互式) ← 优先级 4️⃣ (最后，需要 Phase 1/3/4 基本完成):
├─ 2.1 参数扩展 (依赖: 1.4)
├─ 2.2 算术扩展 (依赖: 1.6)
├─ 2.3 路径名扩展 (依赖: Phase 3)
├─ 2.4 波浪号扩展 (独立)
├─ 2.5 行编辑历史 (依赖: 1.3)
└─ 2.6 数组变量 (独立)
```

**✓ 标记**: 现在可独立开始（无依赖）
**? 标记**: 等待其他任务完成

---

## 成功指标

## 成功指标

### Phase 1完成

- [ ] `cargo test` 所有测试通过
- [ ] 能运行 `/bin/sh` 格式的shell脚本
- [ ] 后台进程 (`&`) 正常运行
- [ ] 函数递归调用正确
- [ ] 所有P0优先级功能实现

### Phase 3完成

- [ ] 通过POSIX兼容性测试
- [ ] 与dash行为一致（95%+）
- [ ] 支持所有POSIX特殊内置命令
- [ ] 完整的错误处理

### Phase 4完成

- [ ] SSA优化器实现
- [ ] 性能与dash接近
- [ ] 完整的文档
- [ ] 所有平台测试通过

### Phase 2完成（最后优先级）

- [ ] 可作为交互式shell使用
- [ ] 支持命令历史和行编辑
- [ ] 高级参数扩展工作
- [ ] 数组变量支持

---

## 风险和缓解

### 风险1: 复杂性爆炸

**问题**: shell功能繁多，实现可能失控
**缓解**: 

- 严格按阶段进行，不提前实现
- 小功能快速迭代
- 定期代码审查

### 风险2: 性能回归

**问题**: SSA可能比直接执行慢
**缓解**:

- Phase 1-3确保功能正确
- Phase 4进行优化
- 持续性能测试

### 风险3: 跨平台不一致

**问题**: Windows实现可能有差异
**缓解**:

- Linux优先，然后测试Windows
- 清晰的条件编译界限
- 文档记录平台差异

### 风险4: 兼容性问题

**问题**: dash脚本行为差异
**缓解**:

- Phase 3对比测试
- 使用dash源码作参考
- 建立回归测试

---

## 参考资源

1. **POSIX标准**: IEEE Std 1003.1-2017 Shell Command Language
2. **dash源码**: `../c-dash/` (v0.5.3)
3. **现有文档**:
   - `Compare.md` - 功能对比分析
   - `TODO.md` - 原始任务列表
   - `SSA_DESIGN.md` - SSA架构设计
   - `IMPLEMENTATION_PLAN.md` - 前期计划
4. **Rust生态**:
   - os_pipe: 管道实现
   - nix: POSIX接口
   - rustyline: 行编辑

---

## 文档版本

- **创建时间**: 2026-02-27
- **最后更新**: 2026-02-27
- **基于**: 
  - Compare.md (功能对比)
  - TODO.md (原始优先级)
  - SSA_DESIGN.md (架构)
  - 用户需求（分阶段、中文commit、Rust库优先）

## 临时需求 (2026-02-27 ~ 2026-02-28)

**Phase 1.1 完整控制结构执行 - 已完成✅**:

- ✅ If-elif-else 完整执行流
- ✅ For 循环的迭代和变量绑定（条件循环SSA-IR）
- ✅ While/until 循环的条件判断
- ✅ Break 语句（语言级，SSA Break指令）
- ✅ Continue 语句（语言级，SSA Continue指令）
- ✅ Case 语句的模式匹配
- ✅ 嵌套控制结构支持
- **状态**: 6/6 任务完成，P1.1 = 100%

**Phase 1.2 函数系统完整实现 - 已完成✅**:

- ✅ 函数定义: name() { ... } 和 function name { ... }
- ✅ 函数参数传递: $0, $1-$9, $#, $@, $*
- ✅ 局部变量作用域: local 命令
- ✅ Return 语句（语言级，SSA Return指令）
- ✅ 函数递归调用
- ✅ 函数内的控制流（break/continue/return）
- **状态**: 6/6 任务完成，P1.2 = 100%

**Phase 1.4 完整变量属性系统 - 已完成✅**:

- ✅ export 内置命令
- ✅ readonly 内置命令
- ✅ local 命令（局部变量作用域）
- ✅ set 命令（位置参数设置）
- ✅ shift 命令（位置参数左移）
- ✅ unset 命令
- ✅ printenv 命令（本会话新增）
- **状态**: 1/1 任务完成，P1.4 = 100%

**本会话改进 (2026-02-28)**:

1. ✅ 修复特殊参数变量名验证 ($0, $#, $*, $@, $?, $-, 新增 $1-$9)
2. ✅ 实现 SSA IR 中的 builtin 命令识别 (is_builtin_command)
3. ✅ 添加 printenv builtin 命令
4. ✅ 完整的函数系统文档 (FUNCTIONS_GUIDE.md)
5. ✅ 自定义函数系统完整总结 (FUNCTIONS_SYSTEM_COMPLETE.md)

**整体进度**:

- 已完成: 15/34 任务 (44%)
  - P1.1: 6/6 ✅ (功能完成，但存在Parser bug)
  - P1.2: 6/6 ✅ (功能完成，但存在Parser bug)
  - P1.4: 1/1 ✅
  - Other: 2/2 ✅

- 阻断性缺陷待修复: 4个bug (高优先级)
  - [BUG-P0] Parser分号/换行处理 ⏳ (影响所有多命令脚本)
  - [BUG-P0] 函数调用赋值卡死 ⏳ (交互式完全不可用)
  - [BUG-P1] 字符串去引号失效 ⏳ (输出异常)
  - [BUG-P2] SSA无意义const赋值 ⏳ (性能问题)
  
- 待进行: 19/34 任务
  - P1.3 作业控制: 0/1 ⏳ (需等待bug修复)
  - P1.5 管道重定向: 0/1 ⏳ (需等待bug修复)
  - P1.6 Shell选项: 0/1 ⏳ (需等待bug修复)
  - P1.7 POSIX内置: 0/1 ⏳ (需等待bug修复)
  - Phase 2-4: 15/15 ⏳ (需等待bug修复)

**严重缺陷发现 (2026-02-28)**:

需要在推进 P1.3-P1.7 之前，先修复以下**阻断性缺陷**，否则任何后续功能都会因这些bug而无法正常工作。

### 缺陷1: 分号和换行处理不当 ⛔ [CRITICAL]

**现象**:
```
DEBUG PARSER SIMPLE_CMD: Processing token: Semicolon
DEBUG PARSER SIMPLE_CMD: Unknown token type, breaking: Semicolon
DEBUG PARSER SIMPLE_CMD: Processing token: Newline
DEBUG PARSER SIMPLE_CMD: Unknown token type, breaking: Newline
```

**问题描述**:
- Parser的`parse_simple_command()`无法识别`;`和`\n`作为命令分隔符
- 每当遇到分号或换行，就粗暴中断命令解析，丢弃这两个关键的流程控制符
- 这导致在函数体内包含分号分隔的多个命令时，只有第一个命令被解析

**预期行为**:
- `;` 应被识别为**命令序列分隔符**（statement separator）
- `\n` 应被识别为**可选的命令分隔符**
- Parser应该在`parse_simple_command()`后，检查是否遇到分隔符，然后继续解析下一条命令

**原因推测**:
- `parse_simple_command()`中的token循环缺少对Semicolon和Newline的处理
- 应该在遇到Semicolon/Newline时**不是中断**，而是**返回**当前已解析的命令

**影响范围**:
- 所有包含`;`的命令都无法正确解析
- 函数体内的多命令序列无法解析
- 脚本中的命令序列执行顺序错误

### 缺陷2: SSA IR生成大量无意义的const_int赋值 ⛔ [CRITICAL]

**现象**:
```
.0: %1 = const_int 0
    %2 = const_int 0
    %3 = const_int 0
    ...
    %5 = const_int 0
```

**问题描述**:
- SSA IR生成阶段产生大量`const_int 0`赋值，且这些值从未被使用过
- 这表明Generator在生成某些AST节点时逻辑缺失
- 这些无效指令会浪费执行时间，且表明IR生成流程有缺陷

**原因推测**:
- `generate_*`方法可能在遇到NullCommand或某些AST节点时，生成默认的常数赋值
- 需要检查`generate_command_list`, `generate_command`等方法

**影响范围**:
- IR代码膨胀，可能导致执行效率下降
- 掩盖了真实的执行逻辑问题
- 表明Generator对某些AST节点的处理不完整

### 缺陷3: 字符串常量被直接输出，而非通过echo命令执行 ⛔ [CRITICAL]

**现象**:
```
预期输出: hello, world!
实际输出: "hello, world!"
```

**问题描述**:
- 函数体内的`echo "hello, world!"`应该输出：`hello, world!`
- 但实际输出包含了双引号：`"hello, world!"`
- 这说明字符串常量被直接输出，而非被正确地作为echo命令的参数处理

**原因推测**:
1. Parser可能将带引号的字符串保存为`Word("\"hello, world!\"")`（包含引号）
2. Executor执行时，未进行**字符串去引号处理**（quote removal）
3. Word常量被直接作为IR输出，而非通过命令执行

**影响范围**:
- 所有字符串输出都会多出引号
- echo、printf等输出命令行为异常
- 字符串参数传递错误

### 缺陷4: 函数调用导致程序卡死（无限循环或死锁） ⛔ [CRITICAL]

**现象**:
```
输入: a = test_func
结果: 程序卡死，需要 Ctrl+C 中断
```

**问题描述**:
- 当执行`a = test_func`时（试图将函数返回值赋给变量），程序陷入无限循环/死锁
- 这是比缺陷1-3更严重的问题，直接导致交互式输入失响

**原因推测**:
1. 可能与缺陷1相关：函数调用后的`=`赋值无法正确解析
2. Parser可能陷入无限循环尝试解析该命令行
3. 变量赋值与函数调用的交互有bug

**影响范围**:
- 任何涉及函数结果赋值的脚本都会卡死
- 交互式使用完全不可用
- 严重影响可用性

---

### 修复优先级（由高到低）:

1. **缺陷1: Parser分号/换行处理** 
   - 优先级: P0 (阻断所有后续功能)
   - 修复位置: `src/modules/parser.rs` `parse_simple_command()`方法

2. **缺陷4: 函数调用卡死**
   - 优先级: P0 (交互式完全不可用)
   - 修复位置: Parser的命令解析流程、变量赋值处理

3. **缺陷3: 字符串去引号**
   - 优先级: P1 (输出异常但功能可用)
   - 修复位置: `src/modules/ssa_executor.rs` 命令执行时的参数处理

4. **缺陷2: SSA无意义const_int赋值**
   - 优先级: P2 (性能问题，功能正确性不受影响)
   - 修复位置: `src/modules/ssa_ir_generator.rs` IR生成逻辑

---

## 交互式Shell新需求 (2026-02-28 追加)

### 多行输入支持 (Critical for Interactive Mode)

**需求1: 反斜杠续行**
- 在行尾使用`\`时，表示续行（不发送命令）
- 下一行继续接收输入
- 示例:
  ```
  echo "hello" \
  "world"
  ```

**需求2: 括号/引号未闭合时自动进入多行模式**
- 当输入中有未闭合的括号或引号时，不发送命令
- 自动进入多行输入模式
- 等待用户闭合括号/引号
- 示例:
  ```
  (echo "hello"
  echo "world"
  )
  ```

**需求3: Ctrl+C 和 Ctrl+D 支持**
- **Ctrl+C**: 打断当前输入行或正在执行的命令
  - 若有未完成的命令行，清除该行
  - 若正在执行命令，终止该命令进程
- **Ctrl+D**: 退出Shell
  - 在空行时按Ctrl+D表示EOF，退出交互式模式

**实现位置**: 需要新增或改进交互式循环逻辑
- 输入行读取: 支持`\`续行检测
- 括号/引号追踪: 未闭合时不解析
- 信号处理: SIGINT捕获Ctrl+C

### 关键架构决策

- **多行缓冲**: 在交互式模式下维护输入缓冲，直到获得完整命令
- **括号匹配**: 扫描输入检查未闭合的`(`, `[`, `{`, `"`, `'`, `` ` ``
- **信号处理**: 注册SIGINT处理器，不让Ctrl+C导致退出
- **清晰的用户反馈**: 多行模式下显示continuation prompt（如`> `）

---

### 测试用例（验证修复）:

```bash
# 测试缺陷1: 分号和换行
test_func() { echo "step1"; echo "step2" }
test_func
# 预期: step1 \n step2

# 测试缺陷4: 函数调用赋值
test_func() { return 0 }
a=$(test_func)
echo $a
# 预期: (程序响应，不卡死)

# 测试缺陷3: 字符串去引号
echo "hello"
# 预期: hello (不带双引号)

# 测试缺陷2: SSA IR清晰
set -x  # 如果实现了set -x，应该只输出有意义的指令
```

---

P1.3 作业控制系统基础 (依赖 P1.5):

- [ ] 实现后台执行操作符 `&`
- [ ] 进程组管理
- [ ] 信号处理 (SIGINT, SIGTSTP, SIGCONT)
- [ ] wait 命令
- [ ] jobs 命令
- [ ] fg/bg 命令

P1.5 完整管道和重定向系统 (优先):

- [ ] 多命令管道执行
- [ ] 输出重定向 (>, >>)
- [ ] 输入重定向 (<)
- [ ] 错误输出重定向 (2>, 2>>)
- [ ] fd 复制和关闭
- [ ] Here 文档 (<<EOF)

**关键决策**:

- 语言级控制流: break, continue, return 作为关键词 (已实现✅)
- SSA 架构维护: 所有功能都在 SSA IR 中表示 (已实现✅)
- Builtin 识别: 在 SsaIrGenerator 中识别 builtin 命令 (已实现✅)
