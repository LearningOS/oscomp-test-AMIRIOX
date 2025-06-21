# 25sp操作系统训练营四阶段技术总结报告

## 0x0 序

非常幸运有这样一个机会能在实践中理解操作系统以至于真真正正用代码写出操作系统的各种功能的, 对我来说这个训练营也在竞赛退役后的迷茫中给了我方向, 让我有了新的目标去追求.

首先要感谢举办训练营、设计lab、课程讲解的各位老师, 我从这些精心设计的课程中受益匪浅;    
同时, 由于初次接触操作系统(以及相关形式的训练营)多少感到有些迷茫, 我也要感谢为我指点方向的陈老师, 凌晨耐心我为答疑的郑老师, 还有在二阶段认识的某位指出了我一个很唐的实现错误的同学.

## 0x1 四阶段主要的工作/我的收获

在项目一(宏内核)中, 由于初次接触操作系统并且在实际项目上的工作经验较少, 我在四阶段还是主要以学习和做一些小任务为主. 我选择了完成内核测例并且在这个过程中了解 POSIX 标准、libc 实现和平时常见的功能之下所需要的内核功能及系统调用.

除此之外, 我也初步了解了实际项目开发中的流程, 例如测试驱动开发, GitHub CI/CD Workflow, 以及在群聊中了解到的实际项目中工具链依赖和维护等等.

## 0x2 具体的实现内容

### 完善系统调用:

1. `sys_unlink`
2. `fscanf` 测例
	* pipe 的 `read` (POSIX read 标准):
        * 管道中无数据, 写端已关闭: 返回 EOF
            * 管道中无数据, 写端未关闭: 说明可能还有数据将要达到, 应当阻塞, yield 或者 spin
            * 管道中有数据: 尽可能多地读完并返回大小
   
    * syscall `writev` 的参数 `iov: *const ctypes::iovec` 中莫名会多一个 `base=0, len=0` 的元素, 然后访问 `0x0` 地址导致 `BadAddress`, 暂时未发现原因, 先 `continue` 以后再说
    * 动态分发的类型擦除经常难以调试()
3. 实现 `ungetc` 测例: 新增 `sys_readv` 系统调用
3. 实现 `fflush_exit` 测例:
    * 修改 close: 允许 close 系统调用关闭 `Stdout`
    * `dup` 系统调用 `get_file_like(old_fd)` 并添加到 `FD_TABLE` 最小的空闲 `fd` 中
    * 新增系统调用 `pread64`: 原子化读入 `fd` 中 offset 处的内容进入 `buf`, 但是不改变 `fd` 的 offset 计数. 记录原来的 offset, 读入后再恢复即可. 不能一开始就 .lock() 否则若调度走可能会造成死锁, 每个原子操作 .lock() 即可
3. 通过`fgetc_buffering` 和 `rewind_clear_error` 测例: 在 `dup2` 系统调用中检查旧的 `fd` 如果被打开就强制 close. 另外 x86_64 需要显示单独条件编译的 `Sysno::dup2`
*  通过 `rlimit_open_files`: 在 `ProcessData` 中添加 `rlimit` 结构体, 在 `get/setrlimit` 系统调用中维护, 然后每次 `openat`/`dup` 时都检查一下当前进程的 `rlim_cur` 是否符合要求. loongarch64 没有 `setrlimit` 和 `getrlimit` 的系统调用号, 所以 libc 在 loongarch64 上的实现是需要 prlimit64 的, 实现这个系统调用然后包装一下 `sys_rt_getrlimit` 和 `sys_rt_setrlimit` 即可

### 小任务

1. 将 `oscamp/arceos` 的工具链更新到 `nightly-2025-05-20`
1. 将 `arceos-hypervisor/arceos` 的工具链更新到 `nightly-2025-05-20`, 并准备查看"第二个任务"(合并 oscamp 和 hypervisor 两个开发方向的代码)
1. 生成 `kernel_guard` 的 deep_wiki 页面 ~~这也太小了~~

## 0x3 未完成的功能与后续计划

### 信号系统

先是从 `man 7 signal` 获得 Linux 的几个信号, 但是发现是字典顺序不是编号顺序, 而操作系统和 `libc` 是靠编号约定的, 所以肯定不行, 又去 `/usr/include/asm-generic/signal.h` 找到了信号的编号, 剔除保证兼容性的重复旧信号; 折腾了很久通过 `enum Signal` 生成 `bitflags SigMask` 的宏

在 `TaskExt` 里加了 `pending: VecDeque<Signal>` 和 `blocked: SigMask`, 但是我发现目前已有接口只能获得 `current task` 的 `task_ext`，包括 `Thread` 里也只有 `tid`（并且没发现有 `tid` 到 `task` 的映射）

于是我试图通过 static weak map 实现 `tid` 到 `task_ext` 的全局静态映射，但是发现所有的 `TaskExt` 都是不被暴露出来的，进一步发现是因为 `TaskInner` 的所有权在任务调度队列中

于是陷入了问题: 我该如何维护线程的 `pending` 信号和阻塞的信号掩码呢？最终解决方案是:   
在 `TaskExt` 的 `ThreadData` 和 `ProcessData` 中加入 Signal 的 `pending` 队列和 `shared` 进程级别的待定信号队列; 

在每次用户态->Trap进内核态->从内核态返回中"从内核态返回"前进行信号处理操作: 在 `axhal` 不同架构的相关 handle trap 函数里加一个 `post_trap_callback`, 而这个函数借助 `linkme` 收集各种 callback 函数并逐个调用(其中就有检查 `from_user` 并且 `check_signals`  的函数). `check_signal` 从信号队列中拉出一个未被阻塞的, 然后匹配对应的 actions.

学习了 `sys_futex` 期待的基本行为: `WAIT` 操作是如果提供的 `val` 相等则令一个线程 yield 走直到超时/中断/`futex wake`, WAKE 操作不管线程是否还需 yield 直接唤醒

试着写了下但是感觉比上面说的复杂, 涉及到 `futex` 自己的 wait queue 等等     
后面又发现每次 post trap 时具体的 `handle_signal` 处理逻辑有问题, 又大改

### 部分改动量比较大的未实现的系统调用

* 尝试实现stat, 调用 `FileLike` trait 的 `get_attr()` 获取文件元信息, 但是 `uid` 和 `gid` 的逻辑没有想好怎么写, 如果要扩展原信息结构体可能要把 crate 拉到本地打 patch
* 试图开 `dlopen`, 但是没搞懂执行流程, 感觉依赖的 syscall 已经实现得差不多了, 而且还找不到那个 `unsupported` 哪里报的, `glibc` 源代码都翻了一遍, 遂放弃; 然后开下一个是线程取消, 发现需要实现信号处理之类的, 任务量也不小, 于是重新看了下 Starry 管理任务的数据结构就收工了

### 后续计划

由于六月六级备考+期末考试等事情较多, 没有充足的时间完成剩下的一百多分的测例了; 

我计划参考一些前辈的实现写出信号系统和 `sys_futex` 并实现 `pthread_cancel_point`;

感谢陈老师的点拨, 我决定去认真读一读 OSTEP

也希望下次训练营时能成长到能实际实现点什么的程度
