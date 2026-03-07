# POSIX 23个必选信号（Windows兼容版）

## 一、不可捕获/忽略的信号（2个）

| 信号名     | 编号  | 核心作用说明                 | Windows等价行为                                         | 实现说明（Shell层面）                               |
| ------- | --- | ---------------------- | --------------------------------------------------- | ------------------------------------------- |
| SIGKILL | 9   | 内核强制终止进程，用于“终极杀死”无响应进程 | `TerminateProcess` API / `taskkill /F /pid`         | 无法捕获/模拟，Shell接收到`trap 'cmd' SIGKILL`时直接报错即可 |
| SIGSTOP | 19  | 内核强制暂停进程，无法被拦截         | `SuspendProcess` API（Windows 10+） / `SuspendThread` | 无法捕获/模拟，Shell接收到`trap 'cmd' SIGSTOP`时报错即可   |

## 二、可捕获信号（21个）

### 1. 退出/中断类（用户高频使用）                                                                                       | -                                                                                    |

| 信号名     | 编号  | 核心作用说明                                          | Windows模拟核心API/机制                                                                                          | Shell实现关键要点                                                                       |
| ------- | --- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| SIGHUP  | 1   | 1. 终端/SSH连接断开；<br>2. 守护进程重载配置（如nginx -s reload） | 1. 控制台关闭：`SetConsoleCtrlHandler` 捕获 `CTRL_CLOSE_EVENT`；<br>2. 自定义重载：命名管道/Event事件（`CreateEvent`/`SetEvent`） | Shell需监听控制台关闭事件，触发`trap`注册的命令；<br> 对“重载配置”场景，可扩展自定义指令模拟                           |
| SIGINT  | 2   | 用户按下Ctrl+C，请求进程中断                               | `SetConsoleCtrlHandler` 捕获 `CTRL_C_EVENT`                                                                  | 核心必实现：按下Ctrl+C时立即执行`trap`注册的清理/提示命令                                               |
| SIGQUIT | 3   | 用户按下Ctrl+\，进程退出并生成core dump（调试用）                | 1. 退出：`ExitProcess` API；<br>2. 核心转储：`MiniDumpWriteDump` API                                                | 可选实现：捕获Ctrl+Break模拟Ctrl+\，执行`trap`命令后退出并生成迷你转储文件；<br> 简易Shell可降级为“仅执行`trap`命令+退出” |
| SIGTERM | 15  | 优雅终止进程（kill默认信号），允许进程清理资源                       | 1. `taskkill /pid`（非强制）；<br>2. 自定义IPC（命名管道/消息）通知进程退出                                                       | 核心必实现：<br>1. 外部终止Shell进程时触发`trap`命令；<br>2. Shell内执行`kill pid`时，向目标进程发送退出通知        |
| EXIT    | 0   | 伪信号：Shell脚本正常/异常退出时触发                           | 脚本`main`函数返回、`ExitProcess` API调用前                                                                          | 核心必实现：脚本退出（无论是否正常）时，优先执行`trap 'cmd' EXIT`注册的命令                                    |

### 2. 程序错误类（Shell脚本极少捕获）

| 信号名     | 编号  | 核心作用说明                | Windows模拟核心API/机制                                                   | Shell实现关键要点                           |
| ------- | --- | --------------------- | ------------------------------------------------------------------- | ------------------------------------- |
| SIGILL  | 4   | 进程执行非法指令（如CPU不支持的指令）  | SEH结构化异常：捕获 `EXCEPTION_ILLEGAL_INSTRUCTION`                         |                                       |
| SIGTRAP | 5   | 调试断点触发（如gdb断点）        | SEH捕获 `EXCEPTION_BREAKPOINT` / `DebugBreak()` API                   |                                       |
| SIGABRT | 6   | 进程主动调用`abort()`触发异常终止 | `abort()` 函数 / `RaiseException` API                                 | 可选实现：Shell内执行`abort`指令时，触发`trap`命令后退出 |
| SIGFPE  | 8   | 浮点错误（除零、溢出）           | SEH捕获 `EXCEPTION_FLT_DIVIDE_BY_ZERO`/`EXCEPTION_INT_DIVIDE_BY_ZERO` |                                       |
| SIGSEGV | 11  | 段错误（非法内存访问，如空指针）      | SEH捕获 `EXCEPTION_ACCESS_VIOLATION`                                  |                                       |

