## 实验功能

本次实验在一个分时多任务系统架构的基础上实现了一个当前任务信息查询调用。

从系统调用的层面上（S态）实现了该功能。

sys_task_info调用的实现建立在对TaskManager的修改的基础上。

我在TaskManagerInner中的TaskControlBlock里添加了syscall_times来统计该任务的各个系统调用次数的统计（在Trap_handler中增加贡献），并且在Inner中添加start_time来计算Task开启的时间长度，任务块实现函数中统计。

由于本次实验框架基于数组的实现方式，是建立在系统体量较小的基础上，所以能够理解。但是如果系统逐渐庞大，系统调用和任务数量的增加，会导致TaskManager内存部分乘法增长（因为是数组套数组）。

任务数量应该远小于系统调用的情况下使用HashMap来替代TaskControlBlock内部的统计数组会好一些。

## 简答题

1. RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
    1. ch2_bad_address.rs: [kernel] PageFault in application, bad_addr = 0x0, bad instruction = 0x8004003c4, kernel killed it. 由于程序试图访问受保护的地址触发 kill。
    2. ch2b_bad_instructions.rs: IlleaglInstruction in application, kernel killed it. 由于用户程序试图使用特权指令触发报错后被kill。
    3. ch2b_bad_register.rs: IlleaglInstruction in application, kernel killed it. 由于用户程序试图使用特权寄存器被kill。

2. 
    1. a0存储了内核栈的地址，两种情景为：处理完trap后从S态回到U态时; 当批处理操作系统初始化完成，或者是某个应用程序运行结束或出错的时通过__restore函数回到用户态。
    2. 将三个寄存器信息从内核栈中取出。
       (1) t0: sstatus，记录trap前处于哪个特权级，用于判断是否在回到用户态时是否发生错误。
       (2) t1: sepc，记录trap前最后一个指令的地址，用于回到U态时找到应该执行的下一条指令。
       (3) t2: sscratch，记录用户栈地址，用户态使用的数据信息保存在其中。
    3. x2是sp，用于保存内核栈/用户栈地址，在__alltraps中就没有存储在内核栈中，x4(tp)一般不会用到所以也没有保存。
    4. sp重新指向用户栈栈顶，sscratch指向内核栈栈顶。
    5. sret，指令集中给出的S态回到M态的指令。
    6. sp指向内核栈栈顶，sscratch指向用户栈栈顶。
    7. user/src/syscall.rs中调用的ecall指令。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流：组内成员：范天奇、余智超。且仅在实验编程题面的信息上做了交流，并没有涉及任何和编写代码有关的交流。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：rCore-Tutorial-Guide 2023 秋季学期、rCore-Tutorial-Book 第三版和相关线上课程视频。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。