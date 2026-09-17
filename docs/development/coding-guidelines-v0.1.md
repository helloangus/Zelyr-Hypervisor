# Zelyr Hypervisor Coding Guidelines v0.1 — Detailed Reference

> **Use:** This is the complete, topic-indexed reference. Every Coding Agent
> must first read [the mandatory concise guide](coding-guidelines.md), then
> read the relevant sections of this document when its task triggers them.
> The concise guide does not weaken this document's requirements.

**Status:** Mandatory Coding Standard  
**Audience:** Human developers, Coding Agents, code-review agents  
**Scope:** Hypervisor、架构后端、平台代码、驱动、虚拟设备、Control/Service Domain 公共 Rust 代码  
**Non-goal:** 本文不是架构设计规范。设计、模块边界、对象关系、函数语义、协议及行为以对应设计文档和 ADR 为准。

---

# 1. 基本执行原则

Coding Agent 在实现任务时必须遵循以下优先级：

```text
具体任务说明
    >
对应模块详细设计
    >
接口 / ABI / 状态机规范
    >
Architecture Decision Record
    >
本 Coding Guidelines
    >
个人编码偏好
```

如果较高优先级文档已经明确某个实现方式，不得以“更优雅”“更高性能”“更 Rust idiomatic”等理由自行改变设计。

Coding Agent 的职责是：

> 把已经确定的设计，以正确、安全、可维护、可验证的 Rust 代码实现出来。

不得擅自：

- 改变架构；
- 改变模块职责；
- 改变公开 ABI；
- 改变状态机；
- 增加未经设计的全局状态；
- 引入新的核心抽象；
- 替换设计指定的数据结构；
- 改变锁模型；
- 改变内存 ownership 规则；
- 改变错误语义；
- 自动扩展任务范围。

发现设计存在明显矛盾时，应停止该冲突部分的实现，并明确指出矛盾，而不是自行决定新的架构。

---

# 2. Rust 基本规范

## 2.1 Rust-first

除以下场景外，代码必须使用 Rust：

- CPU reset/entry；
- exception vector；
- guest entry/exit；
- context switch 中 Rust 无法可靠表达的部分；
- 明确需要特定机器指令且 `core::arch` 无适当 intrinsic；
- ABI 要求的极小 trampoline。

不得仅因为“汇编更快”而使用汇编。

---

## 2.2 `no_std`

EL2 Hypervisor crate 默认：

```rust
#![no_std]
```

只有明确属于 host tool、Control Domain userspace、测试工具等组件才允许依赖 `std`。

不得为了使用某个方便的库而使 Hypervisor Core 依赖 `std`。

---

## 2.3 Rust edition 与工具链

必须使用仓库固定的：

```text
rust-toolchain.toml
Cargo.toml
rustfmt.toml
clippy.toml
```

不得自行升级：

- Rust edition；
- nightly revision；
- LLVM；
- target spec；
- 关键依赖版本。

除非任务明确要求。

---

## 2.4 Unstable feature

不得无理由添加：

```rust
#![feature(...)]
```

增加 nightly feature 必须同时满足：

1. 当前设计确实需要；
2. stable Rust 无合理实现；
3. feature 使用被限制在最小模块；
4. 在代码附近说明使用原因。

不得因为减少几行代码而引入 unstable feature。

---

# 3. 格式和命名

所有代码必须通过：

```text
cargo fmt
cargo clippy
```

仓库另有参数时以仓库 CI 为准。

命名遵守：

```text
Type / Trait           UpperCamelCase
function / variable    snake_case
module                  snake_case
const                   SCREAMING_SNAKE_CASE
static                  SCREAMING_SNAKE_CASE
feature                 kebab-case
crate                    kebab-case
```

不要创建无意义缩写。

允许的领域缩写包括：

```text
vm
vcpu
irq
gic
mmio
dma
iommu
smmu
ipa
gpa
hpa
gva
tlb
asid
vmid
psci
pci
msi
```

例如推荐：

```rust
guest_phys_addr
stage2_page_table
pending_irqs
current_vcpu
```

不推荐：

```rust
gpa_addr
s2ptbl
pirqv
curv
```

---

# 4. 文件和模块规则

每个模块必须具有明确单一职责。

禁止形成：

```text
utils.rs
helpers.rs
common.rs
misc.rs
platform_stuff.rs
```

这种无限增长的杂物模块。

如果存在 utility，应按实际语义命名，例如：

```text
bitmap.rs
intrusive_list.rs
address.rs
checked_range.rs
volatile.rs
```

单个源文件不应无限增长。

当一个文件已经包含多个明显独立职责时，应按照设计边界拆分，而不是继续追加代码。

但不得仅为减少文件长度而拆出大量没有独立语义的小文件。

---

# 5. 依赖方向

Coding Agent 不得通过代码方便性破坏 crate/module 依赖关系。

例如：

```text
core
```

不得为了调用某个平台函数而：

```rust
use hv_board_orangepi3b::...
```

架构无关代码不得引用：

```text
AArch64 system register
VMCS
GIC register
RK3566 MMIO address
QEMU-specific constant
```

具体依赖方向由架构文档确定。

如果实现过程中发现必须形成反向依赖，应视为设计冲突，而不是通过：

```rust
cfg
feature
global callback
unsafe extern
```

绕开。

---

# 6. 地址类型

绝对禁止在 Hypervisor Core 中用裸：

```rust
usize
u64
```

混合表达不同地址空间。

至少应使用独立 newtype：

```rust
HostPhysAddr
HostVirtAddr
GuestPhysAddr
GuestVirtAddr
```

或设计文档指定的等价类型。

例如：

```rust
struct GuestPhysAddr(u64);
struct HostPhysAddr(u64);
```

不得写：

```rust
fn map(addr: usize, host: usize)
```

应该写成语义明确的接口：

```rust
fn map(
    guest_addr: GuestPhysAddr,
    host_addr: HostPhysAddr,
    ...
)
```

不同地址类型不得隐式转换。

---

# 7. 长度、大小、页号与地址不能混用

以下概念必须使用不同类型或明确命名：

```text
byte address
byte length
page count
page frame number
page offset
CPU ID
VM ID
vCPU ID
IRQ number
object handle
```

不得写：

```rust
let x: usize;
```

然后根据上下文猜测 `x` 的语义。

推荐：

```rust
PageCount
PageFrameNumber
CpuId
VmId
VcpuId
IrqNumber
```

对于核心安全路径，应优先 newtype。

---

# 8. 数值运算规则

所有由以下来源产生的整数：

- Guest；
- Management Domain；
- Device descriptor；
- Firmware；
- DTB；
- PCI config；
- MMIO register；

都视为不可信输入。

地址和长度计算必须优先使用：

