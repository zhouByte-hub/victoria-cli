# vmcli 命令行工具使用文档

## 简介

vmcli 是一个用于与 VictoriaMetrics 进行数据导入导出的命令行工具。它提供了简单易用的接口，帮助用户高效地管理和迁移 VictoriaMetrics 中的数据。

## 命令详解

### 1. EXPORT 命令

#### 功能说明

EXPORT 命令用于从 VictoriaMetrics 导出数据。它支持按时间范围和查询条件过滤数据，并将导出的数据保存为压缩文件。

#### 参数说明

- `-q, --query <QUERY>`: 导出数据的过滤条件，默认值为 `{__name__!=""}`
- `-s, --start-time <START_TIME>`: 导出数据的开始时间
- `-e, --end-time <END_TIME>`: 导出数据的结束时间
- `-t, --step <STEP>`: 时间步长（秒），默认值为 60
- `-i, --id <ID>`: 负责导出的组织 ID
- `-n, --name <NAME>`: 负责导出的组织名称

#### 使用示例

##### 示例 1：导出昨天的数据

```bash
vmcli export -i "org123" -n "MyOrganization"
```

此命令将导出昨天的所有数据，使用默认的时间步长（60秒）。

##### 示例 2：导出特定时间范围的数据

```bash
vmcli export -i "org123" -n "MyOrganization" -s "2025-06-01T00:00:00Z" -e "2025-06-01T23:59:59Z"
```

此命令将导出 2025年6月1日 全天的数据。

##### 示例 3：使用自定义查询条件导出数据

```bash
vmcli export -i "org123" -n "MyOrganization" -q '{__name__=~"cpu.*"}'
```

此命令将导出所有名称以 "cpu" 开头的指标数据。

##### 示例 4：使用自定义时间步长导出数据

```bash
vmcli export -i "org123" -n "MyOrganization" -t 30
```

此命令将使用 30 秒的时间步长导出数据。

#### 输出说明

导出的数据将保存在用户主目录下的 `vmcli` 文件夹中，文件名格式为 `YYYY-MM-DD.tar.gz`。每个压缩包包含按时间分段的 JSON 文件和一个元数据文件 `meta.json`，其中包含组织信息和导出时间。

### 2. IMPORT 命令

#### 功能说明

IMPORT 命令用于将之前导出的数据导入到 VictoriaMetrics 中。它支持处理压缩文件，并自动解压和导入数据。

#### 参数说明

- `-f, --file <FILE>`: 要导入的数据文件路径

#### 使用示例

##### 示例 1：导入导出的数据文件

```bash
vmcli import -f "/path/to/exported/data/2025-06-01.tar.gz"
```

此命令将导入指定路径的数据文件。

##### 示例 2：导入用户主目录下的数据文件

```bash
vmcli import -f "~/vmcli/2025-06-01.tar.gz"
```

此命令将导入用户主目录下 vmcli 文件夹中的数据文件。

#### 注意事项

- 导入命令会自动检查文件是否已经导入过，避免重复导入。
- 导入过程中会显示进度条，展示导入进度。
- 导入完成后会自动清理临时文件。

### 3. Info 命令

#### 功能说明

Info 命令用于显示导出/导入的历史信息，包括 VictoriaMetrics 服务地址、最后更新时间、导出状态和导入状态等。

#### 参数说明

该命令不需要任何参数。

#### 使用示例

##### 示例 1：显示历史信息

```bash
vmcli info
```

或使用别名：

```bash
vmcli i
```

#### 输出说明

命令会输出格式化的 JSON 信息，包含以下内容：

- `victoria_metrics_path`: VictoriaMetrics 服务地址
- `update_time`: 最后更新时间
- `export`: 导出状态信息
  - `has_exception`: 是否有异常
  - `export_start_time`: 导出开始时间
  - `export_end_time`: 导出结束时间
  - `export_exception_start_time`: 导出异常开始时间（如果有）
- `import`: 导入状态信息
  - `has_exception`: 是否有异常
  - `import_exception_start_time`: 导入异常开始时间（如果有）

### 4. address 命令

#### 功能说明

address 命令用于设置 VictoriaMetrics 服务的地址。该地址将用于后续的导出和导入操作。

#### 参数说明

- `-p, --path <PATH>`: VictoriaMetrics 服务地址

#### 使用示例

##### 示例 1：设置 VictoriaMetrics 服务地址

```bash
vmcli address -p "http://127.0.0.1:9428"
```

此命令将设置 VictoriaMetrics 服务地址为 `http://127.0.0.1:9428`。

##### 示例 2：设置远程 VictoriaMetrics 服务地址

```bash
vmcli address -p "https://victoria.example.com"
```

此命令将设置远程 VictoriaMetrics 服务地址。

#### 注意事项

- 设置地址前，工具会验证地址的格式是否正确。
- 工具会尝试连接到指定的地址，确保服务可达。
- 地址设置成功后，会更新配置文件并记录更新时间。

