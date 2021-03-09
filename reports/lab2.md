# Lab 2 实验报告

## 一、编程内容

1. 在`batch`模块中添加`valid_app_buf`方法，判断待输出的字符串全部内容是否位于用户栈或 app bin 地址范围内；
2. 在`syscall::fs`模块的`sys_write`方法中添加对`buf`和`len`的检查，若不是有效范围则返回`-1`，同时将遇到未支持的文件描述符时的处理方法同样改为返回`-1`；
3. 修改`build.rs`中的`TARGET_PATH`和查找 app 名称时的遍历目录，将编译好的测试 bin 文件放在`user/target/bin`目录下。


## 二、问答题目

1. RustSBI 版本：0.1.0

测例`_ch2_bad_instruction`会导致`rustsbi-panic`，原因为 invalid instruction ，同时 rustsbi 会打印出异常出现时`mepc`寄存器的值，即出错的代码地址，以及出错的具体指令内容。

测例`_ch2_bad_register`会导致`rustsbi-panic`，原因同样为 invalid instruction ， rustsbi 会打印出异常出现是`mepc`寄存器的值以及具体指令内容。

测例`_ch2t_bad_address`不会导致`rustsbi-panic`，但会抛出`StoreFault`异常，而我们的代码在`trap`模块的`trap_handler`中可以处理这一异常，即立刻终止当前 app 的运行，因此 rustsbi 不会报错退出，系统可以继续运行下一个 app 。


2.

2.1 
`__restore`作为一个函数可以接收一个参数，即位于内核栈顶一个`TrapContext`结构体的起始地址，这个`TrapContext`中包含了程序进入用户态后各寄存器的值。刚进入`__restore`时`a0`寄存器存有此地址。
`__restore`可以用于用户态程序由于各种原因进入内核态后，从内核态返回进入内核态前的位置继续进行执行；也可用于一个 app 刚开始执行时初次从内核态进入用户态。

2.2 特殊处理了`sstatus`, `sepc`和`sscratch`的值。其中`sstatus`记录了`srer`后程序应位于的权限级别，`sepc`记录了`sret`后程序应开始执行的指令地址，`sscratch`里保存了`sret`后`sp`寄存器的值，在执行`sret`指令前需要与当时的`sp`值进行交换，从而使`sp`获得正确的值，并在下次进入内核态时通过读取`sscratch`中的值获得内核栈顶的正确地址。

2.3 `x2`寄存器作为 stack pointer 使用，因为执行`sret`前还会进行一些栈操作，可能修改`x2`的值，因此之前已将它暂存在`sscratch`寄存器中，等到`sret`执行前再与其进行交换；`x4`寄存器作为 thread pointer 使用，目前不需要使用它的功能，因此不作处理。

2.4 此时`sp`中包含了`sret`后程序的栈指针地址，`sscratch`中包含了内核栈顶地址，下次进入内核态时即可通过读取`sscratch`中的值找到内核栈顶。

2.5 L64 `sret`指令进行状态切换，状态切换时通过预设的`sstatus`寄存器的值确定切换到哪一特权级别。因为在初始化 app 时我们将`TrapContext`中`sstatus`保存的特权级别手动设为用户态，或在用户态应用程序主动进入内核态时保存了进入前的特权级别，因此目前每次`sret`后都会进入用户态。

2.6 进入内核态时，读取`sscratch`中的值获得内核栈顶地址，同时将进入前的栈指针保存在`sscratch`中。

2.7 用户态应用程序中的`ecall`指令使得程序从U态进入S态。


3.
riscv64支持的中断包括：Supervisor software interrupt, Machine Software interrupt, Supervisor timer interrupt, Machine timer interrupt, Supervisor external interrupt, Machine external interrupt 。

riscv64支持的异常包括：Instruction address misaligned, Instruction access fault, Illegal instruction, Breakpoint, Load address misaligned, Load access fault, Store address misaligned, Store access fault, Environment call from U-mode, Environment call from S-mode, Environment call from M-mode, Instruction page fault, Load page fault, Store page fault 。

通过读取`scause`寄存器的最高位确定进入内核的是中断还是异常。若最高位为1则为中断，最高位为0则为异常。

在陷入内核时，发生陷入时指令寄存器`pc`的值被存入`sepc`，同时`pc`的值被设为`stvec`寄存器中保存的处理函数地址；`scause`寄存器根据陷入类型进行设置，`stval`寄存器被设置成出错地址或其他特定的异常信息；将`sstatus`寄存器的`SIE`位置零从而屏蔽中断，置零前`SIE`的值被保存在`SPIE`中；陷入发生时的权限模式被保存在`sstatus`的`SPP`域中。通常情况下`sstatus`寄存器保存内核栈顶地址，可与`sp`寄存器交换用以保存陷入时用户栈指针。


4.
对于一些不可能返回原执行处的中断或异常，比如 Illegal Intruction ，可以不保存寄存器的值，直接将栈指针移动相应的大小。因此在陷入时可以通过读取`scause`寄存器的值进行不同的处理。

