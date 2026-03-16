
# 04. SFA v1 数据协议设计

## 1. 设计目标

SFA v1 协议需要同时满足以下目标：

* 严格顺序流式可读
* 解压不依赖 seek
* 支持 bundle/frame 级并行
* 支持 Unix 文件树表达
* 支持前向扩展
* 协议实现简单且稳定

## 2. 物理布局

```text
Archive
= HeaderV1
| ManifestSection
| DataFrame*
| TrailerV1?   // strong 模式或显式启用时存在
```

### 说明

* `HeaderV1` 固定长度，便于快速判定
* `ManifestSection` 位于数据段前部，用于提前获取文件树信息
* `DataFrame` 顺序出现，可被本地文件流或 HTTP 流逐个读取
* `TrailerV1` 不是正常解压的前置依赖

## 3. 字节序与编码规则

* 整个协议固定使用 **little-endian**
* 固定头部结构采用定长字段
* manifest 中的表使用定长 record
* 字节串（路径、link target、扩展元数据）放入独立 arena/blob
* 路径与 link target 以 **原始字节序列** 存储，不强制 UTF-8

## 4. HeaderV1

### 4.1 大小

`HeaderV1` 固定为 **128 bytes**

### 4.2 字段布局

| Offset | Size | Field                 | Type     | 说明                    |
| -----: | ---: | --------------------- | -------- | --------------------- |
|      0 |    8 | magic                 | [u8; 8]  | 固定为 `SFA\0\r\n\x1A\n` |
|      8 |    2 | header_len            | u16      | 固定为 128               |
|     10 |    2 | version_major         | u16      | 主版本，v1 为 1            |
|     12 |    2 | version_minor         | u16      | 次版本，初始为 0             |
|     14 |    2 | data_codec            | u16      | 数据段 codec             |
|     16 |    2 | manifest_codec        | u16      | manifest codec        |
|     18 |    1 | integrity_mode        | u8       | 完整性模式                 |
|     19 |    1 | frame_hash_algo       | u8       | frame 校验算法            |
|     20 |    1 | manifest_hash_algo    | u8       | manifest 校验算法         |
|     21 |    1 | reserved0             | u8       | 置 0                   |
|     22 |    2 | suggested_parallelism | u16      | 建议并发度                 |
|     24 |    4 | bundle_target_bytes   | u32      | 规划时的 bundle 目标大小      |
|     28 |    4 | small_file_threshold  | u32      | 小文件阈值                 |
|     32 |    8 | entry_count           | u64      | entry 数               |
|     40 |    8 | extent_count          | u64      | extent 数              |
|     48 |    8 | bundle_count          | u64      | bundle 数              |
|     56 |    8 | manifest_raw_len      | u64      | manifest 解码后的字节数      |
|     64 |    8 | manifest_encoded_len  | u64      | manifest 编码后的字节数      |
|     72 |    8 | feature_flags         | u64      | 特性位图                  |
|     80 |   32 | manifest_hash         | [u8; 32] | manifest 校验值          |
|    112 |    4 | header_crc32          | u32      | 头部 CRC32（该字段置 0 后计算）  |
|    116 |    2 | writer_version_major  | u16      | 生成器版本主号               |
|    118 |    2 | writer_version_minor  | u16      | 生成器版本次号               |
|    120 |    8 | reserved1             | [u8; 8]  | 置 0，预留                |

### 4.3 Header 语义说明

* `suggested_parallelism`：仅表示编码端建议并发度，解压端可覆盖
* `bundle_target_bytes` / `small_file_threshold`：供诊断、调优与兼容观察使用
* `feature_flags`：标记归档内是否含 symlink / hardlink / xattrs / trailer 等能力

## 5. Header 枚举定义

### 5.1 data_codec

|     值 | 名称              | 说明         |
| ----: | --------------- | ---------- |
|     0 | none            | 不压缩，调试用途   |
|     1 | lz4             | 默认高速模式     |
|     2 | zstd            | 速度 / 压缩比折中 |
|     3 | reserved_snappy | 预留         |
| 65535 | custom          | 自定义扩展保留    |

### 5.2 manifest_codec

|  值 | 名称   | 说明           |
| -: | ---- | ------------ |
|  0 | none | manifest 不压缩 |
|  1 | zstd | 默认建议值        |

### 5.3 integrity_mode