```rust
checked_add
checked_sub
checked_mul
checked_shl
checked_next_power_of_two
```

不得在不可信输入上直接：

```rust
base + size
count * entry_size
offset + len
```

然后再检查结果。

正确顺序是：

```text
输入
→ 合法范围验证
→ checked arithmetic
→ alignment 检查
→ 最终转换
→ 使用
```

---

# 9. 类型转换

禁止滥用：

```rust
as
```

尤其禁止：

```rust
u64 as usize
usize as u32
i64 as u64
```

用于可能截断的数据。

优先：

```rust
usize::try_from(...)
u32::try_from(...)
```

只有明确无损的转换，或者硬件寄存器语义要求时才允许 `as`。

关键转换附近应让不变量明显。

---

# 10. `unsafe` 总原则

默认：

> 所有 Rust 代码都是 safe Rust，只有无法用 safe Rust 正确表达的最小边界允许 `unsafe`。

不得为了：

- 绕过 borrow checker；
- 减少 clone；
- 避免设计生命周期；
- 快速共享全局数据；
- 简化并发；

而使用 `unsafe`。

---

# 11. 每个 `unsafe` 必须解释

所有非显而易见的：

```rust
unsafe { ... }
```

之前必须有：

```rust
// SAFETY:
```

解释调用成立所依赖的不变量。

例如：

```rust
// SAFETY:
// - `ptr` points to a mapped MMIO register block.
// - The register block remains mapped for the lifetime of `self`.
// - Access is serialized by `self.lock`.
unsafe {
    ...
}
```

禁止无信息量注释：

```rust
// SAFETY: this is safe.
```

---

# 12. unsafe function

如果函数本身要求调用者维持不变量，应明确：

```rust
unsafe fn ...
```

并在文档中添加：

```rust
/// # Safety
```

说明所有前置条件。

不要为了减少 `unsafe` 块而把大段 API 标成 `unsafe fn`。

---

# 13. unsafe 封装

优先结构：

```text
small unsafe primitive
        ↓
safe checked wrapper
        ↓
normal code
```

例如：

```text
raw system register access
        ↓
typed register wrapper
        ↓
Stage2AddressSpace
```

上层业务代码不应反复直接调用裸指针或裸寄存器接口。

---

# 14. 裸指针规则

`*const T` / `*mut T` 应只存在于明确需要的模块。

不得长期保存来源不清晰的裸指针。

如果对象具有稳定生命周期，应优先：

```rust
NonNull<T>
```

配合封装类型。

不得使用裸指针绕过资源 ownership。

---

# 15. `static mut`

禁止：

```rust
static mut ...
```

除非：

- 极早期启动代码；
- CPU entry 前；
- 没有并发；
- 无合理替代方案。

一旦 SMP 或 normal runtime 开始，应转换为明确的同步对象。

优先：

```text
Once
OnceLock-like no_std primitive
SpinLock
Atomic*
per-CPU data
```

具体根据设计使用。

---

# 16. `transmute`

默认禁止：

```rust
core::mem::transmute
```

除非 ABI/硬件布局明确要求，且：

- source/target size 已验证；
- alignment 已验证；
- validity invariant 已证明；
- 有 SAFETY 注释。

优先：

```text
from_bits
try_from
pointer cast
MaybeUninit
```

等语义更明确的方式。

---

# 17. `MaybeUninit`

允许用于：

- boot-time object construction；
- interrupt stack/context；
- CPU-local storage；
- DMA descriptors；
- ABI buffers。

但初始化状态必须明显。

禁止通过：

```rust
assume_init()
```

掩盖部分初始化问题。

---

# 18. `ManuallyDrop`

仅用于确实需要控制析构时机的底层资源。

不得用于逃避 Rust ownership。

如果对象需要特殊生命周期，优先定义明确状态机。

---

# 19. 数据结构必须体现不变量

不要让所有对象都退化成：

```rust
struct Foo {
    flags: u64,
    state: u8,
    id: usize,
    ptr: usize,
}
```

应该尽量让类型本身表达语义。

例如：

```rust
enum VcpuState {
    Offline,
    Runnable,
    Running,
    Blocked,
    Paused,
    Stopped,
    Faulted,
}
```

优于：

```rust
const VCPU_RUNNING: u8 = 3;
```

---

# 20. 状态机

设计中定义的状态机必须显式实现。

不得通过随意修改字段：

```rust
self.state = State::Running;
```

绕过状态迁移规则。

推荐：

```rust
self.transition_to(VcpuState::Running)?
```

状态迁移函数负责：

- 验证旧状态；
- 更新附属资源；
- 处理 accounting；
- trace；
- 维护 invariant。

非法状态迁移返回错误或触发内部 invariant failure，具体按设计。

---

# 21. ID 与 Handle

内部索引、公开 ID、Capability Handle 不得混用。

例如：

```text
VmId
ObjectSlot
CapabilityHandle
Generation
```

必须明确区分。

外部提供的 handle 不得直接作为数组索引使用。

流程必须类似：

```text
Handle
→ range check
→ generation check
→ rights check
→ object lookup
```

---

# 22. Generation

可复用 object slot 必须带 generation。

禁止仅通过：

```rust
u32 index
```

作为长期有效句柄。

对象销毁后对应 generation 必须失效旧引用。

---

# 23. Option 和 Result

不存在是正常状态：

```rust
Option<T>
```

操作可能失败：

```rust
Result<T, E>
```

不要通过特殊数字编码：

```rust
usize::MAX
0xffff_ffff
```

表示无对象，除非硬件 ABI 明确如此。

---

# 24. 错误类型

错误必须具有语义。

建议至少区分：

```text
InvalidInput
PermissionDenied
NotFound
AlreadyExists
InvalidState
ResourceExhausted
Unsupported
GuestFault
HardwareError
Timeout
Busy
InvariantViolation
```

不得把所有错误统一为：

```rust
Err(())
```

核心内部也不得滥用字符串错误。

---

# 25. panic 规则

普通 Guest 可触发的输入绝对不得导致 Hypervisor panic。

例如：

```text
非法 HVC
坏 virtio descriptor
Stage-2 fault
非法 MMIO
错误 PSCI 参数
未知系统寄存器访问
```

必须转化为：

- Guest fault；
- ABI error；
- injected exception；
- device error；
- VM termination；

由具体设计决定。

`panic!` 只应用于真正的：

> Hypervisor 内部 invariant 已经被破坏，继续执行不安全。

---

# 26. `unwrap` / `expect`

Hypervisor runtime path 默认禁止：

```rust
unwrap()
expect()
```

以下场景可接受：

- test；
- build script；
- 明确编译期常量；
- 初始化后由结构证明不可能失败的内部路径。

