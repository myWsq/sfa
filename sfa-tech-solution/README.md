# SFA 技术方案包

> 工作名：**SFA（Streaming Folder Archive）**  
> 定位：**面向海量小文件场景的流式并行归档格式与工具**  
> 核心口号：**tar 语义，非 tar 结构**

## 1. 文档说明

本方案包面向以下目标：

- 将“海量小文件压缩/解压工具”的想法沉淀为可落地的技术方案
- 明确 v1 的目标、边界、协议、架构、模块和实施路线
- 使研发团队可以直接据此进入：
  - 协议评审
  - 架构评审
  - 任务拆解
  - MVP 开发
  - 基准测试

## 2. 文档清单

1. `01-overview.md`  
   项目定位、目标、约束、设计原则与成功标准

2. `02-requirements-and-scope.md`  
   功能需求、非功能需求、范围边界、tar 对齐策略与 v1 取舍

3. `03-architecture.md`  
   总体架构、压缩/解压流水线、并发模型、流式读取与 Unix 文件系统策略

4. `04-format-v1.md`  
   SFA v1 数据协议与二进制格式设计

5. `05-workflows-and-algorithms.md`  
   关键流程、算法、状态机、恢复顺序、错误处理和实现细节

6. `06-module-and-interface-design.md`  
   Rust 工程结构、模块拆分、核心 trait、CLI、库 API、配置模型与错误码

7. `07-reliability-performance-security.md`  
   完整性、性能、资源控制、安全与运维观测设计

8. `08-testing-benchmark-and-roadmap.md`  
   测试、基准、验收、里程碑、风险与后续路线图

## 3. 建议阅读顺序

建议按以下顺序阅读：

1. `01-overview.md`
2. `02-requirements-and-scope.md`
3. `03-architecture.md`
4. `04-format-v1.md`
5. `05-workflows-and-algorithms.md`
6. `06-module-and-interface-design.md`
7. `07-reliability-performance-security.md`
8. `08-testing-benchmark-and-roadmap.md`

## 4. 当前已确认前提

以下是已在方案中固化的前提：

- 使用 Rust 编写
- 工具只提供两个核心能力：
  - 压缩文件夹为单文件
  - 从单文件恢复为文件夹
- 数据协议要求支持严格流式读取，不依赖 seek
- 输入既可能来自本地文件，也可能来自 HTTP 网络流
- 优先级为**吞吐性能优先**
- 可以牺牲一部分压缩比
- 可以接受更高内存占用
- 支持可配置压缩算法
- 协议头中写入压缩算法
- 协议头中写入压缩时的并发建议值
- 解压时从协议头读取算法与建议并发度
- 允许压缩前完整扫描目录
- v1 只在 Unix 系统使用，不解决跨平台语义统一问题
- 目标语义对齐 tar，但**字节格式不与 tar 兼容**
- 对标基线为：**tar + 同压缩算法**

## 5. 核心结论

这个项目不应该做成“更快的 tar 包装器”，而应该做成：

- **前置 manifest**
- **bundle 级小文件聚合**
- **独立 data frame**
- **并行压缩 / 并行解压**
- **严格顺序流式可读**

换句话说：

> 将“文件”从协议和流水线中的核心工作单元，升级为“bundle（小文件聚合块）”。

这是整个性能策略的核心。

## 6. v1 目标摘要

### 必须完成
- 目录压缩为单文件
- 单文件恢复为目录
- 流式解析协议
- 支持 LZ4 / Zstd
- bundle 级并行
- header + manifest + frame 协议
- regular file / directory / symlink / hardlink
- mode / uid / gid / mtime 基本恢复
- fast / strong 两档完整性策略
- 与 tar + 同算法进行性能对标

### 暂不作为 v1 交付重点
- Windows 兼容
- 随机访问归档内单个文件
- 增量备份
- 去重
- 加密
- 远程断点续传
- 全量 xattrs / ACL / device file 恢复

## 7. 文档使用方式

这套文档既可以作为：

- 项目立项文档
- 架构设计文档
- 协议说明文档
- 开发分工依据
- 测试验收依据

也可以直接拆为：

- `spec/format-v1.md`
- `docs/architecture.md`
- `docs/benchmark.md`
- `docs/roadmap.md`

用于开源仓库初始化。

