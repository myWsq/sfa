# 03. 总体架构设计

## 1. 架构总览

SFA 的总体结构分为四层：

1. **Unix 文件系统层**
   - 目录扫描
   - 元数据采集
   - 文件恢复
   - 安全路径解析

2. **归档规划层**
   - entry 建树
   - hardlink 识别
   - bundle 规划
   - manifest 生成

3. **协议与编解码层**
   - header / manifest / frame 编解码
   - codec 适配
   - 完整性校验
   - 流式状态机

4. **CLI / API 层**
   - 参数解析
   - 任务调度
   - 统计输出
   - 与上游系统集成

## 2. 逻辑架构图

```text
+--------------------------------------------------------------+
|                           CLI / API                          |
+-------------------------------+------------------------------+
                                |
                                v
+--------------------------------------------------------------+
|                    Pipeline Orchestrator                     |
|     config / queues / threadpool / ordered writer / stats    |
+-------------------------------+------------------------------+
                                |
       +------------------------+------------------------+
       |                                                 |
       v                                                 v
+--------------+                                  +--------------+
|  Scanner     |                                  |  Unpacker    |
|  Planner     |                                  |  Restorer    |
+------+-------+                                  +------+-------+
       |                                                 |
       v                                                 v
+--------------------------------------------------------------+
|                    Format / Codec / Integrity                |
|      header / manifest / frame / lz4 / zstd / checksum      |
+-------------------------------+------------------------------+
                                |
                                v
+--------------------------------------------------------------+
|                       Unix FS / IO Layer                     |
| openat / mkdirat / linkat / symlinkat / write_at / utimensat|
+--------------------------------------------------------------+
```

## 3. 压缩侧架构

### 3.1 流水线

```text
目录扫描 -> entry 归一化 -> bundle 规划 -> 读取原始数据
      -> 压缩 worker pool -> frame 结果重排 -> 顺序写出 archive
```

### 3.2 阶段职责

#### 阶段 A：目录扫描

负责：

* 遍历目录树
* 收集元数据
* 生成稳定顺序的 entry 列表
* 识别 hardlink 主从关系
* 校验路径合法性

#### 阶段 B：归档规划

负责：

* 生成 entry tree
* 将文件映射为 extents
* 规划 bundle
* 生成 manifest

#### 阶段 C：原始数据读取

负责：

* 根据 bundle 规划，从源文件读取原始字节
* 组装为 raw bundle buffer
* 将 buffer 投递到压缩队列

#### 阶段 D：压缩 worker

负责：

* 对 raw bundle 应用指定 codec
* 生成 frame payload
* 计算 frame 校验
* 输出 `EncodedBundle`

#### 阶段 E：有序写出

负责：

* 先写 header
* 再写 manifest
* 再按 bundle_id 顺序写 frame
* 最后可选写 trailer

## 4. 解压侧架构

### 4.1 流水线

```text
Read / AsyncRead
   -> 读取 Header
   -> 读取并解析 Manifest
   -> 预创建目录 / 预注册恢复上下文
   -> 顺序读取 Frame
   -> 解压 worker pool
   -> scatter 到目标文件
   -> 元数据收尾
```

### 4.2 阶段职责

#### 阶段 A：头部解析

* 校验 magic / version
* 加载 codec / integrity / bundle 参数
* 决定有效线程数

#### 阶段 B：manifest 解析

* 解码 manifest
* 构建 entry 表与 extent 索引
* 初始化恢复计划
* 预创建目录结构

#### 阶段 C：frame 消费

* 顺序读取 frame header
* 读取 payload
* 校验 frame
* 并行解码 raw bundle

#### 阶段 D：scatter 恢复

* 根据 bundle_id 找到相关 extents
* 将 raw bundle 中的切片写到对应文件的 `file_offset`
* 处理普通文件、符号链接、硬链接和目录

#### 阶段 E：元数据 finalize

* 设置目录 mtime
* 根据策略应用 chown/chmod
* 校验 strong trailer（如启用）
* 输出统计与结果

## 5. 为什么 manifest 放在前面

由于压缩前允许完整扫描目录，因此 manifest 可以在压缩前完全确定。

manifest 中记录的是：

* entry 结构
* 文件与 bundle 的对应关系
* file extent 在 raw bundle 中的 offset / len

manifest **不记录压缩后 frame 的物理偏移**。
因此它不需要等待压缩完成后回填，可以安全地放在归档头部。

这使协议同时满足：

* 头部可先读
* manifest 可先读
* 数据段可顺序流式消费
* 不依赖 seek

## 6. bundle 作为工作单元

## 6.1 基本原则

