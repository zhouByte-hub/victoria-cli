# victoria-cli
Victoria CLI 是一套用于 VictoriaMetrics 生态系统的 Rust 命令行工具集，包含 vlogcli 和 vmcli 两个独立工具。vlogcli 专注于 VictoriaLogs 日志系统的数据管理，支持按时间范围和过滤条件导出日志数据、跨实例导入以及状态监控。vmcli 则面向 VictoriaMetrics 时序数据库，提供高效的指标数据导出与导入功能。两个工具均采用异步架构，支持多线程并行处理、进度可视化、断点续传和重复检测，帮助用户便捷地实现数据迁移和备份恢复。
