# rs-dash-pro 架构重构计划 (2026-02-28)

## 核心问题诊断

当前系统存在两个根本性的架构问题：

### 问题1: 前端-后端职责混乱

**症状**: Executor在执行时需要处理本应在前端解决的问题

- 去除引号函数存在于Executor而不是Parser
- 字符串展开逻辑分散在各处
- 语法错误应该在Parser阶段报错，而不是Executor才发现

**根因**: Parser/SSA生成器对"正确的IR"定义不清晰

**哲学**: Executor应该是一个**信任前端**的虚拟机

- 前端（Lexer → Parser → SSA Generator）：完全处理语言语义，生成完美的IR
- Executor：只是IR的虚拟机，信任所有输入的正确性
- 错误分类：
  - **编译时错误**（语法错误）→ Parser/Generator阶段报错
  - **运行时错误**（栈溢出等）→ Executor阶段报错

### 问题2: Windows缺乏原生支持

**症状**: 

- fork模拟使用cmd.exe/powershell
- 系统不支持CMD/PWSH时完全不能运行
- 无法充分利用Windows Process API的特性

**根因**: 代码针对Unix设计，Windows是afterthought

**哲学**: 使用Windows原生API处理Windows问题

- fork/exec → CreateProcess (Windows Process API)
- 管道 → CreatePipe (Windows原生管道)
- 信号 → Windows事件和Job Object
- 不依赖外部shell (cmd.exe/powershell.exe)

---

## 重构方案

### 第一阶段: 前端完善（Parser → SSA）

#### 1.1 词法分析器 (Lexer) 改进

**目标**: Lexer正确识别引号语义，保留必要信息供Parser使用

**改动**:

1. Token中区分"字面字符串"和"带引号的字符串"
   
   ```rust
   pub enum Token {
       // 旧: Word(String)  // 可能包含引号
       // 新:
       Word(String),                    // 普通词（无特殊处理）
       QuotedString(String),            // 带引号的字符串（引号已移除）
       SingleQuotedString(String),      // 单引号字符串（字面量）
       // ...
   }
   ```

2. Lexer在词法分析时移除外层引号
   
   ```
   输入: "hello world"
   输出: QuotedString("hello world")  // 引号被移除，但类型标记保留
   ```

**位置**: `src/modules/lexer.rs`
**关键函数**: `scan_string()`, `scan_single_quoted()`, `scan_double_quoted()`

#### 1.2 Parser改进

**目标**: Parser生成的AST包含正确的字符串信息，不再有"包含引号的字面字符串"

**改动**:

1. SimpleCommand的args不再包含引号字符
   
   ```rust
   // 旧的问题:
   SimpleCommand {
       name: "echo",
       args: vec!["\"hello\""],  // ❌ 包含引号字符
   }
   
   // 新的正确方式:
   SimpleCommand {
       name: "echo",
       args: vec!["hello"],  // ✅ 干净的字符串
   }
   ```

2. 参数展开在Parser阶段识别
   
   ```rust
   // 例如: echo $var 或 echo ${var}
   // Parser应该识别为参数展开，而不是字面字符串 $var
   ```

**位置**: `src/modules/parser.rs`
**关键函数**: `parse_simple_command()`, `parse_word()`

#### 1.3 SSA IR生成器改进

**目标**: SSA IR完全反映语言语义，不留歧义

**改动**:

1. 字符串常量已经是干净的
   
   ```ir
   // 旧的问题:
   %1 = const_string "\"hello\""  // ❌ 包含引号
   
   // 新的正确方式:
   %1 = const_string "hello"  // ✅ 干净的字符串
   ```

2. 参数展开生成专用指令
   
   ```ir
   // 例如: echo $var
   %1 = param_expand "var" simple
   %2 = call_builtin "echo" %1
   ```

3. 命令替换生成专用指令
   
   ```ir
   // 例如: x=$(cmd)
   %1 = cmd_subst "cmd"
   %2 = store_var "x" %1
   ```

**位置**: `src/modules/ssa_ir_generator.rs`
**新增指令类型**: (SSA IR中)

- `ParamExpand(mode)` - 参数展开
- `CmdSubst` - 命令替换
- `QuoteRemoval` - 显式的引号移除（如需要）

#### 1.4 执行器简化

**改动**: Executor中移除所有"擦屁股"代码

```rust
// 删除: remove_quotes() 函数

// CallBuiltin执行变为:
Instruction::CallBuiltin(name, args, result) => {
    let arg_strings: Vec<String> = args
        .iter()
        .map(|arg| self.get_value(*arg).as_string())  // 直接使用，已经正确
        .collect();

    let status = self.builtins.execute(name, &arg_strings, &mut self.env);
    // ...
}
```

**位置**: `src/modules/ssa_executor.rs`

---

### 第二阶段: Windows原生支持

#### 2.1 进程创建抽象层

**目标**: 统一的进程创建接口，底层在Unix/Windows上有不同实现

**新建模块**: `src/modules/process_manager.rs`

