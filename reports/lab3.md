# Lab 3 实验报告

## 一、编程内容

1. 由于目前未实现内存动态分配，无法直接使用 rust 提供的`BinaryHeap`数据结构，因此手动实现基于数组，限定最大容量的`BinaryHeap`；
2. 利用`BinaryHeap`实现 stride schedule 算法，每个 app 对应一个`SchedBlock`结构体，结构体内保存`id`，`stride`及`pass`，基于`stride`进行比较，可处理整型溢出；
3. 修改`task`模块，使`TaskManager`可以利用`StrideScheduler`获取下一个被调度的 app 的 id 信息；
4. 编写`set_priority`系统调用，使其可以修改当前任务`SchedBlock`中的`pass`值，修改`get_time`系统调用使其满足测例要求；
5. 重新添加 lab2 中使用的 buf 地址检查（`write`和`get_time`都会使用），在`config`模块中设定单个程序最大运行时间，若超过则砸`TaskManager`中直接终止该任务。


## 二、问答题目

1. 出现时钟中断，应用程序主动进行`yield`系统调用，以及一个应用程序自愿或被操作系统强制退出时都会进行调度。进程信息被保存在一个数组中，每次进行调度时将数组作为一个环形队列，从当前进程开始向后查找，直到找到一个可以运行的进程，或遍历整个队列后发现没有可以运行的进程。新加入的进程会被放在数组尾部。

2. 
(2-1) 不相同。C版调度策略新的进程插入队列的位置是在数组中的第一个空位，但当前进程的指针并不一定位于数组起始位置，因此插入时不一定是在队尾插入。

(2-2)
| 时间点   | 0              | 1      | 2      | 3      | 4      | 5      |
| -------- |--------------- | ------ | ------ | ------ | ------ | ------ |
| 运行进程 |                | p1     | p2     | p3     | p5     | p4     |
| 事件     | p1、p2、p3产生 | p1结束 | p4产生 | p5产生 | p5结束 | p4结束 |

如上表所示，p1 先执行并结束后，p2 执行期间产生的新进程 p4 会被放在原 p1 的位置；下一次调度会执行 p3 ，期间产生的新进程 p5 会被放在 p3 之后。因此接下来两次调度会先执行后创建的 p5 ，后执行先创建的 p4 。

在 stride 算法下运行顺序不确定，因为新加入的所有进程`stride`值相同，选取哪一个进程取决于`BinaryHeap`的实现方式。


3. 
- 实际情况并不是 p1 执行，因为 p2 的`stride`值加上`pass`后出现整型溢出，数值变为4，依然小于 p1 的`stride`值。

- 假设 STRIDE_MAX - STRIDE_MIN > BigStride / 2, 设 p1.stride = STRIDE_MAX，p2.stride = STRIDE_MIN，则 p1 上一次执行时，p1.stride = STRIDE_MAX - p1.pass。由于进程优先级 >= 2，则 p1.pass <= BigStride / 2，那么 p1 上一次执行时，p1.stride > p2.stride，也就是说 p1 上一次执行前 p2 就该被执行，出现矛盾。

-
```
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            if other.0 - self.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else if self.0 > other.0 {
            if self.0 - other.0 <= BIG_STRIDE / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        } else {
            None
        }
    }
}
```