即使使用 `expect`，信息也必须说明 invariant：

```rust
.expect("boot CPU must initialize GLOBAL_PLATFORM before AP startup")
```

而不是：

```rust
.expect("failed")
```

---

# 27. `todo!` / `unimplemented!`

不得在正常可达 runtime path 留：

```rust
todo!()
unimplemented!()
```

如果当前 feature 尚未实现：

```rust
Err(Error::Unsupported)
```

或明确拒绝启用。

开发阶段临时占位必须带：

```text
TODO(issue-id):
```

并确保不会被正常配置触达。

---

# 28. MMIO

不得直接对 MMIO 地址使用普通引用读写。

必须通过明确 volatile abstraction：

```text
read_volatile
write_volatile
```

或项目统一 MMIO wrapper。

MMIO wrapper 必须表达：

- base；
- register offset；
- register width；
- access permission；
- memory ordering requirement。

不得构造指向设备寄存器的普通：

```rust
&mut T
```

并依赖 Rust 普通内存语义。

---

# 29. 硬件寄存器

寄存器位定义应：

- 使用命名常量；
- 或 bitflags；
- 或 typed register wrapper。

不要大量使用：

```rust
reg |= 1 << 17;
reg &= !(3 << 6);
```

而应定义语义名称。

例如：

```rust
HCR_VM
HCR_RW
```

具体命名遵循架构文档。

---

# 30. Reserved bits

写硬件寄存器时必须遵守文档定义的 RES0、RES1、preserve bits。

不得：

```rust
write_reg(0);
```

除非规范明确允许。

如果寄存器要求 read-modify-write，应实现统一 helper。

---

# 31. Barrier

不得凭经验随意增加或删除：

```text
DMB
DSB
ISB
TLBI
```

Barrier 必须对应明确硬件语义。

在代码中应使用项目统一封装，例如：

```rust
barrier::dsb_ish();
barrier::isb();
```

不要到处嵌入裸 asm。

---

# 32. TLB

所有改变：

- Host page table；
- Stage-2；
- permissions；
- address-space identity；

的代码必须明确考虑：

```text
table write visibility
TLB invalidation
barrier ordering
cross-CPU shootdown
```

不得因为 QEMU 中“看起来能工作”就省略。

---

# 33. 汇编

汇编必须：

- 尽量短；
- 明确输入输出；
- 不隐藏 Rust 看不到的副作用；
- 正确声明 clobber；
- 不依赖未声明寄存器状态。

大型流程不得全部写成汇编。

推荐：

```text
asm entry
→ save minimal context
→ Rust handler
→ asm restore/eret
```

---

# 34. Exception context

异常上下文结构必须与汇编保存顺序一一对应。

如果改变：

```rust
struct ExceptionFrame
```

必须同步检查：

- assembler offsets；
- stack alignment；
- restore sequence；
- debugger/crash dump。

禁止单独修改其中一侧。

---

# 35. ABI 数据结构

跨：

- EL2 ↔ Guest；
- Hypervisor ↔ Control Domain；
- Hypervisor ↔ Service Domain；
- migration stream；
- snapshot；

的数据结构不得直接暴露普通 Rust 内存布局。

除非设计明确要求，否则不能假定：

```rust
struct Foo
```

就是 wire format。

---

# 36. `repr`

ABI 结构必须显式：

```rust
#[repr(C)]
```

或项目指定 wire layout。

位级协议不要依赖 Rust enum 默认表示。

enum 若进入 ABI，应：

```rust
#[repr(u32)]
```

并定义未知值处理策略。

---

# 37. ABI padding

不得把未初始化 padding 暴露到其他 protection domain。

输出 ABI buffer 前必须：

- zero initialize；
- 或逐字段序列化。

禁止直接发送含未初始化 padding 的 Rust struct。

---

# 38. Endianness

wire format、device protocol、PCI、virtio 等涉及字节序时必须显式处理。

禁止默认认为：

> 当前 AArch64 是 little-endian，所以所有结构都可 native read。

应使用：

```text
from_le_bytes
to_le_bytes
from_be_bytes
to_be_bytes
```

或规范指定 helper。

---

# 39. ABI 兼容

不得：

- 调整公开 ABI 字段顺序；
- 修改字段含义；
- 复用保留字段；
- 更改 enum 数值；
- 更改 hypercall number；

除非对应 ABI 设计文档明确更新版本。

Coding Agent 不得自行“清理 ABI”。

---

# 40. Feature flag

Cargo feature 表示：

> 当前 binary 是否包含某能力。

不得用 feature 表示运行时实例状态。

错误：

```text
vm-count-4
guest-memory-1g
control-domain-two-vcpu
```

合理：

```text
gicv3
smmuv3
virtio
tracing
pci
```

具体 feature 名以项目设计为准。

---

# 41. `cfg`

`#[cfg]` 应只出现在合理边界。

例如：

```text
architecture implementation
optional feature implementation
test-only code
```

Hypervisor Core 不得出现：

```rust
#[cfg(feature = "orangepi3b")]
```

或：

```rust
if cfg!(feature = "qemu") {
    ...
}
```

来实现业务逻辑。

---

# 42. 平台代码

平台差异只能出现在设计允许的：

```text
Arch
SoC
Board
Driver
Firmware
Quirk
```

层。

不得将平台特判扩散到：

```text
VM
scheduler
memory manager
virtio
capability
management
```

等核心模块。

---

# 43. Driver probe

设备发现与 driver 匹配必须基于：

```text
DT compatible
PCI vendor/device/class
ACPI identifier
明确 capability
```

不得在通用 driver 内判断：

```rust
if board_name == ...
```

---

# 44. Quirk

硬件 quirk 必须：

- 有明确名字；
- 有硬件/firmware 原因注释；
- 限制到最小平台；
- 不改变其他平台行为。

推荐：

```rust
apply_rk3566_xxx_quirk(...)
```

而不是：

```rust
if weird_platform {
    ...
}
```

---

# 45. 并发基本规则

SMP 启用后，任何全局可变状态都必须明确回答：

> 谁可以访问它？  
> 在什么上下文？  
> 使用什么同步机制？  
> 是否允许 IRQ context？  
> 是否允许同时被多个 CPU 修改？

如果无法回答，不得提交。

---

# 46. Lock

锁必须有明确 ownership 和作用域。

禁止：

- 持锁执行长时间设备 I/O；
- 持普通锁进入 Guest；
- 持锁调用可能 block 的 IPC；
- 持锁调用未知 callback；
- 在 IRQ context 获取可能等待的锁。

如果设计存在 lock hierarchy，应严格遵守。

---

# 47. Lock scope

使用最小锁范围：

```rust
{
    let mut guard = lock.lock();
    ...
}
```