```rust
pub struct ProcessManager;

impl ProcessManager {
    /// 创建并执行进程（Unix: fork+exec, Windows: CreateProcess）
    pub fn spawn(cmd: &str, args: &[String]) -> Result<ProcessHandle>;

    /// 创建管道（Unix: pipe(), Windows: CreatePipe）
    pub fn create_pipe() -> Result<(i32, i32)>;

    /// 等待进程（Unix: waitpid, Windows: WaitForSingleObject）
    pub fn wait_process(handle: ProcessHandle) -> Result<i32>;

    /// 发送信号/事件（Unix: kill, Windows: TerminateProcess等）
    pub fn signal_process(handle: ProcessHandle, signal: i32) -> Result<()>;
}
```

#### 2.2 Unix实现

**位置**: `src/modules/process_manager/unix.rs`

使用nix crate:

- fork/exec → nix::unistd::{fork, execvp}
- 管道 → nix::unistd::pipe()
- 信号 → nix::signal
- 进程组 → nix::unistd::{getpgrp, setpgrp}

#### 2.3 Windows实现

**位置**: `src/modules/process_manager/windows.rs`

使用winapi crate或std库:

- CreateProcess → 直接进程创建
- CreatePipe + 标志位处理重定向
- Job Object处理进程组
- 事件处理代替信号

**关键特性**:

```rust
#[cfg(windows)]
{
    // 后台进程: 使用 DETACHED_PROCESS 标志
    // 管道连接: 使用 SetHandleInformation 和管道继承
    // 进程组: 使用 Job Object (CREATE_NEW_PROCESS_GROUP)
}
```

#### 2.4 跨平台抽象

**Cargo.toml依赖更新**:

```toml
[target.'cfg(unix)'.dependencies]
nix = { version = "0.27", features = ["process", "signal", "fs"] }

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["processthreadsapi", "winbase", "handleapi"] }

[dependencies]
# 已有:
os_pipe = "1"
```

---

## 实施路线图

### 第1步: Parser/Lexer修复 (优先级: P0)

1. 修改Lexer区分字符串类型 → 移除外层引号
2. 更新Parser处理新Token类型 → 生成干净的AST
3. 修改SSA生成器 → 移除remove_quotes调用
4. 清理Executor → 删除remove_quotes函数
5. 验证: 测试脚本输出正确

**预期工作量**: 2-3个文件，相对小改

### 第2步: SSA IR完善 (优先级: P0)

1. 实现ParamExpand指令 → $var 和 ${var}
2. 实现CmdSubst指令 → $(cmd) 和 `cmd`
3. 更新SSA生成器生成这些指令
4. 更新Executor执行这些指令
5. 验证: 参数展开和命令替换工作正常

**预期工作量**: 4-5个文件，中等改

### 第3步: Windows进程管理 (优先级: P0)

1. 创建ProcessManager抽象层
2. 实现Unix版本 (nix crate)
3. 实现Windows版本 (winapi)
4. 更新现有fork/exec调用 → 使用ProcessManager
5. 测试后台执行、管道等功能

**预期工作量**: 3个文件 + 大量修改，较大改

### 第4步: 管道和重定向 (优先级: P1)

基于新的ProcessManager实现完整的管道

---

## 关键决策

### 1. Token设计

- QuotedString vs Word: 类型标记引号信息，但移除引号字符
- 这样Parser接收到已"清理"的字符串

### 2. SSA IR扩展

- 参数展开、命令替换等应该是显式的IR指令
- Executor按指令执行，不猜测意图

### 3. 错误报告

- Parser阶段: 语法错误 → ParseError 立即返回
- SSA生成阶段: 语义错误（未定义的变量等） → SSA IR中的Error指令
- Executor阶段: 运行时错误（栈溢出等）

### 4. 平台差异

- ProcessManager是唯一的平台适配层
- 其他代码100%平台无关

---

## 验收标准

### Parser/Lexer修复完成后:

```sh
✅ echo "hello world"        # 输出: hello world (无多余引号)
✅ echo 'hello $var'         # 输出: hello $var (单引号无展开)
✅ echo "hello $var"         # 输出: hello <var值>
✅ echo hello\ world         # 输出: hello world (转义)
```

### SSA IR完善后:

```sh
✅ var=value; echo $var      # 输出: value
✅ echo ${var}               # 输出: value
✅ result=$(echo test)       # 赋值工作
✅ echo $(echo hello)        # 输出: hello (嵌套命令替换)
```

### Windows支持完善后:

```sh
✅ 在纯Windows环境（无WSL/bash）运行完整功能
✅ 后台进程 (sleep 10 &) 工作
✅ 管道 (echo test | grep test) 工作
✅ 重定向 (echo hello > file.txt) 工作
```

---

## 后续优化机会

1. **SSA优化器**: 在Executor前进行IR优化（常数折叠、死代码消除等）
2. **缓存**: 编译结果缓存（快速重复执行）
3. **JIT**: 对频繁执行的路径进行JIT编译

但这些都不影响当前的正确性，可以后续添加。

---

**重构目标**: 使系统架构清晰、职责分明、平台无关
**预期结果**: 更易维护、更易扩展、更完整的POSIX兼容性
