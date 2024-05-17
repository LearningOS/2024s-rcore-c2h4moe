# ch3报告 -- c2h4moe
## 做了什么
开发了一个新的系统调用`sys_task_info(*mut TaskInfo)`，统计每个应用的从调用到现在经过的时间以及发出系统调用的次数。如何统计？我的方法是在任务管理模块内加入了一个新的全局变量APP_INFO，对每个任务都记录了它所有信息，然后该模块对外暴露一个`update_task_syscall_info(syscall_id: usize)`和`get_task_info(buf: *mut TaskInfo)`接口，在syscall模块中在转到对应syscall处理函数之前，会先调用更新syscall情况的函数，保证任务管理模块内所有任务的运行情况都是最新的。如果是task_info系统调用，就使用对应接口就好。
## 问答题
1\) ch2_bad_address程序由于试图往0号地址存储，被pmp机制捕捉引发了store access fault，委托到用户态在`TrapHandler`中被杀死。运行结果：打印出`[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003ac, kernel killed it.`。剩余两个程序均试图调用更高特权级指令，引发illegal instruction被内核杀死，运行结果均为`[kernel] IllegalInstruction in application, kernel killed it.`

2-1\) a0的值是上一个任务的taskcontext的内存位置，使用__restore可能是第一次进入程序，此时sp指向的位置保存了这个任务所有的寄存器，其余初始化为0，sepc初始化为程序entry，sstatus的spp初始化用户态，这样恢复寄存器后sret就开始了任务执行。使用__restore也可能是恢复程序运行，这种情况是平凡的。

2-2\) 处理了sstatus寄存器，sepc和sscratch，处理这几个寄存器可以让sret后程序回到正确位置，以用户态执行，且栈指针与系统调用前一致。

2-3\) x4是线程指针，不需要保存，x2是sp，sp最后是通过与sscratch交换来完成从内核栈到用户栈的切换的，所以无需存储。

2-4\) sp: 指向用户栈; sscratch: 指向内核栈中储存该任务上下文的地址处

2-5\) sret指令后，因为在OS开始运行进行初始化时将每个任务上下文的sstatus中MPP设为了USER

2-6\) 发生异常后交换内核栈和用户栈，该指令后sp指向内核栈中储存该任务上下文的地址，sscratch指向该任务的用户栈

2-7\) 用户态中造成异常的指令或时钟中断

## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我未与他人就（与本次实验相关的）方面做过交流。


2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
《RISC-V-Reader》

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。