不要让 guard 因函数过长而不必要地保持。

必要时显式：

```rust
drop(guard);
```

使生命周期明显。

---

# 48. Atomics

不得因为“无锁更快”而随意使用 atomics。

只在数据模型适合时使用：

```text
flags
counters
single-word state
producer/consumer index
```

复杂对象 ownership 不应依赖大量独立 atomic 字段拼装。

---

# 49. Memory Ordering

禁止默认全部使用：

```rust
Ordering::SeqCst
```

也禁止默认全部：

```rust
Relaxed
```

每个非显而易见的 atomic ordering 应能解释 happens-before 关系。

关键 lock-free 算法附近应注释：

```text
Acquire synchronizes with Release on ...
```

---

# 50. Interrupt context

可能在中断上下文调用的函数必须明确标记其限制。

IRQ handler 不得：

- sleep；
- 等待普通 mutex；
- 做大规模 allocation；
- 执行复杂日志格式化；
- 访问可能 fault 的 Guest buffer；
- 进行长耗时扫描。

IRQ handler 应尽量：

```text
ack
record
enqueue
notify
return
```

---

# 51. per-CPU data

CPU-local 数据不得通过普通可变全局变量模拟。

必须通过统一 per-CPU abstraction 获取。

不得缓存会在 CPU migration 后失效的 per-CPU 引用到长期对象中。

---

# 52. 生命周期与引用

不得让：

```rust
&'static T
```

成为绕过生命周期设计的工具。

`'static` 只用于真正拥有静态生命周期的数据。

长期对象关系优先使用：

```text
stable handle
Arc-like ownership
intrusive ownership
object table
```

具体按设计要求。

---

# 53. `Arc`

Hypervisor Core 不应因为方便而普遍使用：

```rust
Arc<Mutex<T>>
```

尤其不能把它当默认对象模型。

使用共享所有权前必须符合具体设计。

资源 ownership、VM ownership、页 ownership 应由明确模型表达。

---

# 54. Clone

大型状态对象不得随意：

```rust
#[derive(Clone)]
```

尤其：

```text
Vm
Vcpu
AddressSpace
Device
Capability
MemoryObject
```

这类身份对象通常不应该语义上可复制。

只有“值对象”才自然适合 Clone。

---

# 55. Copy

必须非常谨慎地为类型实现：

```rust
Copy
```

适合：

```text
address newtype
small IDs
flags
immutable descriptor
```

不适合具有 ownership 或状态的对象。

---

# 56. Drop

底层资源类型应在合适情况下利用 `Drop` 保持局部资源安全。

但不得把关键 VM 生命周期行为隐藏在难以观察的 Drop 中。

例如：

```text
stop VM
flush DMA
revoke capability
```

这种复杂操作应由显式 lifecycle API 完成。

---

# 57. Allocation

高频 fast path 中不得无意识 allocation。

尤其：

```text
VM-exit
IRQ
virtqueue processing
scheduler
TLB shootdown
```

优先：

- 预分配；
- object pool；
- fixed-size local buffer；
- per-CPU cache；

具体由设计确定。

---

# 58. Collection

选择容器必须基于访问模式。

不要默认所有东西都用：

```rust
Vec
BTreeMap
HashMap
```

考虑：

```text
固定最大数量
按 ID 索引
顺序遍历
插入删除频率
IRQ context
allocation requirement
确定性
```

如果最大对象数量由架构天然有限，固定数组可能优于动态 map。

---

# 59. HashMap

EL2 中不要随意使用普通 HashMap。

原因包括：

- allocation；
- hash DoS；
- 不确定执行时间；
- 依赖复杂度。

只有设计明确需要时使用。

---

# 60. 递归

Hypervisor Core 默认避免递归。

特别是：

- Guest-controlled descriptor；
- page-table traversal；
- device tree；
- capability graph；

不应使用无界递归。

优先显式 stack/iterator，并设深度限制。

---

# 61. Guest 输入

所有 Guest 输入必须被当作攻击面。

包括：

```text
register
HVC
MMIO
PIO（x86）
sysreg trap
virtio descriptor
PCI config
shared memory
guest-owned page table-related metadata
```

必须：

```text
validate before use
```

而不是使用以后再检查。

---

# 62. Guest pointer

不得直接把 Guest 地址转换为 Host pointer：

```rust
let ptr = gpa as *mut T;
```

必须通过统一 Guest memory access abstraction。

例如：

```text
GuestAddressSpace
GuestMemory
copy_from_guest
copy_to_guest
map_guest_slice
```

具体接口按设计。

---

# 63. Guest slice

对于 Guest 提供：

```text
address + length
```

必须至少验证：

- overflow；
- mapping；
- permission；
- page boundary；
-最大长度；
- ownership；
- 读写方向。

不得因为 buffer 位于 Guest RAM 就默认安全。

---

# 64. Virtio descriptor

Virtqueue descriptor walker 必须防御：

- descriptor loop；
- chain too long；
- invalid next；
- integer overflow；
- overlapping buffers；
- invalid GPA；
- writable/readable direction violation；
- indirect descriptor abuse；
- excessive total length。

不得相信 Guest driver 正确。

---

# 65. DMA

标记为 DMA-pinned 的内存：

- 不得回收；
- 不得迁移；
- 不得取消映射；
- 不得重新分配给其他 VM；

直到 DMA ownership 明确解除。

---

# 66. IOMMU

IOMMU map/unmap 代码必须把：

```text
CPU ownership
DMA ownership
device ownership
IOMMU mapping
```

视为统一事务。

不得先把页交给另一个 VM，再稍后异步清除旧 DMA mapping。

---

# 67. MMIO device emulation

MMIO handler 必须验证：

- access offset；
- size；
- alignment；
- read/write；
- register validity。

未知寄存器访问行为必须遵循设备规范：

```text
RAZ/WI
error
abort
reserved
```

不得默认 panic。

---

# 68. Trait 使用

Trait 应表达稳定语义边界。

合理：

```text
GuestAddressSpace
Scheduler
InterruptController
DeviceBackend
PlatformDiscovery
```

不合理：

```text
ArchitectureThing
PlatformEverything
GenericHelper
```

不得为了“将来可能扩展”提前抽象只有一个实现且语义尚不清楚的细节。

如果详细设计已经规定 trait，则按设计实现。

---

# 69. Trait object 与 generics

性能关键、对象数量大且实现编译期已知时，可采用 generics。

运行时需要多 backend、插件式 driver 或对象异构时，可采用：

```rust
dyn Trait
```

不得基于个人偏好将既定动态分派改成泛型，反之亦然。

---

# 70. Generic 参数

不要制造：

```rust
Foo<A, B, C, D, E, F>
```

式的深层类型传播。

