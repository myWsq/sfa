
# 05. 核心流程与算法设计

## 1. pack 全流程

```text
输入目录
  -> 扫描目录树
  -> 规范化 entry
  -> 识别 hardlink
  -> 规划 bundle / extent
  -> 生成 Header + Manifest
  -> 写 Header
  -> 写 Manifest
  -> 读取 bundle 原始数据
  -> 并行压缩
  -> 有序写 Frame
  -> 可选写 Trailer
  -> 完成
```

## 2. unpack 全流程

```text
输入归档流
  -> 读 Header
  -> 校验 Header
  -> 读 Manifest
  -> 校验 Manifest
  -> 构建 RestorePlan
  -> 创建目录树
  -> 顺序读 Frame
  -> 并行解码 raw bundle
  -> 根据 extent scatter 到文件
  -> 创建 symlink / hardlink
  -> 恢复 mode / uid / gid / mtime
  -> strong 模式校验 Trailer
  -> 完成
```

## 3. 目录扫描算法

## 3.1 原则

* 使用 `lstat`，不跟随 symlink
* 先建立完整 entry 列表
* 输出顺序稳定、可重复
* 以相对路径为逻辑标识
* 基于 `(st_dev, st_ino)` 识别 hardlink 组

## 3.2 扫描输出结构

扫描阶段输出：

* `Vec<ScannedEntry>`
* `HashMap<(dev, ino), master_entry_id>`
* `Vec<DirEntryId>` 目录创建顺序
* `Vec<FileEntryId>` 普通文件顺序

### `ScannedEntry` 建议字段

* entry_id
* parent_id
* basename bytes
* entry kind
* mode
* uid
* gid
* mtime
* size
* dev
* ino
* symlink target
* raw source path

## 3.3 稳定排序策略

建议排序为：

1. 目录优先于其内容
2. 同父节点下按 basename 字节序排序
3. 目录扫描输出必须 deterministic

这样可保证：

* 同一目录内容在不同运行中顺序一致
* manifest / bundle_id 稳定
* benchmark 更可比

## 4. hardlink 处理策略

### 4.1 识别

对 regular file：

* 若 `(st_dev, st_ino)` 首次出现，则记为 master
* 若再次出现，则记为 hardlink entry

### 4.2 存储策略

* master entry 按 regular file 存储，拥有 extents
* hardlink entry 不存数据，记录 `hardlink_master_entry_id`

### 4.3 恢复策略

* master 对应文件先恢复
* 所有 hardlink entry 在 master 存在后通过 `linkat` 创建

## 5. bundle 规划算法

## 5.1 输入

* `Vec<ScannedEntry>`
* `bundle_target_bytes`
* `small_file_threshold`

## 5.2 输出

* `Vec<BundlePlan>`
* `Vec<ExtentRecord>`
* `ManifestSection`

## 5.3 规则

1. 非数据类 entry（directory / symlink / hardlink）不参与 bundle 数据体
2. 空文件不占用 bundle 数据体，但保留 entry
3. 小于 `small_file_threshold` 的 regular file 进入聚合 bundle
4. 大于等于阈值的文件，按 `bundle_target_bytes` 分片
5. bundle 以 `bundle_id` 递增分配
6. 同一个 bundle 的原始长度不超过目标上限（最后一个分片可不足）

## 5.4 规划伪代码

```text
bundle_id = 0
current_aggregate = new bundle()

for file in regular_files_in_stable_order:
    if file.is_empty():
        emit empty entry extents = []
        continue

    if file.size < small_file_threshold and !file.is_hardlink():
        if current_aggregate.raw_len + file.size > bundle_target_bytes:
            finalize current_aggregate as bundle_id
            bundle_id += 1
            current_aggregate = new bundle()

        place file into current_aggregate at raw_offset = current_aggregate.raw_len
        create one extent(file_offset=0, raw_offset_in_bundle=..., raw_len=file.size)
        current_aggregate.raw_len += file.size
    else:
        for each chunk in split(file.size, bundle_target_bytes):
            create single-file bundle with this chunk
            create extent(file_offset=chunk.start, raw_offset_in_bundle=0, raw_len=chunk.len)
            assign bundle_id
            bundle_id += 1

if current_aggregate not empty:
    finalize current_aggregate
```

## 5.5 为什么不做复杂装箱算法

v1 不建议做全局 bin packing（如 first-fit decreasing），因为那会：

* 增加规划复杂度
* 打乱文件局部性
* 降低结果可预测性
* 对吞吐收益不一定明显

v1 建议使用**稳定线性聚合**：

* 保持目录与文件顺序局部性
* 便于诊断
* 便于 deterministic output
* 已能覆盖主要性能收益

## 6. pack 写出算法

## 6.1 写出顺序

