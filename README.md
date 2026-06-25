# Victoria CLI

Victoria CLI 是一套用于 VictoriaMetrics 生态系统的 Rust 命令行工具集，包含 vlogcli 和 vmcli 两个独立工具。vlogcli 专注于 VictoriaLogs 日志系统的数据管理，支持按时间范围和过滤条件导出日志数据、跨实例导入以及状态监控。vmcli 则面向 VictoriaMetrics 时序数据库，提供高效的指标数据导出与导入功能。两个工具均采用异步架构，支持多线程并行处理、进度可视化、断点续传和重复检测，帮助用户便捷地实现数据迁移和备份恢复。

## 功能特性

- **异步高性能**：基于 Tokio 异步运行时，支持高并发数据处理
- **多线程并行**：导出和导入操作支持并行处理，大幅提升效率
- **进度可视化**：实时进度条显示，直观了解任务执行状态
- **断点续传**：支持记录导出/导入状态，便于异常恢复
- **重复检测**：导入时自动检测已处理文件，避免重复导入
- **灵活过滤**：支持时间范围、过滤条件等多维度数据筛选

## 项目结构

```
victoria-cli/
├── vlogcli/          # VictoriaLogs 日志系统 CLI 工具
│   ├── src/
│   ├── config/
│   └── README.md
└── vmcli/            # VictoriaMetrics 时序数据库 CLI 工具
    ├── src/
    ├── config/
    └── README.md
```

## 安装

### 前置要求

- Rust 1.70+ (edition 2024)
- Cargo

### 编译安装

```bash
# 克隆仓库
git clone https://github.com/your-org/victoria-cli.git
cd victoria-cli

# 编译 vlogcli
cd vlogcli
cargo build --release

# 编译 vmcli
cd ../vmcli
cargo build --release
```

编译后的二进制文件位于 `target/release/` 目录下。

## 快速开始

### vlogcli - VictoriaLogs 日志管理

```bash
# 设置 VictoriaLogs 服务地址
vlogcli -p http://127.0.0.1:9428

# 导出昨天的日志数据
vlogcli export -i "org123" -n "测试组织"

# 导出指定时间范围的日志
vlogcli export -s 2025-08-20T00:00:00Z -e 2025-08-21T00:00:00Z -i "org123" -n "测试组织"

# 导入日志数据
vlogcli import -f /path/to/logs.tar.gz

# 查看状态信息
vlogcli info
```

### vmcli - VictoriaMetrics 指标管理

```bash
# 设置 VictoriaMetrics 服务地址
vmcli address -p http://127.0.0.1:9428

# 导出昨天的指标数据
vmcli export -i "org123" -n "MyOrganization"

# 导出特定时间范围的指标
vmcli export -s "2025-06-01T00:00:00Z" -e "2025-06-01T23:59:59Z" -i "org123" -n "MyOrganization"

# 导入指标数据
vmcli import -f /path/to/metrics.tar.gz

# 查看状态信息
vmcli info
```

## 详细文档

- [vlogcli 使用文档](./vlogcli/README.md)
- [vmcli 使用文档](./vmcli/README.md)

## 技术栈

- **语言**: Rust (Edition 2024)
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **命令行解析**: Clap
- **序列化**: Serde, Serde JSON, TOML
- **日志**: Flexi Logger, Log
- **压缩**: Async Compression, Tokio Tar
- **进度条**: Indicatif
- **哈希**: Blake3

## 许可证

MIT License