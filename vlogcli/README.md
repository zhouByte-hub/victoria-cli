# VlogCli 工具使用说明

目前 Vlogcli 处于 v1.2.2 版本。

## 工具概述

VlogCli 是一个用于 VictoriaLogs 日志系统的命令行工具，提供了数据导出、导入和状态管理功能。它支持从 VictoriaLogs 服务器导出日志数据，并将导出的数据导入到其他 VictoriaLogs 实例中，同时提供了状态监控和历史记录功能。

主要功能包括：
- 设置和验证 VictoriaLogs 服务器地址
- 按时间范围和条件导出日志数据
- 将导出的数据导入到 VictoriaLogs 服务器
- 查看导出/导入历史状态信息

#
## 命令详解

### 设置 VictoriaLogs 服务地址

设置 VictoriaLogs 的服务地址是使用工具的第一步，这个命令会验证当前机器与服务器是否能正常通信。

**语法：**
```bash
vlogcli -p http://127.0.0.1:9428
```

**参数说明：**
- `-p, --path`: VictoriaLogs 服务器的完整 URL 地址

**功能说明：**
- 验证 URL 格式是否正确
- 检查与 VictoriaLogs 服务器的 TCP 连接是否可达
- 将配置保存到 `config/status_config.toml` 文件中
- 更新配置时间戳

**示例：**
```bash
# 设置本地 VictoriaLogs 服务器地址
vlogcli -p http://127.0.0.1:9428
```

### 导出数据

导出命令用于从 VictoriaLogs 服务器导出日志数据，支持按时间范围、过滤条件等参数进行数据筛选。

**语法：**
```bash
vlogcli export [OPTIONS]
```

**参数说明：**
- `-f, --filter <FILTER>`: 过滤条件，用于筛选导出的日志数据（默认值："*"）
- `-s, --start-time <START_TIME>`: 导出数据的开始时间（RFC3339 格式）
- `-e, --end-time <END_TIME>`: 导出数据的结束时间（RFC3339 格式）
- `-l, --limit <LIMIT>`: 导出数据的行数限制
- `-t, --step <STEP>`: 时间步长，单位是秒（默认值：60）
- `-i, --id <ID>`: 负责导出的组织 ID
- `-n, --name <NAME>`: 负责导出的组织名称

**功能说明：**
- 支持按时间范围导出日志数据
- 支持多线程并行导出，提高导出效率
- 自动将导出的数据打包成 tar.gz 格式
- 生成元数据文件，记录导出组织和时间信息
- 支持进度条显示导出进度

**示例：**
```bash
# 导出昨天的所有日志数据
vlogcli export -i "org123" -n "测试组织"

# 导出指定时间范围的日志数据
vlogcli export -s 2025-08-20T00:00:00Z -e 2025-08-21T00:00:00Z -i "org123" -n "测试组织"

# 导出特定过滤条件的日志数据，并限制行数
vlogcli export -f "_stream='{\"service\":\"api\"}'" -l 10000 -i "org123" -n "测试组织"

# 导出数据并设置时间步长为1小时
vlogcli export -s 2025-08-20T00:00:00Z -e 2025-08-21T00:00:00Z -t 3600 -i "org123" -n "测试组织"
```

**输出：**
导出结束后会在控制台输出 tar 包的路径，例如：
```
导出的位置为：/Users/username/vlogcli/2025-08-20.tar.gz
```

### 导入数据

导入命令用于将导出的数据导入到 VictoriaLogs 服务器中。

**语法：**
```bash
vlogcli import -f <FILE>
```

**参数说明：**
- `-f, --file <FILE>`: 要导入的 tar 包文件路径

**功能说明：**
- 自动解压 tar 包文件
- 支持递归解压和导入嵌套的 tar 包
- 多线程并行导入，提高导入效率
- 记录导入历史，避免重复导入
- 支持进度条显示导入进度

**示例：**
```bash
# 导入单个 tar 包
vlogcli import -f /data01/vlog/data/2025-08-20T00:00:00Z.tar

# 导入指定路径的 tar 包
vlogcli import -f ./vlogcli/2025-08-20.tar.gz
```

**输出：**
导入成功后会显示：
```
VictoriaLogs data import successfully!
```

### 查看状态信息

状态命令用于查看导出/导入的历史信息，包括当前配置、最近的导出/导入状态等。

**语法：**
```bash
vlogcli info
# 或者使用别名
vlogcli i
```

**功能说明：**
- 显示当前 VictoriaLogs 服务器地址
- 显示配置最后更新时间
- 显示导出状态（开始时间、结束时间、异常状态）
- 显示导入状态（异常状态）
- 以 JSON 格式美化输出状态信息

**示例：**
```bash
# 查看状态信息
vlogcli info

# 使用别名查看状态信息
vlogcli i
```

**输出示例：**
```json
{
  "victoria_logs_path": "http://127.0.0.1:9428",
  "update_time": "2025-08-20 10:30:00",
  "export": {
    "has_exception": false,
    "export_start_time": "2025-08-19T00:00:00Z",
    "export_end_time": "2025-08-19T23:59:59Z",
    "export_exception_start_time": null
  },
  "import": {
    "has_exception": false,
    "import_exception_start_time": null
  }
}
```