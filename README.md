## 概述

Solarchain netwrok基于Polkadot Substrate，旨在为为企业和个人创作者提供NFT化其IP和艺术品的一站式解决方案。  
Solarchain的runtime核心组件包括：
- frame_system
- pallet_randomness_collective_flip
- pallet_timestamp
- pallet_aura
- pallet_grandpa
- pallet_balances
- pallet_transaction_payment
- pallet_sudo
- pallet_contracts
- pallet_scheduler

组件版本信息如下
```
git = "https://github.com/paritytech/substrate",version = "4.0.0-dev",tag = 'monthly-2021-11-1'
```
## 入门

### 从源码构建
1. 安装rust
```
curl https://sh.rustup.rs -sSf | sh
```
2. 初始化wasm构建环境
```
rustup update nightly
rustup update stable
rustup target add wasm32-unknown-unknown --toolchain nightly
```
3. 克隆代码并构建
```
git clone https://github.com/netwarps/solarchain.git
cd solarchain
cargo build --release
```
4. 检查是否构建成功，了解节点的 CLI 选项的更多信息。
```
./target/release/solar-node --help
```
### 镜像构建
1. 修改node/Cargo.toml版本信息
2. 执行脚本构建docker镜像
```
./scripts/dockerfiles/build.sh
```
3. 推送到镜像仓库

docker镜像地址: `https://registry.paradeum.com/harbor/projects/45/repositories/netwarps%2Fsolarchain `

目前最新版本为:`registry.paradeum.com/netwarps/solarchain:v0.1.4`

### 启动单个开发节点
清除现有的开发链状态
```
./target/release/solar-node  purge-chain --dev
```
启动开发链
```
./target/release/solar-node --dev
```
使用详细的日志记录启动开发链
```
RUST_LOG=debug RUST_BACKTRACE=1  ./target/release/solar-node --dev
```
打开polkadot-js ui查看是否正常
```
http://polkadot.js.paradeum.com/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
```
### 启动单个节点(docker)

```
docker run --name solar-dev-node  \
-p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 \
registry.paradeum.com/netwarps/solarchain:v0.1.4 \
--port 30333 \
--ws-port 9944 \
--rpc-port 9933 \
--prometheus-port 9615 \
--unsafe-ws-external \
--unsafe-rpc-external \
--prometheus-external \
--rpc-cors all \
--dev
```