泛型参数必须服务真实静态多态需求。

平台差异不应全部通过类型参数向 Core 传播。

---

# 71. 宏

宏只用于：

- 重复硬件寄存器定义；
- compile-time table；
- assembler glue；
- 重复 boilerplate 且函数无法替代。

不得用宏隐藏复杂控制流。

优先：

```rust
fn
const fn
trait
```

再考虑 macro。

---

# 72. proc macro

Hypervisor Core 应谨慎引入 proc macro。

任何 proc macro dependency 都增加：

- build complexity；
- supply-chain surface；
- IDE/debug complexity。

没有明显收益不得引入。

---

# 73. 第三方 crate

新增 dependency 前必须考虑：

- `no_std`；
- unsafe 数量；
- transitive dependencies；
- 维护状态；
- license；
- 是否引入 allocator；
- 是否引入 panic；
- 是否适合 hypervisor 环境。

不得为了一个简单数据结构引入大型依赖树。

---

# 74. 自己实现 vs crate

不要为了“纯自己写”而重复实现成熟、稳定、适用的小型基础组件。

也不要因为 crate 存在就无条件引入。

具体遵循：

> 可审计性 + no_std + dependency surface + 正确性 + 维护成本。

---

# 75. 注释

注释主要解释：

> 为什么。

代码本身应该解释：

> 做什么。

不推荐：

```rust
// Increment i.
i += 1;
```

推荐：

```rust
// VMID reuse is delayed until the global Stage-2 TLB invalidation
// completes on every physical CPU.
generation += 1;
```

---

# 76. 硬件注释

架构寄存器/设备行为存在不直观限制时，应注明：

- 规范章节；
- register 名；
- architectural requirement；

如果仓库规范允许，可写：

```text
Arm ARM: DDI xxxx, section ...
```

不要复制大段规范文本。

---

# 77. Public API 文档

核心公开 API 应使用 rustdoc 描述：

- 功能；
- 前置条件；
- 并发语义；
- 错误；
- Safety；
- blocking/IRQ 限制。

例如需要明确：

```text
May allocate
May block
IRQ-safe
Requires address-space lock
```

---

# 78. TODO

TODO 格式统一：

```text
TODO(#issue): description
```

禁止：

```text
TODO: fix later
TODO: hack
TODO: maybe
```

长期 hack 必须能够被追踪。

---

# 79. FIXME

`FIXME` 表示：

> 当前代码已知存在 correctness/security 问题。

默认不得把 FIXME 提交到稳定分支。

实验阶段若必须存在，应关联 issue，并不能进入 release profile。

---

# 80. 日志

不得在 fast path 无限制：

```rust
info!
debug!
trace!
```

高频事件应使用：

```text
structured trace
counter
rate-limited log
```

尤其：

- every VM exit；
- every IRQ；
- every page fault；
- every virtqueue descriptor；

不能默认打印文本。

---

# 81. 日志不能泄漏敏感内容

不要默认输出：

- Guest memory；
- credential；
- TLS key；
-完整磁盘 block；
-未清理的共享 buffer。

地址输出也应考虑 release/debug policy。

---

# 82. Trace

Trace event 应优先使用稳定 event ID + typed fields。

例如：

```text
VcpuEnter
VcpuExit
Stage2Fault
IrqInject
ScheduleSwitch
VirtqueueKick
```

而不是依赖解析自由格式字符串。

---

# 83. 性能优化

优化顺序：

```text
correctness
→ measurable behavior
→ benchmark
→ profiling
→ targeted optimization
```

不得凭感觉做：

- lock-free rewrite；
- unsafe aliasing；
- custom allocator；
- assembly optimization；
- cache-line tricks。

---

# 84. `#[inline]`

不要遍地：

```rust
#[inline(always)]
```

只有非常短且明确性能关键的底层 primitive 才考虑。

默认让编译器决定。

---

# 85. Cache line

per-CPU 高频写数据、跨 CPU atomic 等产生 false sharing 时，可使用 cache alignment wrapper。

不得在没有测量或明确硬件理由时给所有结构做 64/128-byte 对齐。

---

# 86. `repr(packed)`

默认禁止：

```rust
#[repr(packed)]
```

因为容易产生 unaligned reference UB。

协议/硬件结构需要 packed layout 时，应通过：

- byte parsing；
- unaligned read/write；
- 明确 wrapper；

避免创建未对齐 Rust 引用。

---

# 87. 对齐

所有 page table、descriptor、stack、DMA buffer 的 alignment 必须由类型或 allocator API 表达。

不要依赖：

```rust
debug_assert!(addr % 4096 == 0);
```

作为唯一保护。

对外输入需要 runtime check。

---

# 88. Bitfield

不要使用不透明 bitfield crate，除非项目明确选定。

推荐：

```text
mask
shift
typed getter/setter
bitflags
```

确保：

- reserved bits 明确；
- width 明确；
- unknown bits 策略明确。

---

# 89. C FFI

FFI 边界必须放在独立模块。

FFI 中：

- 使用固定宽度整数；
- 使用 `repr(C)`；
- 不跨 FFI 传 Rust enum 默认布局；
- 不跨 FFI unwind；
- 不跨 FFI 传普通 Rust trait object；
- 明确 pointer ownership。

---

# 90. Panic unwind

Hypervisor 默认不得依赖 panic unwind。

按项目 profile 通常采用：

```text
panic = abort
```

具体以构建配置为准。

不可设计需要 stack unwinding 才能恢复状态的逻辑。

---

# 91. 测试

新代码应优先同时考虑三种测试：

```text
host-side unit test
QEMU integration test
Guest-driven test
```

纯算法/解析器尽可能做 host-side test。

必须依赖 EL2 的代码则通过 QEMU integration test。

---

# 92. 测试确定性

测试不得依赖：

- 随机 sleep；
- 不受控 race；
- 主机负载；
- 固定执行耗时。

需要等待事件时，应等待明确 condition，并带 timeout。

---

# 93. Security regression

以下模块增加功能时原则上必须加入非法输入测试：

```text
HVC
virtio
MMIO emulation
management ABI
DTB parser
PCI config
snapshot parser
migration stream
```

只测试 happy path 不足够。

---

# 94. Fuzz

适合 fuzz 的逻辑应尽量与硬件层解耦成普通函数，例如：

```text
virtqueue descriptor parser
management message parser
DTB helper
snapshot decoder
PCI capability parser
```

不要让 parser 和 MMIO/EL2 register 强绑定而无法 fuzz。

---

# 95. Property test

对于：

```text
page allocator
bitmap
range map
capability generation
state machine
address translation
```

应优先考虑 property-based test。

例如：

> allocate 后 free，available count 恢复。

---

# 96. Debug assert

`debug_assert!` 只能检查内部 invariant。

