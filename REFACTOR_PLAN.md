# rs-dash-pro 重构计划

## 目标架构
Lexer → Parser → IRGenerator[SSA] → Optimizer[NOP] → VMExecutor

## 当前状态分析

### 已实现部分：
1. 基础Lexer - 支持基本词法分析
2. 基础Parser - 支持基本语法分析
3. SSA IR结构定义 - 基本完成
4. SSA IR生成器 - 部分实现
5. SSA执行器 - 部分实现

### 需要增强的部分：

#### Phase 1: 增强Lexer (POSIX完整支持)
- [ ] 完整的POSIX token集
- [ ] Here-document处理
- [ ] Process substitution (`<(cmd)`, `>(cmd)`)
- [ ] 算术扩展 `$((expr))`
- [ ] 模式匹配通配符
- [ ] 转义序列处理
- [ ] 多字节字符支持

#### Phase 2: 增强Parser (完整AST生成)
- [ ] 完整的POSIX语法规则
- [ ] Case语句
- [ ] Select语句
- [ ] 函数定义
- [ ] 别名展开
- [ ] 参数扩展完整语法
- [ ] 命令替换完整语法

#### Phase 3: 完善SSA IR生成器
- [ ] 所有AST节点到SSA IR的转换
- [ ] 控制流图生成
- [ ] Phi节点插入
- [ ] 变量作用域处理
- [ ] 环境变量处理

#### Phase 4: 完善SSA执行器
- [ ] 所有SSA指令的执行语义
- [ ] 进程管理 (fork/exec/wait)
- [ ] 管道和重定向
- [ ] 信号处理
- [ ] 作业控制

#### Phase 5: 优化器 (NOP实现)
- [ ] 空优化器框架
- [ ] 未来优化接口设计

#### Phase 6: 测试和验证
- [ ] POSIX一致性测试套件
- [ ] 与原生dash行为对比
- [ ] Windows/WSL双平台测试
- [ ] 性能基准测试

## 实施步骤

### Step 1: 增强Lexer
基于现有lexer.rs，添加缺失的POSIX功能。

### Step 2: 增强Parser
基于现有parser.rs，完善语法规则，确保能解析所有POSIX Shell语法。

### Step 3: 完善SSA IR
基于现有ssa_ir.rs，确保IR能表示所有Shell语义。

### Step 4: 完善IR生成器
基于现有ssa_ir_generator.rs，实现完整的AST到SSA转换。

### Step 5: 完善执行器
基于现有ssa_executor.rs，实现完整的SSA执行语义。

### Step 6: 集成测试
创建测试套件，确保功能完整性和POSIX兼容性。

## 提交策略
- 每完成一个主要功能模块就提交一次
- 提交信息使用中文，简洁明了
- 确保每次提交都能编译通过
- 优先实现核心功能，再完善边缘情况

## 参考资源
1. POSIX.1-2017 Shell Command Language
2. dash源代码 (c-dash目录)
3. rs-dash实现 (rs-dash目录)
4. 现有rs-dash-pro代码