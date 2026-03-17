
# 08. 测试、基准、验收与路线图

## 1. 测试策略总览

测试建议分为五层：

1. 单元测试
2. 集成测试
3. golden 兼容测试
4. 损坏与安全测试
5. benchmark 回归测试

## 2. 单元测试

## 2.1 协议结构

* Header encode/decode
* ManifestHeader encode/decode
* EntryRecord / ExtentRecord / BundlePlanRecord 序列化
* FrameHeader / Trailer encode/decode
* feature_flags / 枚举映射

## 2.2 规划器

* 小文件聚合
* 大文件切片
* 空文件处理
* hardlink 主从映射
* deterministic bundle_id 分配

## 2.3 完整性

* Header CRC32
* manifest hash
* frame hash
* trailer hash

## 3. 集成测试

## 3.1 pack -> unpack 正确性

对以下样例进行 roundtrip：

* 空目录
* 单文件
* 多级目录
* 海量小文件
* 混合大小文件
* symlink
* hardlink
* 空文件
* 大文件分片

验证项：

* 目录结构一致
* 文件内容一致
* mode 恢复正确
* mtime 恢复正确
* 默认 / 显式 no-owner 路径不会应用归档内 uid / gid
* symlink target 一致
* hardlink inode 关系一致（同设备下）

## 3.2 CLI 测试

* 参数解析
* 帮助信息
* 参数默认值
* 错误退出码
* stdin 解压

## 4. golden 测试

## 4.1 目的

在协议冻结后，必须为每个典型归档样例保存 golden fixture：

* `.sfa` 二进制样例
* 对应目录树样例
* manifest 文本 dump
* 统计摘要

## 4.2 用途

* 防止协议编码变更导致兼容性破坏
* 为未来多版本兼容提供基线
* 为社区实现或调试提供样例

## 5. 损坏与安全测试

## 5.1 损坏样例

* header magic 错误
* header_crc32 错误
* manifest_hash 错误
* frame_hash 错误
* manifest_encoded_len 截断
* frame payload 截断
* trailer hash 错误

## 5.2 安全样例

* 绝对路径
* `..` 路径
* 空路径段
* NUL 路径
* symlink 指向根外
* special file 输入（默认拒绝）

## 6. 流式测试

## 6.1 目标

验证协议对“输入被任意切碎”时仍然稳定：

* 每次只喂 1 字节
* 每次随机喂 1~64 KiB
* 模拟 HTTP chunk 边界
* 模拟慢速流

## 6.2 必测项

* Header 逐字节解析
* Manifest 分段到达
* Frame payload 分段到达
* strong 模式 trailer 末尾到达

## 7. Fuzz 测试

建议对以下对象做 fuzz：

* Header parser
* Manifest parser
* Frame parser
* 路径验证逻辑
* TLV 扩展元数据解析

目标：

* 不 panic
* 不越界
* 不出现死循环
* 错误分类明确

## 8. benchmark 设计

## 8.1 对标原则

统一以 `tar + 同算法` 为基线。

### 示例

```bash
tar -cf - DIR | lz4  > out.tar.lz4
tar -cf - DIR | zstd -T8 -3 > out.tar.zst

sfa pack DIR --codec lz4  --threads 8
sfa pack DIR --codec zstd --threads 8

sfa unpack out.sfa -C out_dir
tar -xf out.tar -C out_dir
```

## 8.2 数据集建议

### 数据集 A：超多小文本文件

* 源码
* json
* yaml
* toml
* 配置模板

特点：

* 文件数量极多
* 单文件很小
* 可压缩性较好

### 数据集 B：小二进制混合集

* 缩略图
* wasm
* class
* 小 so / 小 dylib
* 索引文件

特点：

* 文件数量较多
* 小文件偏多
* 可压缩性中等

### 数据集 C：少量大文件

* 大日志
* 大镜像层
* 大二进制

特点：

* 作为对照组
* 检查大文件场景不显著退化

## 8.3 指标

