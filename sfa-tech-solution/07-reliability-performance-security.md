
# 07. 可靠性、性能与安全设计

## 1. 完整性设计

## 1.1 设计目标

完整性方案必须在三者之间平衡：

* 吞吐
* 提前发现损坏
* 最终一致性确认

因此推荐三档策略：

| 模式     | 含义                    | 建议用途   |
| ------ | --------------------- | ------ |
| off    | 不校验                   | 极端临时场景 |
| fast   | 头部 + manifest + frame | 默认     |
| strong | fast + 整体 trailer     | 严谨归档场景 |

## 1.2 fast 模式

### 校验内容

* Header CRC32
* Manifest hash
* Frame payload hash

### 价值

* 读取头部即可发现明显损坏
* 读完 manifest 可发现协议元信息错误
* 逐 frame 校验有助于尽早在流式解压中止损

## 1.3 strong 模式

在 fast 基础上增加：

* Trailer 中的 archive hash
* 原始总字节数与编码总字节数核对

### 价值

* 能在整个归档完成后做最终一致性确认
* 更适合离线归档与归档存证场景

## 2. 性能设计

## 2.1 性能原则

* 优先吞吐，不优先压缩比
* 优先 bundle 级并发，不优先 codec 内部线程
* 优先减少小 IO 与频繁上下文切换
* 优先让 writer / reader 保持顺序流

## 2.2 影响性能的关键因素

### 目录扫描

* `lstat` 数量
* basename 分配与拷贝
* 排序成本
* hardlink 哈希表维护

### bundle 读取

* 文件 open/close 次数
* 小文件读放大
* buffer 分配频率
* 读取线程与磁盘能力匹配度

### 编码 / 解码

* codec 本身吞吐
* bundle 大小
* 线程池规模
* 内存复制次数

### 恢复写回

* 文件句柄缓存命中率
* `write_at` 系统调用数量
* 目录 / 文件元数据 finalize 顺序
* 目标磁盘性能

## 2.3 默认调优参数建议

| 参数                    |       建议默认值 | 说明        |
| --------------------- | ----------: | --------- |
| codec                 |         lz4 | 默认高速      |
| threads               |      CPU 核数 | 可被 CLI 覆盖 |
| bundle_target_bytes   |       4 MiB | 吞吐与内存折中   |
| small_file_threshold  |     256 KiB | 小文件聚合阈值   |
| queue_depth_per_stage | threads * 2 | 控制背压      |
| integrity             |        fast | 默认完整性档位   |

## 2.4 内存模型建议

### pack

内存主要由以下部分组成：

* `N_read` 个待压缩 raw bundle
* `N_encode` 个已压缩等待写出的 payload
* manifest 构建期的 entry / extent / name arena

粗略上界可估算为：

```text
memory ~= manifest_memory
       + queue_depth * bundle_target_bytes
       + queue_depth * avg_encoded_bundle_bytes
       + file_handle_cache
```

### unpack

内存主要由以下部分组成：

* 已解析 manifest
* 待读取 frame payload buffer
* 解压后的 raw bundle
* 文件句柄缓存

### 控制策略

* 队列使用 bounded channel
* 限制同时在途 bundle 数
* 复用 buffer
* 解压端句柄缓存设置上限

## 3. 句柄缓存策略

## 3.1 背景

大量小文件恢复时，频繁 open/close 会增加明显开销。
但全量持有文件句柄也可能导致 fd 爆炸。

## 3.2 建议方案

使用 LRU 句柄缓存：

* key: `entry_id`
* value: 已打开的文件句柄与状态
* 上限：默认 256 或与线程数相关
* 淘汰时执行 flush / close

## 4. 安全设计

## 4.1 核心原则

解压归档属于高风险操作，必须默认“安全优先”。

### 必须满足

* 不允许绝对路径
* 不允许 `..`
* 不允许路径段为空
* 不允许 NUL
* 不跟随输出根内已有 symlink 进行路径逃逸
* 默认拒绝恢复 special file