|  值 | 名称     | 说明                       |
| -: | ------ | ------------------------ |
|  0 | off    | 不做完整性校验                  |
|  1 | fast   | 默认，manifest + frame 级校验  |
|  2 | strong | 在 fast 基础上追加 trailer 强校验 |

### 5.4 frame_hash_algo

|  值 | 名称      | 说明         |
| -: | ------- | ---------- |
|  0 | none    | 无 frame 校验 |
|  1 | xxh3_64 | 默认推荐       |

### 5.5 manifest_hash_algo

|  值 | 名称         | 说明            |
| -: | ---------- | ------------- |
|  0 | none       | 无 manifest 校验 |
|  1 | blake3_256 | 默认推荐          |

## 6. feature_flags 建议位图

|  Bit | 含义             |
| ---: | -------------- |
|    0 | 含 symlink      |
|    1 | 含 hardlink     |
|    2 | 含 special file |
|    3 | 含扩展元数据 blob    |
|    4 | 含 trailer      |
|    5 | 保留 uid/gid     |
|    6 | 保留 xattrs      |
|    7 | 保留 ACL         |
| 8~63 | 保留             |

## 7. ManifestSection

## 7.1 结构

```text
ManifestSection
= ManifestHeaderV1
| EntryRecordV1[entry_count]
| ExtentRecordV1[extent_count]
| BundlePlanRecordV1[bundle_count]
| NameArena[name_arena_bytes]
| MetaBlob[meta_blob_bytes]
```

其中：

* `EntryRecordV1`：描述文件树节点
* `ExtentRecordV1`：描述文件与 bundle 的映射
* `BundlePlanRecordV1`：描述每个 bundle 的基本信息
* `NameArena`：保存 basename / link target 等原始字节
* `MetaBlob`：扩展元数据 TLV

## 7.2 ManifestHeaderV1

### 大小

固定 **64 bytes**

### 字段布局

| Offset | Size | Field            | Type     | 说明           |
| -----: | ---: | ---------------- | -------- | ------------ |
|      0 |    4 | magic            | [u8; 4]  | 固定为 `MFST`   |
|      4 |    2 | header_len       | u16      | 固定为 64       |
|      6 |    2 | flags            | u16      | manifest 级标志 |
|      8 |    8 | entry_count      | u64      | entry 数      |
|     16 |    8 | extent_count     | u64      | extent 数     |
|     24 |    8 | bundle_count     | u64      | bundle 数     |
|     32 |    8 | name_arena_bytes | u64      | 名字区大小        |
|     40 |    8 | meta_blob_bytes  | u64      | 扩展区大小        |
|     48 |   16 | reserved         | [u8; 16] | 置 0          |

## 7.3 EntryRecordV1

### 大小

固定 **96 bytes**

### 语义

* `entry_id` 不单独存储，默认使用数组下标
* `parent_id` 指向父 entry 的下标
* 根节点固定为 `entry_id = 0`

### 字段布局

| 顺序 | Field                    | Type    | 说明                           |
| -: | ------------------------ | ------- | ---------------------------- |
|  1 | parent_id                | u32     | 父节点 id；根节点为 `u32::MAX`       |
|  2 | kind                     | u8      | 节点类型                         |
|  3 | flags                    | u8      | entry 标志                     |
|  4 | reserved0                | u16     | 置 0                          |
|  5 | mode                     | u32     | Unix mode                    |
|  6 | uid                      | u32     | 用户 id                        |
|  7 | gid                      | u32     | 组 id                         |
|  8 | mtime_sec                | i64     | 秒                            |
|  9 | mtime_nsec               | u32     | 纳秒                           |
| 10 | size                     | u64     | regular file 的原始大小           |
| 11 | name_off                 | u32     | basename 在 NameArena 中的偏移    |
| 12 | name_len                 | u32     | basename 长度                  |
| 13 | link_off                 | u32     | link target 在 NameArena 中的偏移 |
| 14 | link_len                 | u32     | link target 长度               |
| 15 | first_extent             | u64     | 第一个 extent 的索引               |
| 16 | extent_count             | u32     | extent 数                     |
| 17 | hardlink_master_entry_id | u32     | 硬链接主 entry；无则为 `u32::MAX`    |
| 18 | dev_major                | u32     | special file 预留              |
| 19 | dev_minor                | u32     | special file 预留              |
| 20 | meta_off                 | u32     | 扩展元数据偏移                      |
| 21 | meta_len                 | u32     | 扩展元数据长度                      |
| 22 | reserved1                | [u8; 8] | 置 0                          |