不能用它验证不可信输入，因为 release build 可能移除。

Guest 输入必须使用真实 runtime validation。

---

# 97. Release correctness

代码不能依赖：

```text
overflow-checks debug behavior
debug_assert
logging side effect
test-only initialization
```

才能正确工作。

Debug 和 Release 的逻辑语义应一致。

---

# 98. Validation Guest

Validation Guest 是正式测试组件，不是一次性 demo。

新增以下核心能力时，应考虑增加对应 Validation Guest test：

```text
HVC
Stage-2
timer
IRQ
SGI
MMIO
SMP
shared memory
virtio
PSCI
```

---

# 99. 代码提交最小完整性

Coding Agent 完成一个函数或模块时，至少检查：

```text
build
fmt
clippy
relevant unit test
relevant QEMU test
```

如果环境无法执行某项，应明确报告：

> 未验证什么。

不能默认宣称成功。

---

# 100. 不要偷偷修别的问题

在实现任务 A 时发现 B 有问题：

- 若 B 是完成 A 所必需的明显 bug，可做最小修复；
- 若 B 是独立问题，应记录，不扩大任务。

Coding Agent 不得借机“大规模重构”。

---

# 101. 重构规则

除非任务明确要求重构，否则：

> Prefer minimal diff.

不要因为：

- 命名个人不喜欢；
- 想统一代码风格；
- 有另一种抽象；
- 想减少重复；

而修改不相关模块。

---

# 102. 公共接口稳定性

如果任务只要求实现函数内部，不得顺手修改：

```text
public function signature
public struct fields
trait method
ABI
module path
crate feature
```

如果当前接口无法正确实现，应报告设计冲突。

---

# 103. 数据结构字段可见性

默认：

```rust
private
```

只为真正需要跨模块访问的接口开放：

```rust
pub(crate)
pub
```

不要为了方便全部：

```rust
pub
```

核心对象应通过方法维护 invariant。

---

# 104. Getter

不要机械为每个字段生成 getter/setter。

尤其 setter：

```rust
set_state()
set_owner()
set_mapping()
```

可能绕过状态机。

优先提供语义操作：

```text
pause()
assign_to()
map_region()
revoke()
```

---

# 105. Builder

复杂配置对象可使用 Builder，但：

- 必填项必须最终验证；
- Builder 不能产生 invalid runtime object；
- runtime object 构造成功后应处于合法状态。

不要通过几十个 `Option<T>` 把非法状态拖入运行期。

---

# 106. Typestate

只有当状态差异能显著提高安全性且不会导致 API 极度复杂时，才使用 typestate。

不要为了展示 Rust 技巧，把所有 VM 生命周期建成：

```rust
Vm<Created>
Vm<Running>
Vm<Paused>
```

除非详细设计明确如此。

---

# 107. RAII 与硬件状态

RAII 很适合：

- lock；
- temporarily disabled IRQ；
- temporary mapping；
- resource reservation。

但不适合隐藏大规模异步硬件操作。

例如设备 reset、DMA drain 应显式表达。

---

# 108. Interrupt disable guard

如果提供：

```text
IrqDisableGuard
PreemptDisableGuard
```

必须支持正确嵌套。

不能简单：

```text
drop → enable interrupts
```

而忽略进入前 IRQ 已经关闭。

---

# 109. 时间

内部时间单位必须使用明确类型。

不要混用：

```text
ticks
nanoseconds
microseconds
counter values
```

例如：

```text
TimerTicks
DurationNs
CounterValue
```

转换必须通过明确函数。

---

# 110. Timeout

等待硬件时不得无限循环，除非该阶段明确无法恢复且设计要求如此。

设备初始化通常应有：

```text
timeout
error
diagnostic
```

QEMU 正常并不代表真机永远及时响应。

---

# 111. Poll loop

硬件 poll loop 应适当使用：

```text
spin_loop
WFE
timer deadline
```

避免无界 CPU busy loop。

具体机制由硬件语义决定。

---

# 112. 资源释放顺序

资源销毁应严格反向撤销其依赖。

典型设备：

```text
stop new requests
→ quiesce
→ mask IRQ
→ drain DMA
→ detach IOMMU
→ unmap shared memory
→ release physical device
```

不得简单：

```rust
drop(device);
```

假设一切自动安全。

---

# 113. VM 销毁

VM destroy 必须考虑：

```text
stop vCPU
remove from scheduler
stop devices
revoke capabilities
detach DMA
remove IRQ routes
unmap memory
invalidate Stage-2/TLB
release pages
destroy object table entries
```

具体次序按设计，不得省略。

---

# 114. 可恢复失败

创建复杂对象时使用事务式思维。

例如 VM 创建到一半失败时：

```text
已分配页面
已创建 vCPU
已注册 IRQ
```

必须能够 rollback。

不得泄漏半初始化对象。

---

# 115. `Drop` 不能替代完整 rollback 设计

RAII 可帮助局部 rollback，但复杂多阶段对象创建应有明确 ownership transfer。

每一步成功后，责任归属必须明确。

---

# 116. Boot-only 内存

Early boot allocator 和 normal allocator 必须明确切换点。

Boot memory 不应在进入 normal runtime 后继续被隐式使用，除非设计明确保留。

---

# 117. 常量

硬件常量应集中到合理模块。

不得散落：

```rust
0x0800_0000
0x30
1 << 27
```

命名必须表达语义。

---

# 118. Magic number

普通算法同样避免 magic number。

例如：

```rust
if descriptors > 1024
```

应改为：

```rust
MAX_VIRTQUEUE_CHAIN_LEN
```

并说明来源。

---

# 119. 配置默认值

默认值必须在设计明确后统一定义。

不同模块不得各自假定：

```text
默认 vCPU 数
默认 page size
默认 timer frequency
默认 queue size
```

---

# 120. Logging 与 error 不重复负责

返回：

```rust
Err(...)
```

的底层函数通常不应同时无条件打印 error。

由决定如何处理该错误的层负责日志。

否则同一错误会被多层重复打印。

硬件 fatal diagnostic 等例外按设计。

---

# 121. Public enum 兼容性

进入 ABI 或持久化格式的 enum 新增成员时，解析器必须处理未知值。

内部 enum 可采用 exhaustive match。

ABI enum 不应默认：

```rust
unsafe { transmute(value) }
```

---

# 122. Match

核心状态机优先完整：

```rust
match
```

而不是：

```rust
_ => {}
```

除非明确需要忽略 future/unknown value。

通配符可能隐藏新增状态未处理的问题。

---

# 123. `unreachable!`

只有类型/逻辑已经证明路径不可达时使用。

Guest 输入不能使：

```rust
unreachable!()
```

可达。