## 4.2 路径安全策略

建议所有恢复操作基于输出根 `dirfd`，并坚持：

* 逐级解析 basename
* 每级目录通过 fd 打开
* 创建文件 / 目录 / 链接都相对于父 dirfd
* 不使用 `chdir`
* 不将拼接后的字符串直接传给不安全路径 API

## 4.3 symlink 风险控制

风险：

* 归档内 symlink 指向外部路径
* 输出目录中已有恶意 symlink
* 中间路径解析被 symlink 劫持

控制策略：

* symlink target 只作为数据恢复，不参与父路径解析
* 创建普通文件时必须对父目录使用 dirfd 安全操作
* 不允许通过已存在 symlink 打开最终写入路径

## 4.4 owner / permission 策略

默认行为建议：

* 非 root：忽略 owner 恢复
* root：仅在显式 `--restore-owner preserve` 时恢复 uid/gid
* mode 默认恢复，但不应覆盖安全策略限制
* 特殊文件默认禁止创建

## 5. xattrs / ACL 设计

## 5.1 v1 方案

协议中预留 `MetaBlob TLV`，实现上作为次阶段能力。

### 这样做的好处

* 协议先具备扩展位置
* 不把核心吞吐路径拖入高复杂度元数据分支
* 后续可在不破坏主流程的情况下逐步增强

## 5.2 建议开关

* `--xattrs none|user|all`
* `--acl off|on`

默认：

* `xattrs = none`
* `acl = off`

## 6. 观测与诊断

## 6.1 最小日志字段

pack / unpack 都建议输出：

* codec
* threads
* bundle_target_bytes
* small_file_threshold
* entry_count
* bundle_count
* raw_bytes
* encoded_bytes
* duration breakdown
* files/s
* MiB/s

## 6.2 debug 信息

在 debug 模式下，建议输出：

* manifest 统计
* bundle 分布
* 大文件切片情况
* 队列长度峰值
* frame hash 校验统计
* 文件句柄缓存命中率

## 7. 可靠性测试重点

### 7.1 协议正确性

* 错误 magic
* 不支持版本
* header 长度错误
* manifest 长度错误
* frame 截断
* bundle_id 非单调递增

### 7.2 安全性

* `../../etc/passwd`
* `/absolute/path`
* `a//b`
* 包含 NUL 的路径
* symlink 绕出输出根
* special file 默认拒绝

### 7.3 稳定性

* 超大 entry_count
* 超大 manifest
* 极端小 bundle
* 极端高线程数
* 慢速网络流 / 碎片化 chunk 输入

## 8. 性能优化路线

### 8.1 首要优化点

1. bundle 规划质量
2. 有界队列参数
3. buffer 复用
4. 文件句柄缓存
5. manifest 编码与压缩

### 8.2 次级优化点

1. 并行目录遍历
2. 更精细的 bundle 装箱
3. 动态线程调节
4. 读写阶段的 NUMA 亲和优化
5. 自适应小文件阈值

## 9. 风险与对策

| 风险              | 影响       | 对策                    |
| --------------- | -------- | --------------------- |
| manifest 过大     | 头部读取压力变大 | manifest codec + 结构压缩 |
| bundle 太小       | 调度开销高    | 提高默认 bundle size      |
| bundle 太大       | 内存峰值偏高   | bounded queue + 参数调优  |
| hardlink 处理错误   | 文件树语义不正确 | 明确 master-first 规则    |
| 解压路径绕逸          | 安全风险     | dirfd + 拒绝绝对路径/..     |
| xattrs/ACL 过早引入 | 开发复杂度升高  | 先协议预留，后实现             |

## 10. 结论

SFA 的可靠性与安全设计必须建立在两个原则之上：

1. **协议层尽早失败**
2. **文件系统层默认保守**

性能优化则应建立在第三个原则之上：

3. **把主要算力花在 bundle 级数据流水线，而不是碎片化文件边界上**