### 3. I/O/网络类

| 信号名     | 编号  | 核心作用说明                 | Windows模拟核心API/机制                                                             | Shell实现关键要点                                                     |
| ------- | --- | ---------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------- |
| SIGPIPE | 13  | 向无读端的管道/套接字写入数据，默认终止进程 | 1. 管道：`WriteFile` 返回 `ERROR_BROKEN_PIPE`；<br>2. 套接字：`send` 返回 `WSAEPIPE`      | 核心实现：<br>1. 监听管道/套接字写入错误码；<br>2. 触发`trap`命令后，按POSIX默认行为终止进程（可选） |
| SIGALRM | 14  | 定时器超时（`alarm()`设置的时间到） | 1. 高精度定时器：`CreateWaitableTimer`/`SetWaitableTimer`；<br>2. 简易版：`SetTimer` + 回调 | 可选实现：Shell内`alarm`命令绑定定时器，超时触发`trap`命令                          |

### 4. 子进程/作业控制类

| 信号名     | 编号  | 核心作用说明                        | Windows模拟核心API/机制                                                                | Shell实现关键要点                                                    |
| ------- | --- | ----------------------------- | -------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| SIGCHLD | 17  | 子进程退出/暂停/继续，用于回收子进程资源（避免僵尸进程） | 1. 同步：`WaitForSingleObject` 等待子进程句柄；<br>2. 异步：`RegisterWaitForSingleObject` 注册回调 | 核心必实现：<br>1. 子进程退出时触发`trap`命令；<br>2. 执行`waitpid`回收资源，模拟POSIX行为 |
| SIGCONT | 18  | 恢复被暂停的进程（如`fg`/`bg`命令）        | `ResumeThread` / `ResumeProcess` API（Windows 10+）                                | 可选实现：Shell内`fg`/`bg`命令触发时，执行`trap`命令后恢复进程                      |
| SIGTSTP | 20  | 用户按下Ctrl+Z，暂停前台进程             | `SetConsoleCtrlHandler` 捕获 `CTRL_BREAK_EVENT` + `SuspendThread`                  | 核心实现：<br>1. 按Ctrl+Break模拟Ctrl+Z；<br>2. 触发`trap`命令后暂停脚本，`fg`恢复  |
| SIGTTIN | 21  | 后台进程尝试读取终端输入                  | 后台进程读控制台返回 `ERROR_ACCESS_DENIED` 错误码                                             |                                                                |
| SIGTTOU | 22  | 后台进程尝试写入终端输出                  | 后台进程写控制台返回 `ERROR_ACCESS_DENIED` 错误码                                             |                                                                |

### 5. 通用/自定义类

| 信号名      | 编号  | 核心作用说明          | Windows模拟核心API/机制                                                   | Shell实现关键要点                                                          |
| -------- | --- | --------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------- |
| SIGUSR1  | 10  | 用户自定义信号，无系统默认行为 | 1. 事件（Event）：`CreateEvent`/`SetEvent`；<br>2. 命名管道：`CreateNamedPipe` | 核心实现：<br>1. 注册自定义IPC通道；<br>2. Shell内`kill -USR1 pid`时，触发目标进程`trap`命令 |
| SIGUSR2  | 12  | 用户自定义信号，无系统默认行为 | 同SIGUSR1（不同Event/管道标识区分）                                            | 同SIGUSR1，仅用不同IPC标识区分                                                 |
| SIGURG   | 23  | 套接字收到带外（紧急）数据   | `WSAAsyncSelect` 注册 `FD_OOB` 事件 / `WSARecv(MSG_OOB)`                | 可选实现：仅网络场景支持，Shell基础版可忽略                                             |
| SIGWINCH | 28  | 终端窗口大小改变        | 1. 轮询：`GetConsoleScreenBufferInfo`；<br>2. GUI程序：`WM_SIZE` 消息        | 可选实现：控制台Shell可轮询窗口大小，变化时触发`trap`命令                                   |

> 我们这只是为了给 Windows 也提供一个 Unix-Like 的终端使用体验
> 完全可以不按照这份文件里的指示的方法来做
> 目标仅仅是行为一致就够了
> 就像是可以进行降级处理, 比如让管道降级为串行处理而不是并行