### kind 枚举

|   值 | 类型           |
| --: | ------------ |
|   0 | root         |
|   1 | directory    |
|   2 | regular      |
|   3 | symlink      |
|   4 | hardlink     |
|   5 | char_device  |
|   6 | block_device |
|   7 | fifo         |
| 255 | reserved     |

### entry flags 建议

| Bit | 含义            |
| --: | ------------- |
|   0 | 显式 empty file |
|   1 | path 已校验      |
|   2 | 含扩展元数据        |
| 3~7 | 保留            |

## 7.4 ExtentRecordV1

### 大小

固定 **32 bytes**

### 语义

一个 extent 表示某个文件的一段原始字节，位于某个 bundle 的某个 raw offset 上。

### 字段布局

| 顺序 | Field                | Type | 说明                  |
| -: | -------------------- | ---- | ------------------- |
|  1 | bundle_id            | u64  | 归属 bundle           |
|  2 | file_offset          | u64  | 写回文件时的偏移            |
|  3 | raw_offset_in_bundle | u32  | 在 raw bundle 内的起始位置 |
|  4 | raw_len              | u32  | 该 extent 长度         |
|  5 | flags                | u32  | extent 标志           |
|  6 | reserved             | u32  | 置 0                 |

### extent flags 建议

|  Bit | 含义                   |
| ---: | -------------------- |
|    0 | 该 extent 为文件最后一段     |
|    1 | 该 extent 属于聚合 bundle |
|    2 | 该 extent 属于大文件分片     |
| 3~31 | 保留                   |

## 7.5 BundlePlanRecordV1

### 大小

固定 **32 bytes**

### 作用

* 用于描述 manifest 期望的 bundle 列表
* 解压时可校验 frame 序列与 raw_len 是否符合预期
* 便于统计与调试

### 字段布局

| 顺序 | Field                 | Type    | 说明          |
| -: | --------------------- | ------- | ----------- |
|  1 | bundle_id             | u64     | bundle 序号   |
|  2 | raw_len               | u32     | 原始长度        |
|  3 | planned_file_count    | u32     | 涉及文件数量      |
|  4 | expected_extent_count | u32     | 涉及 extent 数 |
|  5 | flags                 | u32     | bundle 标志   |
|  6 | reserved              | [u8; 8] | 置 0         |

### bundle flags 建议

|  Bit | 含义                   |
| ---: | -------------------- |
|    0 | small-file aggregate |
|    1 | large-file chunk     |
|    2 | single-file bundle   |
| 3~31 | 保留                   |

## 7.6 NameArena

存储以下原始字节：

* 每个 entry 的 basename
* symlink 的 link target
* 未来扩展中需要记录的原始路径附加字段

规则：

* basename 不包含 `/`
* 不允许 NUL
* 相对路径通过 `parent_id + basename` 复原
* link target 按原始字节存储，不做 canonicalize

## 7.7 MetaBlob

用于扩展元数据，采用 TLV：

```text
MetaBlob
= TLV*
```

### TLV 建议结构

```text
type: u16
flags: u16
len:  u32
data: [u8; len]
```

### 预留 type

|     值 | 含义        |
| ----: | --------- |
|     1 | xattrs    |
|     2 | POSIX ACL |
|     3 | uname     |
|     4 | gname     |
|     5 | custom    |
| 65535 | reserved  |

## 8. DataFrame

## 8.1 结构

```text
DataFrame
= FrameHeaderV1
| payload[encoded_len]
```

### 说明

* 每个 bundle 对应一个 DataFrame
* frame 按 bundle_id 顺序写出
* 只要顺序消费 frame，即可完成严格流式解压

## 8.2 FrameHeaderV1

### 大小

固定 **48 bytes**

### 字段布局

| Offset | Size | Field       | Type     | 说明         |
| -----: | ---: | ----------- | -------- | ---------- |
|      0 |    4 | magic       | [u8; 4]  | 固定为 `FRM1` |
|      4 |    2 | header_len  | u16      | 固定为 48     |
|      6 |    2 | flags       | u16      | frame 标志   |
|      8 |    8 | bundle_id   | u64      | bundle 序号  |
|     16 |    4 | raw_len     | u32      | 解压后长度      |
|     20 |    4 | encoded_len | u32      | payload 长度 |
|     24 |    8 | frame_hash  | u64      | frame 校验值  |
|     32 |   16 | reserved    | [u8; 16] | 置 0        |

