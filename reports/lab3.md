## 实验功能

本次实验继承了ch4中的所有功能。

同时实现了spawn创建进程以及通过设置priority优先级和stride参数来对进程进行调度。

## 简答题

1. 不是轮到p1运行，因为在p2的stride更新时发生了溢出，又由于是8bit无符号整数，所以p2.stride一定小于等于p1.stride，所以大概率还是p2运行。

2. STRIDE的最大变动值为BIG_STRIDE/2，无论如何都是最小的stride变动，所以无论如何值域不可能超过BIG_STRIDE/2。

3. partial_cmp

    ~~~rust
        use core::cmp::Ordering;
        struct Stride(u64);
        impl PartialOrd for Stride {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                if self.0 < other.0 {
                    if other.0 - self.0 > BIG_STRIDE / 2 {
                        Some(Ordering::Less)
                    } else {
                        Some(Ordering::Greater)
                    }
                } else {
                    if self.0 - other.0 > BIG_STRIDE / 2 {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
            }
        }
        impl PartialEq for Stride {
            fn eq(&self, other: &Self) -> bool {
                false
            }
        }
    ~~~


## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流：组内成员：范天奇、余智超。且仅在实验编程题面的信息上做了交流，并没有涉及任何和编写代码有关的交流。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：RISC-V 手册、rCore-Tutorial-Guide 2023 秋季学期、rCore-Tutorial-Book 第三版和相关线上课程视频。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。