* pack wall time
* unpack wall time
* CPU usage
* RSS
* raw bytes/s
* files/s
* output size
* bundle 分布
* frame 校验开销

## 8.4 验收目标

不强行承诺固定倍数，但应达到以下工程目标：

* 在海量小文件集上，相比 `tar + 同 codec` 有明确优势
* 在大文件集上不显著劣化
* 协议稳定、结果可重复
* 性能趋势可以被参数调优解释

## 9. 里程碑建议

以下里程碑不写死工期，只给出交付顺序与完成物。

## 9.1 M0：协议冻结

交付物：

* `spec/format-v1.md`
* 字段级定义
* feature flags 定义
* golden 样例草案

完成标准：

* 通过协议评审
* 明确 v1 Must/Should/Won't

## 9.2 M1：最小可用链路

交付物：

* pack/unpack MVP
* regular file / directory
* LZ4
* fast integrity
* sync unpack
* 初版 CLI

完成标准：

* roundtrip 正确
* 可跑基准
* 能生成第一批 golden 样例

## 9.3 M2：性能主链路

交付物：

* bundle planner 完整版
* ordered writer
* bounded queue
* 多线程 pipeline
* 句柄缓存
* benchmark harness
* tar 基线对比

完成标准：

* 海量小文件场景有明显性能收益
* 各阶段耗时可观测

## 9.4 M3：Unix 语义增强

交付物：

* 收口现有 `mode` / `mtime` / owner policy contract
* metadata roundtrip 与 owner-policy 验证资产
* link / safety / metadata 语义在仓库级测试中可审计
* 路线图与仓库文档同步，明确 deferred 范围

完成标准：

* 当前 v1 Unix metadata 语义有明确 spec、实现与验证资产
* 默认非特权路径与 preserve-owner 分支都可在仓库内追溯
* xattrs / ACL 仍保持 deferred，不混入当前里程碑

当前仓库状态：

* 本阶段已经完成，并作为首个稳定版 `v1.0.0` 的 Unix 语义边界
* 下一步不是继续扩大 M3 范围，而是围绕当前 `main` 的候选修订准备稳定发布

## 9.5 M4：增强能力

交付物：

* async unpack（如仍需要）
* xattrs / ACL（可选）
* 更广的 special file / Unix 扩展元数据能力
* 更多 benchmark 数据集与自动化发版能力

完成标准：

* 扩展元数据能力有独立 spec 与回归资产
* 后续增强不破坏已冻结的 v1 metadata contract

说明：

* M4 属于 `v1.0.0` 之后的增强路线，而不是首个稳定版的发布前置条件

## 10. 风险清单

| 风险               | 现象          | 处理建议                    |
| ---------------- | ----------- | ----------------------- |
| manifest 过大      | 头部读取变慢      | 压缩 manifest + 收紧 record |
| pipeline 调参困难    | 吞吐波动        | 暴露统计与 debug 输出          |
| 磁盘不是瓶颈但 CPU 吃满   | codec/校验开销高 | 调 codec、降低校验等级          |
| 磁盘成为瓶颈           | 线程再加无收益     | 限制 worker，优化 bundle     |
| 安全恢复实现复杂         | 容易踩路径坑      | 统一基于 dirfd 操作           |
| xattrs/ACL 影响稳定性 | 实现分支过多      | 次阶段启用，不入核心闭环            |

## 11. 开源仓库初始化建议

仓库初始化时建议至少包含：

* README
* 协议文档
* 架构文档
* benchmark 说明
* roadmap
* 示例命令
* 版本兼容策略
* 贡献指南

## 12. 最终结论

SFA 项目的正确推进方式不是先写代码再补规范，而是：

1. 先冻结协议与边界
2. 再完成最小闭环
3. 然后用 benchmark 驱动性能优化
4. 最后补充 Unix 语义和扩展元数据

一句话总结：

> **先把“可流式、可并行、能跑赢 tar”的主链路做稳，再扩展功能面。**