1. 扫描并规划完成
2. 写 HeaderV1
3. 写 ManifestSection（可压缩）
4. 提交 bundle 读取任务
5. 并行压缩 bundle
6. `OrderedWriter` 按 bundle_id 顺序写 `FrameHeader + payload`
7. 根据完整性策略写 TrailerV1

## 6.2 OrderedWriter 伪代码

```text
next_id = 0
pending = map<bundle_id, EncodedBundle>

while results available:
    pending.insert(result.bundle_id, result)

    while pending contains next_id:
        write_frame(pending[next_id])
        remove pending[next_id]
        next_id += 1
```

### 说明

这样可以保证：

* worker 可乱序完成
* archive 物理顺序依然严格稳定
* 解压器只需顺序读取

## 7. unpack 恢复算法

## 7.1 恢复前准备

在读完 manifest 后，构建：

* `entry_table`
* `bundle_to_extents: Vec<Vec<ExtentRef>>`
* `directory_creation_order`
* `directory_finalize_order`
* `symlink_entries`
* `hardlink_entries`

## 7.2 目录创建顺序

原则：

* 根目录已存在
* 先创建所有目录
* 再恢复 regular file
* 再创建 symlink / hardlink
* 最后恢复目录元数据

## 7.3 raw bundle scatter

解压出 raw bundle 后：

1. 通过 `bundle_id` 找到对应 extents
2. 对每个 extent：

   * 定位目标 file entry
   * 截取 `raw_bundle[raw_offset .. raw_offset + raw_len]`
   * 使用 `write_at(file_offset, slice)` 写入

### 为什么推荐 `write_at`

因为它有两个明显优势：

* 不依赖文件当前 cursor
* 允许未来更容易地扩展到更高并发 scatter

在 Unix-only 场景中，这是一个非常自然的选择。

## 7.4 regular file 恢复伪代码

```text
for extent in bundle_to_extents[bundle_id]:
    file = open_or_get_cached_file(entry_id)
    buf  = raw_bundle[extent.raw_offset_in_bundle .. extent.raw_offset_in_bundle + extent.raw_len]
    write_all_at(file, extent.file_offset, buf)

mark extent done
if all extents done for entry_id:
    finalize file metadata
```

## 7.5 symlink 恢复

symlink 不参与 bundle 数据。

恢复规则：

* 父目录已存在
* 调用 `symlinkat`
* 不跟随 target
* 如目标已存在，依据 overwrite 策略决定报错或替换

## 7.6 hardlink 恢复

恢复规则：

* 必须确认 master 已创建完成
* 调用 `linkat(master, target)`
* 不复制数据
* 如 master 缺失，报一致性错误

## 8. 完整性校验流程

## 8.1 fast

* 校验 Header CRC32
* 校验 manifest hash
* 每个 frame 校验 payload hash

## 8.2 strong

在 `fast` 基础上追加：

* 对整个 archive 的 frame payload / 统计值做最终 hash
* 比对 TrailerV1 中的 archive_hash

## 8.3 失败策略

* Header 不合法：立即失败
* Manifest 校验失败：立即失败
* Frame 校验失败：立即失败
* Trailer 校验失败：已恢复内容需标记为“不可信”

## 9. 错误处理与恢复原则

## 9.1 pack 阶段

* 单个文件 `stat` 失败：默认立即失败；可选 `--skip-unreadable`
* 文件读取失败：立即失败
* 压缩失败：立即失败
* 写 archive 失败：立即失败并返回非零退出码

## 9.2 unpack 阶段

* 协议错误：立即失败
* 路径非法：立即失败
* frame 截断：立即失败
* 校验失败：立即失败
* 文件写入失败：立即失败
* metadata 设置失败：依据策略决定警告或失败

## 9.3 原子性说明

v1 不保证“整个解压过程目录级原子提交”，即：

* 解压失败时目标目录可能已有部分输出

如需更强语义，可后续支持：

* `unpack -> temp dir -> rename`

但这不是 v1 核心路径。

## 10. 状态机设计建议

## 10.1 pack 状态机

```text
Init
 -> Scan
 -> Plan
 -> WriteHeader
 -> WriteManifest
 -> ReadBundles
 -> CompressBundles
 -> WriteFrames
 -> WriteTrailer?
 -> Done
```

## 10.2 unpack 状态机

```text
Init
 -> ReadHeader
 -> ReadManifest
 -> BuildRestorePlan
 -> CreateDirs
 -> ReadFrame*
 -> DecodeFrame*
 -> RestoreData*
 -> RestoreLinks
 -> FinalizeMetadata
 -> VerifyTrailer?
 -> Done
```

## 11. 算法结论

SFA 的核心算法并不复杂，关键在于：

* 扫描顺序稳定
* bundle 规划简单可预期
* 并行压缩、顺序写出
* 并行解码、按 extent scatter

这套模型既适合工程落地，也适合后续持续优化。

