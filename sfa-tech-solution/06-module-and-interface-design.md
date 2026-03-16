
# 06. 模块设计与接口方案

## 1. Rust 工程结构

建议采用 workspace 结构：

```text
sfa/
├── Cargo.toml
├── crates/
│   ├── sfa-core/
│   ├── sfa-unixfs/
│   ├── sfa-cli/
│   └── sfa-bench/
├── spec/
│   └── format-v1.md
├── tests/
│   ├── golden/
│   ├── corruption/
│   ├── streaming/
│   └── compat/
└── benches/
```

## 2. crate 职责拆分

## 2.1 sfa-core

职责：

* Header / Manifest / Frame 编解码
* bundle / extent 规划
* codec 适配
* 完整性校验
* pipeline 协调
* 错误模型
* sync / async 输入状态机

建议模块：

```text
sfa-core/
├── format/
├── manifest/
├── planner/
├── codec/
├── integrity/
├── pipeline/
├── io/
├── stats/
└── error.rs
```

## 2.2 sfa-unixfs

职责：

* Unix 目录扫描
* 元数据采集
* 安全恢复
* dirfd / openat 系列封装
* uid/gid/mode/mtime 应用

建议模块：

```text
sfa-unixfs/
├── scan/
├── restore/
├── metadata/
├── path/
└── file_cache/
```

## 2.3 sfa-cli

职责：

* 参数解析
* 子命令分发
* 进度输出
* 日志初始化
* 错误码映射

## 2.4 sfa-bench

职责：

* benchmark 数据集驱动
* tar 基线封装
* pack/unpack 性能采集
* 结果报告输出

## 3. 核心数据结构

## 3.1 配置对象

### PackConfig

```rust
pub struct PackConfig {
    pub codec: DataCodec,
    pub compression_level: Option<i32>,
    pub threads: usize,
    pub bundle_target_bytes: u32,
    pub small_file_threshold: u32,
    pub integrity: IntegrityMode,
    pub preserve_uid_gid: bool,
    pub preserve_xattrs: XattrMode,
    pub preserve_acl: bool,
    pub follow_symlinks: bool, // 默认 false
}
```

### UnpackConfig

```rust
pub struct UnpackConfig {
    pub threads: Option<usize>,
    pub integrity: IntegrityPolicy,
    pub restore_owner: RestoreOwnerPolicy,
    pub restore_xattrs: bool,
    pub restore_acl: bool,
    pub overwrite: OverwritePolicy,
    pub special_files: SpecialFilePolicy,
}
```

## 3.2 核心模型

### Entry

```rust
pub struct Entry {
    pub entry_id: u32,
    pub parent_id: u32,
    pub kind: EntryKind,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime_sec: i64,
    pub mtime_nsec: u32,
    pub size: u64,
    pub name: Vec<u8>,
    pub link_target: Option<Vec<u8>>,
    pub extents: Vec<ExtentRef>,
    pub hardlink_master: Option<u32>,
}
```

### Extent

```rust
pub struct Extent {
    pub bundle_id: u64,
    pub entry_id: u32,
    pub file_offset: u64,
    pub raw_offset_in_bundle: u32,
    pub raw_len: u32,
    pub flags: u32,
}
```

### BundlePlan

```rust
pub struct BundlePlan {
    pub bundle_id: u64,
    pub raw_len: u32,
    pub file_count: u32,
    pub extent_count: u32,
    pub kind: BundleKind,
    pub slices: Vec<BundleSlice>,
}
```

### EncodedBundle

```rust
pub struct EncodedBundle {
    pub bundle_id: u64,
    pub raw_len: u32,
    pub payload: Vec<u8>,
    pub frame_hash: u64,
}
```

## 4. trait 设计建议

## 4.1 codec 统一接口

```rust
pub trait BundleCodec: Send + Sync + 'static {
    fn codec_id(&self) -> DataCodec;

    fn encode(&self, input: &[u8]) -> Result<Vec<u8>, Error>;

    fn decode(&self, input: &[u8], expected_raw_len: usize) -> Result<Vec<u8>, Error>;
}
```

### 设计说明

* v1 不建议把 codec trait 设计得过于抽象
* 因为 bundle 已经天然是一个“完整输入块”
* `encode/decode` 直接面向内存 buffer 即可
* 若未来要做 streaming codec adapter，可在外层增加 bridge

## 4.2 输入源抽象

### sync

```rust
pub trait ArchiveRead {
    fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}
```

### async

```rust
#[async_trait::async_trait]
pub trait AsyncArchiveRead {
    async fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}
```

### 说明

核心协议解析器最好抽象为“状态机 + 拉取字节”，而不是把所有逻辑直接绑到某个 IO trait 上。

## 4.3 Unix 恢复接口

```rust
pub trait Restorer {
    fn create_dir(&mut self, entry: &Entry) -> Result<(), Error>;
    fn ensure_file(&mut self, entry: &Entry) -> Result<(), Error>;
    fn write_extent(&mut self, entry: &Entry, file_offset: u64, buf: &[u8]) -> Result<(), Error>;
    fn create_symlink(&mut self, entry: &Entry) -> Result<(), Error>;
    fn create_hardlink(&mut self, entry: &Entry, master: &Entry) -> Result<(), Error>;
    fn finalize_entry(&mut self, entry: &Entry) -> Result<(), Error>;
    fn finalize_dirs(&mut self) -> Result<(), Error>;
}
```