否则这是安全漏洞风险。

---

# 124. `assert!`

`assert!` 用于内部 invariant。

任何 Guest、设备、firmware、管理输入不得依赖 assert 验证。

这些必须返回错误。

---

# 125. 安全边界函数要短

如下函数应尽量保持短小、易审计：

```text
copy_from_guest
capability_lookup
virtqueue descriptor validation
IOMMU mapping
Stage-2 mapping mutation
MMIO dispatch
hypercall dispatch
```

不要把大量业务逻辑混入安全检查函数。

---

# 126. Parser 与执行分离

外部输入处理推荐：

```text
bytes/registers
→ parse
→ validate
→ typed request
→ execute
```

不要：

```text
边解析
边修改 Hypervisor state
```

除非协议天然 streaming 且设计明确。

---

# 127. 两阶段修改

涉及多个全局资源时优先：

```text
validate/reserve
→ commit
```

例如：

```text
VM memory hotplug
device assignment
CPU hotplug
capability delegation
```

减少半完成状态。

---

# 128. 可观测状态

不得为了 telemetry 改变核心语义。

计数器失败不能影响 VM 正常运行。

但关键 security/audit event 丢失行为按对应设计。

---

# 129. Counters

高频统计优先 per-CPU/per-vCPU counter，避免全局 atomic 热点。

聚合应在读取时完成。

---

# 130. Benchmark path

性能 benchmark 必须能关闭：

```text
trace
debug logging
extra validation not required for correctness
```

但绝不能关闭安全边界检查。

---

# 131. Debug feature

`debug` feature 不得改变安全语义。

可以增加：

```text
assert
trace
dump
poison
redzone
ownership checker
```

不能在 release 中移除必须存在的边界检查。

---

# 132. 测试专用 API

测试 helper 应放：

```rust
#[cfg(test)]
```

或 `test-support` crate。

不得因为测试方便而把内部状态开放为 production `pub fn set_anything()`。

---

# 133. Mock

硬件 mock 应实现与真实 driver 相同的稳定接口。

不要为了测试创建完全不同调用路径。

目标是测试 Core，而不是测试 mock 自己。

---

# 134. 代码生成

寄存器表、ABI constants 等如果使用代码生成：

- 生成源必须版本控制或生成过程可完全重复；
- build 不得依赖联网；
- generated file 有明显标记；
- 不手改 generated output。

---

# 135. Build script

`build.rs` 应保持简单。

不得把核心架构逻辑塞进 build script。

典型职责：

```text
linker args
generated constants
version metadata
assembly compilation
```

---

# 136. 链接脚本

修改 linker script 时必须同步确认：

```text
text
rodata
data
bss
stack
percpu
page-table pool
boot package
alignment
```

不得只为了修一个 symbol 临时移动段布局而不检查物理地址影响。

---

# 137. Symbol visibility

汇编需要的 Rust symbol 应明确：

```rust
#[no_mangle]
extern "C"
```

仅在必要边界使用。

不要对普通内部函数添加 `no_mangle`。

---

# 138. ABI 调用约定

跨汇编/FFI 边界必须明确：

```text
calling convention
callee/caller saved registers
stack alignment
return convention
error convention
```

不得假定 Rust ABI 稳定。

---

# 139. Float / SIMD

Hypervisor Core 默认不要使用浮点。

编译器、formatting 或依赖也不得意外要求 FP/SIMD state，除非 vCPU FP/SIMD 保存策略已经明确。

---

# 140. Dynamic dispatch in IRQ/exit fast path

如设计允许，应避免在极高频路径进行多层动态分派。

但不得为了优化提前破坏 backend abstraction。

先 profile，再决定是否 devirtualize。

---

# 141. String

EL2 中不要在核心路径无谓构造：

```rust
String
format!
```

特别是错误/trace path。

优先：

```text
enum error
static str
structured fields
```

---

# 142. UTF-8

来自 Guest/firmware 的 byte buffer 不得默认是有效 UTF-8。

需要文本时：

```rust
str::from_utf8
```

失败时按协议处理。

---

# 143. 生命周期注释

复杂对象如果依赖：

> A 必须比 B 活得久，

应通过 ownership/type 尽可能表达。

无法通过类型表达时，必须在对象定义或 SAFETY 注释中明确 invariant。

---

# 144. Capability rights

权限必须使用正向白名单。

推荐：

```rust
rights.contains(Rights::MAP)
```

而不是：

```text
只要没有 DENY_MAP 就允许
```

未知权限位不得默认授予。

---

# 145. 权限削减

Capability delegation 时，新权限必须满足：

```text
child_rights ⊆ parent_rights
```

不能通过 delegation 获得调用者本身没有的权限。

---

# 146. Capability revoke

实现 revoke 时必须考虑：

```text
in-flight operation
shared mappings
device ownership
child capability
generation
```

不得仅从一个 map 删除条目就认为撤权完成。

具体语义按设计实现。

---

# 147. 角色

`ControlDomain`、`DriverDomain` 等 role 只能用于：

- 默认配置；
- telemetry；
- policy input。

EL2 权限判断不得写：

```rust
if domain.role == Control {
    allow();
}
```

除非具体安全设计明确规定该操作本身就是角色语义。

默认必须检查 capability。

---

# 148. Machine type

Guest-visible hardware 行为必须由 versioned MachineType 驱动。

不得根据 Host board 自动改变 Guest ABI。

例如同一个：

```text
rusthv-arm-virt-v1
```

在 QEMU host 与 RK3566 host 上应保持相同 Guest-visible 语义。

---

# 149. Host/Guest 概念命名

代码中必须清楚区分：

```text
host
guest
physical
virtual
platform
machine
```

禁止一个变量：

```rust
irq
```

同时可能表示：

```text
physical IRQ
virtual IRQ
guest IRQ number
```

应命名：

```rust
phys_irq
virt_irq
guest_irq
```

---

# 150. Configuration code

Hypervisor EL2 内不得引入：

```text
YAML
JSON
XML
libvirt XML
复杂 TOML
```

parser。

EL2 只消费设计规定的：

```text
typed boot config
native management ABI
simple versioned binary representation
```

---

# 151. Control Domain code

Control Domain 可以使用：

```text
serde
database
filesystem
network stack
TLS
libvirt
```

但必须与 EL2 ABI 类型分层。

不要让 Control Domain 的 high-level model 直接成为 Hypervisor wire struct。

---

# 152. Migration / snapshot

任何持久化数据都必须：

- 有 magic；
- 有 version；
- 有长度；
- 有边界检查；
- 明确 endianness；
- 能拒绝未知/损坏数据。

不得直接：

```text
dump Rust struct memory
```

作为 snapshot 格式。

---

# 153. Forward compatibility