### 启动多个节点local test network
首先启动 Alice 的节点。以下命令使用默认 TCP 端口 (30333) 并指定 /tmp/alice为链数据库位置。Alice 的节点 ID 是 12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp( 由node-key决定）
```
./target/release/solar-node  \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --prometheus-port 9615 \
  --validator \
  --rpc-cors all 
```
启动后将看到如下打印信息
```
2022-01-17 16:04:52 Solar Node    
2022-01-17 16:04:52 ✌️  version 0.1.4-aa983ccf6c-x86_64-macos    
2022-01-17 16:04:52 ❤️  by netwarps Technologies <admin@netwarps.com>, 2021-2022    
2022-01-17 16:04:52 📋 Chain specification: Local Testnet    
2022-01-17 16:04:52 🏷 Node name: Alice    
2022-01-17 16:04:52 👤 Role: AUTHORITY    
2022-01-17 16:04:52 💾 Database: RocksDb at /tmp/alice/chains/local_testnet/db/full    
2022-01-17 16:04:52 ⛓  Native runtime: solar-node-106 (solar-node-1.tx1.au1)    
2022-01-17 16:04:52 🔨 Initializing Genesis block/state (state: 0x63f2…d716, header-hash: 0x20d3…e3bd)    
2022-01-17 16:04:52 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2022-01-17 16:04:52 ⏱  Loaded block-time = 6s from block 0x20d3f7a5cd70a7015ad3660d3aa463ac472d7589b964abf812ddb40bab86e3bd    
2022-01-17 16:04:52 🏷 Local node identity is: 12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp    
2022-01-17 16:04:52 📦 Highest known block at #0    
2022-01-17 16:04:52 〽️ Prometheus exporter started at 127.0.0.1:9615    
2022-01-17 16:04:52 Listening for new connections on 127.0.0.1:9944.
```

再启动 Bob 的节点, bootnodes指定alice
```
./target/release/solar-node \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
  --chain=local \
  --bob \
  --validator \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --prometheus-port 9616  \
  --rpc-cors all 
```
启动成功后打印以下信息
```
2022-01-17 16:05:52 Solar Node    
2022-01-17 16:05:52 ✌️  version 0.1.4-aa983ccf6c-x86_64-macos    
2022-01-17 16:05:52 ❤️  by netwarps Technologies <admin@netwarps.com>, 2021-2022    
2022-01-17 16:05:52 📋 Chain specification: Local Testnet    
2022-01-17 16:05:52 🏷 Node name: Bob    
2022-01-17 16:05:52 👤 Role: AUTHORITY    
2022-01-17 16:05:52 💾 Database: RocksDb at /tmp/bob/chains/local_testnet/db/full    
2022-01-17 16:05:52 ⛓  Native runtime: solar-node-106 (solar-node-1.tx1.au1)    
2022-01-17 16:05:52 🔨 Initializing Genesis block/state (state: 0x63f2…d716, header-hash: 0x20d3…e3bd)    
2022-01-17 16:05:52 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2022-01-17 16:05:52 ⏱  Loaded block-time = 6s from block 0x20d3f7a5cd70a7015ad3660d3aa463ac472d7589b964abf812ddb40bab86e3bd    
2022-01-17 16:05:52 🏷 Local node identity is: 12D3KooWDR2AGL5WyeuYgthLjTgxxDsr52v5xWeg13NmCQzdCwZe    
2022-01-17 16:05:52 📦 Highest known block at #0    
2022-01-17 16:05:52 〽️ Prometheus exporter started at 127.0.0.1:9616    
2022-01-17 16:05:52 Listening for new connections on 127.0.0.1:9945.    
2022-01-17 16:05:54 🙌 Starting consensus session on top of parent 0x20d3f7a5cd70a7015ad3660d3aa463ac472d7589b964abf812ddb40bab86e3bd    
2022-01-17 16:05:54 🎁 Prepared block for proposing at 1 [hash: 0xaa69ba67ab52e5481c0aa767e71967fd07e214aed845208710395f11381b4df9; parent_hash: 0x20d3…e3bd; extrinsics (1): [0x361a…1b8e]]    
2022-01-17 16:05:54 🔖 Pre-sealed block for proposal at 1. Hash now 0xa7e14aad0b2627610d0219637f07bb4add13924d46b5715072ed21ec1ea6fada, previously 0xaa69ba67ab52e5481c0aa767e71967fd07e214aed845208710395f11381b4df9.    
2022-01-17 16:05:54 ✨ Imported #1 (0xa7e1…fada)    
2022-01-17 16:05:57 💤 Idle (1 peers), best: #1 (0xa7e1…fada), finalized #0 (0x20d3…e3bd), ⬇ 1.5kiB/s ⬆ 1.6kiB/s    
2022-01-17 16:06:00 ❌ Error while dialing /dns/telemetry.polkadot.io/tcp/1024/ws: Custom { kind: Other, error: Other(A(Handshake("server rejected handshake; status code = 200"))) }    
2022-01-17 16:06:00 ✨ Imported #2 (0x416f…14de)    
2022-01-17 16:06:02 💤 Idle (1 peers), best: #2 (0x416f…14de), finalized #0 (0x20d3…e3bd), ⬇ 0.7kiB/s ⬆ 0.6kiB/s    
```

### 启动多个节点local test network(docker-compose)
```
docker-compose  -f scripts/dockerfiles/docker-compose-local.yml up 
```

### 加入solarchain测试网络

创建数据目录
```
mkdir solarnode-data
```
启动容器,需要指定链测试网络配置文件customSpecRaw.json和bootnodes
```
docker run --name solar-test-node -d -v $PWD/solarnode-data:/data/solarnode \
-v $PWD/spec/test/customSpecRaw.json:/config/customSpecRaw.json \
-p 30333:30333 -p 9933:9933 -p 9944:9944 -p 9615:9615 \
registry.paradeum.com/netwarps/solarchain:v0.1.4 \
--base-path /data/solarnode \
--chain /config/customSpecRaw.json \
--port 30333 \
--ws-port 9944 \
--rpc-port 9933 \
--prometheus-port 9615 \
--unsafe-ws-external \
--unsafe-rpc-external \
--prometheus-external \
--rpc-cors all \
--name myNodeName \
--pruning archive \
--sync Full \
--bootnodes /ip4/120.76.157.38/tcp/30333/p2p/12D3KooWQjU2Hn9JowPBNMM7rjC6Uiko3wT7DkQto2BSFq3zbsvi
```

查看节点是否连接, block同步情况
```
http://polkadot.js.paradeum.com/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer/node
```
