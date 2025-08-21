# Data Transfer - 数据包传输测试工具

基于 pcapfile-io 包开发的高性能命令行数据包传输测试工具，支持发送和接收 pcap 数据集。

## 功能特性

### 发送功能

- 读取 pcap 数据集并按时间戳顺序发送
- 支持单播、广播、组播三种传输模式
- 可配置发送速率（Mbps）
- 基于原始数据包时间戳的精确时序控制
- 实时显示发送统计信息

### 接收功能

- 接收 UDP 数据包并保存为 pcap 数据集
- 支持单播、广播、组播三种接收模式
- 可设置最大接收包数限制
- 自动生成索引文件以提高后续读取性能
- 实时显示接收统计信息

## 安装

### 前提条件

- Rust 1.70+
- pcapfile-io 库（作为本地依赖）

### 编译

```bash
cd data-transfer
cargo build --release
```

## 使用方法

### 发送数据集

#### 基本单播发送

```bash
data-transfer send -d ./example_dataset -a 192.168.1.100 -p 8080
```

#### 广播发送（指定速率）

```bash
data-transfer send -d ./example_dataset -a 192.168.1.255 -p 8080 -n broadcast -r 50.0
```

#### 组播发送

```bash
data-transfer send -d ./example_dataset -a 224.0.0.1 -p 8080 -n multicast -i eth0
```

### 接收数据包

#### 基本接收

```bash
data-transfer receive -o ./output -d received_dataset -a 0.0.0.0 -p 8080
```

#### 限制接收包数

```bash
data-transfer receive -o ./output -d test_data -a 0.0.0.0 -p 8080 -m 10000
```

#### 组播接收

```bash
data-transfer receive -o ./output -d multicast_data -a 224.0.0.1 -p 8080 -n multicast
```

## 命令行参数

### 通用参数

- `-h, --help`: 显示帮助信息
- `-V, --version`: 显示版本信息

### 发送模式参数

- `-d, --dataset-path <PATH>`: pcap 数据集路径（必需）
- `-a, --address <IP>`: 目标 IP 地址（必需）
- `-p, --port <PORT>`: 目标端口（必需）
- `-n, --network-type <TYPE>`: 网络类型 [可选值: unicast, broadcast, multicast] [默认: unicast]
- `-i, --interface <INTERFACE>`: 网络接口名称（组播/广播时可选）


### 接收模式参数

- `-o, --output-path <PATH>`: 输出目录路径（必需）
- `-d, --dataset-name <NAME>`: 数据集名称（必需）
- `-a, --address <IP>`: 监听 IP 地址（必需）
- `-p, --port <PORT>`: 监听端口（必需）
- `-n, --network-type <TYPE>`: 网络类型 [可选值: unicast, broadcast, multicast] [默认: unicast]
- `-i, --interface <INTERFACE>`: 网络接口名称（组播/广播时可选）
- `-m, --max-packets <COUNT>`: 最大接收包数（可选，0 表示无限制）

## 网络模式说明

### 单播（Unicast）

- 点对点传输
- 目标地址为具体的 IP 地址
- 适用于一对一的数据传输测试

### 广播（Broadcast）

- 向网络内所有主机发送
- 目标地址通常为 x.x.x.255
- 适用于网络性能测试

### 组播（Multicast）

- 向组播组内的主机发送
- 目标地址为组播地址（224.0.0.0-239.255.255.255）
- 适用于一对多的数据分发测试

## 使用示例

### 端到端测试场景

#### 1. 发送端

```bash
# 以50Mbps速率发送测试数据集
data-transfer send \
  --dataset-path ./test_data \
  --address 192.168.1.100 \
  --port 8080 \
  --rate-mbps 50.0
```

#### 2. 接收端

```bash
# 接收数据并保存到新数据集
data-transfer receive \
  --output-path ./received \
  --dataset-name test_received \
  --address 0.0.0.0 \
  --port 8080 \
  --max-packets 100000
```

### 组播测试场景

#### 1. 发送端

```bash
data-transfer send \
  --dataset-path ./multicast_test \
  --address 224.1.1.1 \
  --port 9090 \
  --network-type multicast
```

#### 2. 多个接收端

```bash
# 接收端1
data-transfer receive \
  --output-path ./receiver1 \
  --dataset-name multicast_data1 \
  --address 224.1.1.1 \
  --port 9090 \
  --network-type multicast

# 接收端2
data-transfer receive \
  --output-path ./receiver2 \
  --dataset-name multicast_data2 \
  --address 224.1.1.1 \
  --port 9090 \
  --network-type multicast
```

## 性能特点

- **高精度时序控制**: 基于原始数据包时间戳进行纳秒级精确发送
- **原始模式复现**: 完整保持原始数据包的传输时序特征
- **高性能发送**: 优化的发送逻辑，支持高速数据传输
- **实时统计**: 提供实时的传输速率和统计信息
- **内存高效**: 流式处理，支持大型数据集
- **错误处理**: 完善的错误处理和恢复机制

## 输出信息

### 发送器输出示例

```
INFO 初始化发送器...
INFO 数据集信息:
INFO   文件数量: 3
INFO   数据包总数: 25000
INFO   数据集大小: 12.5 MB
INFO   时间跨度: 10.234 秒
INFO 开始发送数据包 (按原始时序)...
INFO 已发送 1000 包, 当前速率: 55.17 Mbps
INFO 已发送 2000 包, 当前速率: 58.00 Mbps
INFO 发送完成统计:
INFO   发送包数: 25000
INFO   发送字节: 12.5 MB
INFO   错误数量: 0
INFO   用时: 0.08 秒
INFO   平均速率: 60.58 Mbps
INFO   平均包大小: 512 B
```

### 接收器输出示例

```
INFO 初始化接收器...
INFO 开始接收数据包...
INFO   监听地址: 0.0.0.0:8080
INFO   网络类型: 单播
INFO   输出路径: ./received
INFO   数据集名称: test_received
INFO   最大包数: 100000
INFO 按 Ctrl+C 停止接收
INFO 已接收 1000 包, 512.0 KB, 速率: 45.23 Mbps
INFO 已接收 2000 包, 1.0 MB, 速率: 47.89 Mbps
INFO 收到停止信号，正在结束接收...
INFO 正在完成数据集写入...
INFO 接收完成统计:
INFO   接收包数: 25000
INFO   接收字节: 12.5 MB
INFO   错误数量: 0
INFO   用时: 10.67 秒
INFO   平均速率: 48.45 Mbps
INFO   平均包大小: 512 B
```

## 许可证

MIT License

## 依赖项目

- [pcapfile-io](../pcapfile-io) - 高性能 PCAP 文件读写库