解析 versioned 格式时：

- unknown mandatory feature → 拒绝；
- unknown optional feature → 按协议忽略；
- unknown enum/value → 不得 UB。

---

# 154. Security critical review marker

以下代码建议在 review 中标记为高风险：

```text
unsafe
asm
page table
IOMMU
capability
Guest pointer
virtqueue
ABI parser
interrupt entry
context switch
object destruction
```

Coding Agent 在这些区域修改时，应在结果说明中明确指出。

---

# 155. Coding Agent 输出要求

Agent 完成任务后应报告：

1. 修改了哪些文件；
2. 实现了哪些设计要求；
3. 是否增加 `unsafe`；
4. 是否修改 ABI；
5. 是否修改 public API；
6. 是否增加 dependency；
7. 执行了哪些测试；
8. 哪些验证无法执行；
9. 是否存在 TODO/FIXME；
10. 是否发现设计文档矛盾。

不要只回复：

> implemented successfully.

---

# 156. Coding Agent 开工前检查

开始编码前必须先确定：

```text
目标模块
目标函数
输入输出
ownership
状态机
并发上下文
错误语义
平台/架构边界
ABI 是否稳定
是否允许 allocation
是否允许 blocking
```

若这些在详细设计中已经给出，严格执行。

若设计没有给出，但实现必须依赖其中之一，不应凭个人偏好做重大选择。

---

# 157. Coding Agent 禁止的典型行为

明确禁止：

```text
为了方便加入 global singleton
为了让 borrow checker 通过加 unsafe
为了快速成功直接 unwrap
为了跑过 QEMU 删除 barrier
为了性能删除 Guest 输入检查
为了通用化引入巨大 trait
为了复用让 Core import Board crate
为了简单使用 VM ID == 0 判断 root
为了方便在 EL2 引入 std
为了测试修改 production public API
为了减少代码把不同地址空间都用 usize
为了“以后可能用”提前加入复杂框架
为了让 Clippy 安静直接 #[allow(...)] 整个模块
为了暂时跑通把错误变成 panic
为了设备直通跳过 DMA ownership
为了避免实现状态机直接修改 state 字段
```

---

# 158. Clippy allow

不得大范围：

```rust
#![allow(clippy::all)]
```

任何：

```rust
#[allow(...)]
```

都应限制到最小作用域。

非显而易见的 allow 应说明原因。

---

# 159. Dead code

不要长期保留：

```rust
#[allow(dead_code)]
```

掩盖未使用代码。

尚未接入的未来实现应尽量不提前提交，或放在明确 feature/实验模块。

---

# 160. 设计一致性检查

Coding Agent 在完成实现后必须自检：

```text
代码是否实现了设计，而不是重新设计？
Core 是否泄漏架构/平台细节？
错误输入能否导致 panic？
是否存在新的隐式全局状态？
是否存在裸 usize 地址？
是否新增未经说明的 unsafe？
是否破坏状态机？
是否绕过 capability？
是否遗漏 SMP 同步？
是否遗漏 TLB/IRQ/DMA 生命周期？
```

---

# 161. Definition of Done

一个编码任务只有在以下条件满足后才算完成：

```text
设计要求实现完整
代码可构建
格式检查通过
静态检查通过
相关测试通过
错误路径已实现
非法输入已考虑
SMP/并发语义明确
unsafe 有 SAFETY 说明
无未经授权的架构变化
无隐藏 ABI 变化
无无关重构
文档/注释与实际行为一致
```

如果其中部分无法验证，必须明确列出。

---

# 162. 最重要的实现原则

所有 Coding Agent 应始终遵循以下五条：

**第一，设计已经决定“做什么”，编码阶段不要重新发明架构。**

**第二，让类型系统表达地址、身份、ownership、权限和状态，而不是依赖注释约定。**

**第三，任何来自 Guest、设备、firmware 和管理域的数据都必须先验证再使用。**

**第四，unsafe、汇编、裸指针、MMIO 和并发原语必须被限制在小而可审计的边界。**

**第五，QEMU 上“能跑”不代表实现正确；代码必须满足真实 SMP、cache、TLB、IRQ、DMA 和硬件 ordering 语义。**

---

# 163. Coding Agent 简版系统提示词

当完整准则无法全部放入 Coding Agent context 时，至少提供以下精简版本：

```text
You are implementing an existing Rust Type-1 Hypervisor design.

Do not redesign architecture, APIs, state machines, module boundaries,
resource ownership, ABI, locking, or platform abstractions unless the
task explicitly requires it.

Follow the provided detailed design literally.

Mandatory implementation rules:

1. Hypervisor EL2 code is no_std and Rust-first.
2. Use assembly only for unavoidable architecture entry/exit/context code.
3. Minimize unsafe; every unsafe block/function must document its safety invariant.
4. Never use raw usize/u64 interchangeably for HPA/HVA/GPA/GVA.
   Use the project's typed address/newtype APIs.
5. Guest, device, firmware and management inputs are untrusted.
   Validate ranges, permissions, lengths, alignment and integer overflow before use.
6. Guest-caused errors must not panic the hypervisor.
7. Avoid unwrap/expect/todo/unimplemented in reachable runtime paths.
8. Preserve explicit state machines; do not mutate lifecycle state directly.
9. Respect capability/rights checks. Never replace authorization with VM ID or role checks.
10. Respect crate layering. Core must not depend on board/SoC-specific implementations.
11. Platform-specific code belongs only in Arch/SoC/Board/Driver/Quirk layers.
12. Do not add board-name conditionals to generic core code.
13. MMIO must use volatile access and correct barriers.
14. Page-table changes must account for ordering, TLB invalidation and SMP shootdown.
15. DMA ownership, IOMMU mapping and device ownership must stay consistent.
16. Interrupt-context code must not block, perform heavy allocation or execute long operations.
17. Do not introduce Arc<Mutex<_>>, atomics, lock-free code or globals merely for convenience.
18. Use checked arithmetic for untrusted address/length computations.
19. Wire/ABI formats must have explicit layout, width, endianness and versioning.
20. Never serialize raw Rust struct memory as an external ABI/snapshot format.
21. Minimize allocation in VM-exit, IRQ, scheduler and virtqueue hot paths.
22. Do not introduce new dependencies without a concrete need.
23. Prefer minimal diffs; do not perform unrelated refactoring.
24. Add or update relevant unit/QEMU/guest/security regression tests.
25. Report all new unsafe, ABI changes, dependencies, untested paths and design conflicts.

Correctness and isolation take priority over elegance and premature optimization.
QEMU success does not justify violating real ARM/x86 memory ordering,
TLB, interrupt or DMA semantics.
```

---

**End of Rust Type-1 Hypervisor Coding Guidelines v0.1**