## 5. pipeline 组件

### 5.1 PackPipeline

```rust
pub struct PackPipeline {
    pub config: PackConfig,
    pub planner: Planner,
    pub codec: Arc<dyn BundleCodec>,
    pub stats: Stats,
}
```

### 5.2 UnpackPipeline

```rust
pub struct UnpackPipeline {
    pub config: UnpackConfig,
    pub codec: Arc<dyn BundleCodec>,
    pub stats: Stats,
}
```

## 6. CLI 设计

## 6.1 命令格式

```bash
sfa pack <src_dir> -o <archive.sfa> \
  --codec lz4|zstd \
  --level <n> \
  --threads <n> \
  --bundle-size 4M \
  --small-file-threshold 256K \
  --integrity off|fast|strong \
  --preserve-owner \
  --xattrs none|user|all \
  --acl off|on

sfa unpack <archive.sfa|-> -C <dst_dir> \
  --threads <n> \
  --integrity auto|off|fast|strong \
  --restore-owner ignore|preserve \
  --xattrs off|on \
  --acl off|on \
  --special-files deny|allow \
  --overwrite fail|replace
```

## 6.2 CLI 行为约定

* `pack` 时未指定线程数：默认 `available_parallelism()`
* `unpack` 时未指定线程数：默认读取 `suggested_parallelism`，再与本地上限取最优值
* `unpack <archive.sfa|->` 支持从标准输入读取
* 进度输出默认到 stderr

## 7. 对外库 API 设计

## 7.1 sync API

```rust
pub fn pack_dir_to_writer<W: std::io::Write>(
    src_dir: &std::path::Path,
    writer: W,
    config: PackConfig,
) -> Result<PackStats, Error>;

pub fn unpack_reader_to_dir<R: std::io::Read>(
    reader: R,
    dst_dir: &std::path::Path,
    config: UnpackConfig,
) -> Result<UnpackStats, Error>;
```

## 7.2 async API

```rust
pub async fn unpack_async_reader_to_dir<R>(
    reader: R,
    dst_dir: &std::path::Path,
    config: UnpackConfig,
) -> Result<UnpackStats, Error>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static;
```

### 说明

* v1 重点是 pack sync + unpack sync/async
* pack from network 不是目标场景
* async 主要服务于 HTTP body / object storage download 流

## 8. 错误模型

建议错误类型分层：

```rust
pub enum Error {
    Io(IoError),
    Format(FormatError),
    Integrity(IntegrityError),
    Path(PathError),
    Metadata(MetadataError),
    Unsupported(UnsupportedError),
    Internal(InternalError),
}
```

### 子类建议

* `FormatError::BadMagic`
* `FormatError::UnsupportedVersion`
* `FormatError::TruncatedHeader`
* `FormatError::TruncatedManifest`
* `FormatError::InvalidEntryKind`
* `IntegrityError::ManifestHashMismatch`
* `IntegrityError::FrameHashMismatch`
* `IntegrityError::TrailerHashMismatch`
* `PathError::AbsolutePathRejected`
* `PathError::ParentTraversalRejected`
* `UnsupportedError::UnknownCodec`
* `UnsupportedError::SpecialFileDisabled`

## 9. 统计与观测

### 9.1 PackStats

```rust
pub struct PackStats {
    pub entry_count: u64,
    pub regular_file_count: u64,
    pub bundle_count: u64,
    pub raw_bytes: u64,
    pub encoded_bytes: u64,
    pub scan_ns: u128,
    pub plan_ns: u128,
    pub encode_ns: u128,
    pub write_ns: u128,
}
```

### 9.2 UnpackStats

```rust
pub struct UnpackStats {
    pub entry_count: u64,
    pub bundle_count: u64,
    pub raw_bytes: u64,
    pub encoded_bytes: u64,
    pub header_ns: u128,
    pub manifest_ns: u128,
    pub decode_ns: u128,
    pub restore_ns: u128,
}
```

## 10. 依赖策略建议

### 10.1 core 依赖

* 协议和流程尽量自控
* 仅复用成熟 codec / hash / channel / unix syscall 封装

### 10.2 Unix 层依赖

建议引入 dirfd / openat 风格的 syscall 封装库，以提升恢复安全性和路径控制能力。

### 10.3 基准与测试依赖

* benchmark 封装 tar 基线
* 采用 property test / fuzz test 覆盖协议边界

## 11. 模块设计结论

工程组织上建议坚持一个原则：

> `sfa-core` 只做协议与流水线，
> `sfa-unixfs` 只做 Unix 语义与安全恢复，
> `sfa-cli` 只做交互与集成。

这样职责边界清晰，后续：

* 增加新 codec
* 增加 xattrs / ACL
* 增加 async 支持
* 增加 service wrapper

都会更容易演进。