* 小文件尽量聚合到一个 bundle 中
* 大文件切分为多个 bundle
* bundle 大小保持相对均匀
* bundle_id 全局递增且稳定

## 6.2 默认参数建议

* `bundle_target_bytes = 4 MiB`
* `small_file_threshold = 256 KiB`

### 解释

* 小于 `small_file_threshold` 的文件优先参与聚合
* 大于阈值的文件按固定 chunk 切分为一个或多个 bundle
* 一个 bundle 的原始字节量以 `bundle_target_bytes` 为目标上限

## 6.3 bundle 的收益

* 降低小文件 syscall 密度
* 降低 codec 上下文切换次数
* 提高多线程工作均衡性
* 提高 frame 粒度与顺序写出效率
* 便于流式恢复

## 7. 并发模型

## 7.1 原则

* 并发建立在 bundle 级
* 压缩和解压 worker pool 大小可配置
* writer / reader 使用单顺序流
* 中间阶段使用有界队列实现背压

## 7.2 压缩侧队列

建议队列拓扑：

```text
Scan/Plan  ->  ReadQueue  ->  CompressQueue  ->  OrderedWriteQueue
```

其中：

* `ReadQueue`：待读取 bundle 任务
* `CompressQueue`：待压缩 raw bundle
* `OrderedWriteQueue`：等待写出的 encoded bundle

## 7.3 解压侧队列

建议队列拓扑：

```text
FrameReadQueue  ->  DecodeQueue  ->  RestoreQueue
```

其中：

* `FrameReadQueue`：等待读取 frame payload
* `DecodeQueue`：等待解压 payload
* `RestoreQueue`：等待 scatter 到文件

## 7.4 ordered writer / restore 的必要性

压缩阶段可并行，但 archive 的物理写出必须按 bundle_id 顺序进行。
解压阶段即便内部并行，也应保证对外部顺序流的消费稳定可控。

建议设计：

* `next_bundle_id_to_write`
* `BTreeMap<u64, EncodedBundle>` 缓存乱序完成结果
* 当且仅当连续 bundle 就绪时才写出

相同思路也可应用于解压后的有序恢复。

## 8. IO 设计

## 8.1 pack 阶段

pack 只面向本地目录，因此可主要使用同步文件 IO + 多线程。

设计建议：

* 目录扫描使用 `lstat`
* 文件读取用 `std::fs::File`
* 通过 bundle 任务实现读取并行
* 避免过多同时打开文件
* 使用固定容量 buffer 复用

## 8.2 unpack 阶段

unpack 必须支持：

* `Read`
* `AsyncRead`

建议实现分层：

* 协议解析核心以“字节状态机”实现
* sync / async 只负责向状态机喂入字节
* 这样可避免维护两套协议逻辑

## 8.3 Unix 安全 IO

恢复目标目录时，建议所有路径操作都建立在输出根目录的 dirfd 之上：

* `openat`
* `mkdirat`
* `linkat`
* `symlinkat`
* `fchmodat`
* `fchownat`
* `utimensat`

这样可减少：

* 路径拼接错误
* `..` 绕过风险
* 中间路径 symlink 逃逸风险

## 9. 文件恢复模型

### 9.1 regular file

* 创建目标文件
* 使用 `write_at(file_offset, bytes)` 写入
* 完成后统一设置 mode / mtime / owner

### 9.2 directory

* 预先创建
* 所有内容恢复完成后再设置 mtime / mode

### 9.3 symlink

* 在目标路径创建符号链接
* 不跟随 link target
* 恢复阶段不允许其影响父路径解析

### 9.4 hardlink

* 第一个出现的 inode 作为 master entry
* 其余 entry 只保存对 master 的引用
* 解压阶段在 master 存在后调用 `linkat`

## 10. 临时缓冲策略

### 10.1 原则

* 允许内存缓冲，以提升吞吐
* 不将“先落临时文件”作为协议或功能前提
* 有界缓冲，避免因 worker 堵塞导致内存无限增长

### 10.2 建议

* 每个正在处理的 bundle 保留一份 raw buffer
* 编码后的 payload 保留到 ordered writer 消费
* 队列深度按线程数倍数设置，例如：

  * `read_queue_depth = threads * 2`
  * `compress_queue_depth = threads * 2`

## 11. 架构结论

SFA 的核心架构不是“给 tar 换个 codec”，而是：

> **通过前置 manifest 与 bundle/frame 级工作单元，将目录树归档问题转化为稳定的并行数据流水线问题。**

这使它天然适合：

* 海量小文件
* 多核机器
* 顺序流输入
* 本地与网络统一消费