### flags 建议

|  Bit | 含义              |
| ---: | --------------- |
|    0 | payload 压缩      |
|    1 | payload 为空      |
|    2 | 末尾 frame（一般不需要） |
| 3~15 | 保留              |

### frame_hash 规则

* 当 `frame_hash_algo = none` 时置 0
* 当 `frame_hash_algo = xxh3_64` 时，对 **payload 字节** 计算

## 9. TrailerV1

## 9.1 作用

Trailer 只在以下情况下存在：

* `integrity_mode = strong`
* 或者显式启用了 trailer 输出

它**不能**成为正常解压的前置依赖。
即：解压器读取 header + manifest + frame 后即可完成恢复；trailer 只用于最终一致性确认。

## 9.2 大小

固定 **64 bytes**

## 9.3 字段布局

| Offset | Size | Field               | Type     | 说明         |
| -----: | ---: | ------------------- | -------- | ---------- |
|      0 |    4 | magic               | [u8; 4]  | 固定为 `END1` |
|      4 |    2 | header_len          | u16      | 固定为 64     |
|      6 |    2 | flags               | u16      | trailer 标志 |
|      8 |    1 | archive_hash_algo   | u8       | 强校验算法      |
|      9 |    3 | reserved0           | [u8; 3]  | 置 0        |
|     12 |   32 | archive_hash        | [u8; 32] | 整体校验       |
|     44 |    8 | total_raw_bytes     | u64      | 原始总字节数     |
|     52 |    8 | total_encoded_bytes | u64      | 编码总字节数     |
|     60 |    4 | reserved1           | [u8; 4]  | 置 0        |

### archive_hash_algo 建议

|  值 | 名称         |
| -: | ---------- |
|  0 | none       |
|  1 | blake3_256 |

## 10. 协议约束

### 10.1 顺序性约束

* HeaderV1 必须位于归档最开始
* ManifestSection 必须紧随其后
* DataFrame 必须按 `bundle_id` 递增顺序出现
* TrailerV1 如存在，必须位于最后

### 10.2 兼容性约束

* 解压器看到更高 `version_major` 必须拒绝
* 看到相同主版本、更高次版本时，可按 feature_flags 进行兼容判断
* 未识别但被标记为“必需”的 feature，应拒绝
* 未识别但被标记为“可忽略”的 feature，可跳过

### 10.3 路径约束

entry 路径恢复时必须满足：

* 不允许绝对路径
* 不允许空 basename
* 不允许 NUL
* 不允许 `.` / `..` 作为路径段
* 不允许通过 symlink 影响父路径分辨

## 11. 为什么 manifest 不记录压缩后偏移

因为压缩后 frame 的 `encoded_len` 在压缩前未知，若 manifest 记录物理偏移，将导致：

* 需要两遍写 archive
* 或者需要 seek 回填
* 或者先落临时归档

这与“严格顺序流式写出 / 读取”的目标相冲突。

因此 manifest 只记录：

* entry -> extent
* extent -> bundle_id + raw_offset + raw_len

这样解压器只需按 frame 序列顺序读取，即可在拿到 raw bundle 后完成 scatter 写回。

## 12. 协议示例

### 12.1 一个包含 3 个小文件的归档

* HeaderV1
* ManifestSection

  * root dir
  * subdir
  * a.txt
  * b.json
  * c.cfg
  * 3 个 entry 指向同一个 bundle_id = 0
* DataFrame(bundle 0)
* TrailerV1（可选）

### 12.2 一个大文件被切成 3 个 bundle

* HeaderV1
* ManifestSection

  * file.bin 的 `extent_count = 3`
  * 3 条 extent 分别指向 bundle 0 / 1 / 2
* DataFrame(0)
* DataFrame(1)
* DataFrame(2)

## 13. 协议结论

SFA v1 的协议核心是：

> **把目录结构信息前置，把数据体切成独立自描述 frame，并用 manifest 连接“文件世界”和“bundle 世界”。**

这是实现高吞吐、流式与并行三者兼得的关键。

