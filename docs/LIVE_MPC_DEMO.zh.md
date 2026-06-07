# 真实 MPC 现场演示 — 原始 CLI、可独立验证

> 英文原文 / English: [LIVE_MPC_DEMO.md](./LIVE_MPC_DEMO.md)

**适用对象：** 负责执行本次演示的操作者，以及在现场观看的（持怀疑态度的）投资人。
**目标：** 在持怀疑态度的投资人面前证明，这是**运行在真实、彼此独立的多台机器上的真实门限密码学**——而不是一段假装的脚本。每一条命令都是原始且可见的；
最终的签名由**一个并非我们编写的工具**（Node.js / Python / OpenSSL）来验证，
并且其公钥是一个**真实的 Solana 地址**。

> **让这一切无法造假的核心思路：** 我们在 **ed25519** 曲线上运行整个仪式。一个 FROST-ed25519
> 门限签名就是一个*完全标准的* RFC-8032 Ed25519 签名。因此投资人可以用自己笔记本上的密码学库来验证它，
> 完全无需了解我们的任何代码。如果一个被普遍信任的库说"有效"，那它就是有效的。（在 secp256k1
> 上我们的签名遵循 RFC-9591，没有任何现成工具能直接校验——所以我们刻意选择在 ed25519 上做演示。）

---

## 0. 本次演示证明了什么（以及它如何回应持怀疑态度者的质疑）

| 投资人的疑虑 | 演示如何回应 |
|---|---|
| "这只是一个程序在假扮多个参与方。" | **三个彼此独立的进程**（理想情况下是三台笔记本），每个都有**自己的 keystore 文件**，每个都独立打印自己的结果。 |
| "是你们的工具说它有效——也许你们的工具在撒谎。" | 签名由 **Node.js / Python / OpenSSL 内置的密码学功能**验证——这些代码不是我们写的——而且这个公钥是一个**真实的 Solana 地址**。 |
| "也许某一台机器其实偷偷掌握了完整的密钥。" | **没有任何一台机器能够独自签名。** 在一个 2-of-3（门限：任意两台即可签名）的钱包中，单台设备尝试签名只会**超时**。必须两台协作。（现场证明见 §6。） |
| "这是预先录好的 / 套路化的。" | 由投资人**当场指定要签名的消息**。签名随之改变；但它依然能通过验证。 |
| "密钥是一次性生成并硬编码进去的。" | 通过分布式密钥生成（DKG）**现场创建一个全新的钱包**；每次运行密钥都不同，且取决于全部三台机器的随机性。 |

---

## 1. 角色与前置条件

- **三位参与者**——我们称之为 **alice**、**bob**、**carol**。理想情况下是三台独立的笔记本；
  在一台机器上开三个终端也可以（说服力稍弱，但密码学完全相同）。
- 每台机器都需要 `mpc-wallet-cli` 二进制文件：
  ```bash
  cargo build --release -p mpc-wallet-cli
  # binary: ./target/release/mpc-wallet-cli
  ```
- 需要互联网访问（本演示使用托管的信令服务器 `wss://panda.qzz.io`）。
  - *没有互联网？* 见 §7——由一台笔记本在局域网上运行服务器。
- 投资人机器上需要一个验证工具：**Node.js**（最简单），或带有
  `cryptography`/`PyNaCl` 的 Python，或 `openssl`。全部示例见 §5。

> **协议传输方式。** 每个节点都运行 `mpc-wallet-cli serve`，它使用**以换行符分隔的 JSON**
> 进行通信：你在 stdin 上输入一个命令对象，它在 stdout 上打印事件对象。
> 投资人是真的能看到底层的通信协议——没有任何东西被隐藏。

---

## 2. 一次性准备：共享的房间

托管服务器是多租户的。同一个钱包的每位参与者都必须使用**同一个强随机的房间 id**（≥16 个字符）。
由一个人生成它并分享这个确切的值：

```bash
ROOM=$(uuidgen | tr -d -)      # e.g. 7f3a9c2e4b1d4e8a9c2f001122334455
echo "$ROOM"                   # send this exact string to bob and carol
```

也为本次演示选一个共享密码（例如 `demo`）。在真实部署中每台设备都有自己的密码；
为了演示方便，共用一个能让流程更简单。**永远不要把真实密码打在演示幻灯片上。**

---

## 3. 启动三个节点

每人运行**一条**命令（替换为各自的名字）。保持这些终端打开并可见。

```bash
# alice
./target/release/mpc-wallet-cli serve --curve ed25519 \
  --device-id alice --keystore ~/.frost_alice \
  --signal-server wss://panda.qzz.io --room "$ROOM"

# bob
./target/release/mpc-wallet-cli serve --curve ed25519 \
  --device-id bob --keystore ~/.frost_bob \
  --signal-server wss://panda.qzz.io --room "$ROOM"

# carol
./target/release/mpc-wallet-cli serve --curve ed25519 \
  --device-id carol --keystore ~/.frost_carol \
  --signal-server wss://panda.qzz.io --room "$ROOM"
```

每个节点都会打印：
```json
{"event":"ready","protocol":1,"device_id":"alice","curve":"ed25519"}
{"event":"connection","connected":true}
```

> **设备 id 必须唯一。** 两个节点使用相同的 `--device-id` 会在服务器上发生冲突，
> 网状连接（mesh）将无法建立。alice / bob / carol——必须各不相同。

---

## 4. 创建钱包，然后签名（现场仪式）

下面的所有内容都是输入到某个节点终端（它的 stdin）中的。输入 JSON 然后按回车。

### 4a. alice 创建一个 2-of-3 钱包（分布式密钥生成）

在 **alice** 的终端中：
```json
{"cmd":"create_wallet","threshold":2,"total":3,"password":"demo"}
```

alice 会立即打印出会话 id——**把它读出来**：
```json
{"event":"session_announced","session_id":"dkg_8f1c…"}
```

### 4b. bob 和 carol 加入该会话

在 **bob** 和 **carol** 的终端中（使用 alice 刚刚宣布的那个 id）：
```json
{"cmd":"join_session","session_id":"dkg_8f1c…","password":"demo"}
```

几秒钟后，**三个**终端都会各自独立地打印出相同的结果：
```json
{"event":"dkg_complete","wallet_id":"…","address":"<Solana base58 address>","group_public_key":"<64 hex chars>"}
```

> 🎤 **讲解要点：** "三台彼此独立的机器刚刚共同生成了一个共享钱包。没有任何一台
> 持有完整的私钥——每台只持有一个*密钥分片（share）*。请注意，三台机器都各自独立地打印出了
> **相同**的公钥和**相同**的 Solana 地址。"
>
> 向投资人展示这三个 keystore 文件确实存在且各不相同：
> ```bash
> ls -la ~/.frost_alice ~/.frost_bob ~/.frost_carol   # three separate shares on disk
> ```

### 4c. 对投资人指定的消息进行签名

请投资人给出一句话。把它放进 **alice** 的终端（`message` = 他们说的内容）：
```json
{"cmd":"sign","wallet_id":"<wallet_id from dkg_complete>","message":"we closed the round","encoding":"utf8","password":"demo"}
```

**bob** 会看到一个审批请求并予以同意（使用 bob 打印出的 `sign_…` id）：
```json
{"event":"signing_request","session_id":"sign_3a2e…","wallet":"…"}
```
```json
{"cmd":"approve_signing","session_id":"sign_3a2e…","password":"demo"}
```

alice 打印出完成的签名：
```json
{"event":"signature_complete","signature":"0x<128 hex chars>","message_hash":"…"}
```

> 🎤 **讲解要点：** "三台设备中的两台刚刚共同为投资人指定的那条确切消息进行了签名。
> 现在让我们用一个我们谁都没有编写的工具来证明它是真实的。"

---

## 5. 决定性的一幕：独立验证

把本次运行得到的**三个值**交给投资人：

- **GK** —— 来自 `dkg_complete` 的 `group_public_key`（64 个十六进制字符）
- **SIG** —— 来自 `signature_complete` 的 `signature`，**去掉开头的 `0x`**（128 个十六进制字符）
- **MSG** —— 你签名的那条确切的消息字符串（例如 `we closed the round`）

投资人在**他们自己的机器上**运行以下**任意一个**：

### Node.js（内置 `crypto`，无需安装）
```bash
node -e '
const crypto=require("crypto");
const GK="PASTE_GK", SIG="PASTE_SIG_NO_0x", MSG="we closed the round";
const der=Buffer.concat([Buffer.from("302a300506032b6570032100","hex"),Buffer.from(GK,"hex")]);
const pub=crypto.createPublicKey({key:der,format:"der",type:"spki"});
console.log("VERIFIED:", crypto.verify(null, Buffer.from(MSG), pub, Buffer.from(SIG,"hex")));
'
# → VERIFIED: true
```

### Python（`cryptography`）
```bash
python3 -c '
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
GK="PASTE_GK"; SIG="PASTE_SIG_NO_0x"; MSG=b"we closed the round"
Ed25519PublicKey.from_public_bytes(bytes.fromhex(GK)).verify(bytes.fromhex(SIG), MSG)
print("VERIFIED: True")   # raises + prints nothing if invalid
'
```

### Python（`PyNaCl`）
```bash
python3 -c '
from nacl.signing import VerifyKey
GK="PASTE_GK"; SIG="PASTE_SIG_NO_0x"; MSG=b"we closed the round"
VerifyKey(bytes.fromhex(GK)).verify(MSG, bytes.fromhex(SIG)); print("VERIFIED: True")
'
```

### OpenSSL
```bash
# portable hex→binary helper (works without xxd):
hex2bin(){ python3 -c "import sys,binascii;open(sys.argv[2],'wb').write(binascii.unhexlify(sys.argv[1]))" "$1" "$2"; }

hex2bin "302a300506032b6570032100PASTE_GK" pub.der     # 12-byte SPKI prefix + key
openssl pkey -pubin -inform DER -in pub.der -out pub.pem
printf '%s' "we closed the round" > msg.bin
hex2bin "PASTE_SIG_NO_0x" sig.bin
openssl pkeyutl -verify -pubin -inkey pub.pem -rawin -in msg.bin -sigfile sig.bin
# → Signature Verified Successfully
```

### 附加亮点：它是一个真实的 Solana 地址
来自 `dkg_complete` 的 `address` 就是同一个 32 字节密钥的 base58 编码——一个有效的 Solana
账户。把它粘贴到 <https://explorer.solana.com>，即可展示这是一个真实的、格式正确的、
由这个钱包所控制的账户。

> 🎤 **收尾陈词：** "你自己的密码学库——而不是我们的——刚刚确认了：一个针对*你*所选择的消息的签名，
> 在一个本身就是真实 Solana 地址的密钥下是有效的，而它是由三台彼此独立、各自只持有一个碎片的机器中的两台
> 共同产生的。这就是现场的门限 MPC。"

---

## 5b. 进阶：在 Solana 区块链上完成一笔真实交易

最有冲击力的版本：这个 MPC 钱包对一笔真实的 Solana 转账进行签名，并且它真的上链，
能在一个公开的区块浏览器里看到。分工是这样的（这正是让它可信的关键）：由**标准的 `@solana/web3.js` 库**
来构建并提交交易；我们原始的 `mpc-wallet-cli` **只负责签名**。辅助脚本：`scripts/demo/solana_onchain.mjs`。

> **务必从 `group_public_key` 派生地址**（`solana_onchain.mjs address
> <groupKeyHex>`），而不要使用 `dkg_complete` 事件中的 `address` 字段——该字段
> 目前对 ed25519 并不可靠（已单独跟踪处理）。

### 演示前准备（在你上台之前完成——现场的水龙头有速率限制）
钱包地址需要一点 SOL 来支付转账和手续费。公开的 devnet 空投
API 限速很严，所以**请提前给地址充值**：
```bash
node scripts/demo/solana_onchain.mjs address <groupKeyHex>     # -> the Solana address
# then fund it via the web faucet (https://faucet.solana.com, has a captcha)
# or transfer ~0.01 SOL from any funded devnet wallet. Confirm it's funded:
node scripts/demo/solana_onchain.mjs airdrop <groupKeyHex> 1   # works only if not rate-limited
```

### 台上——两种呈现方式（根据你是否提前充值来选择）

**(i) 与资金无关的证明（不会被限速——推荐作为稳妥的默认方案）：**
```bash
node scripts/demo/solana_onchain.mjs prepare <groupKeyHex> self 1000   # prints MESSAGE hex
# MPC-sign that message (2-of-3): in alice's serve terminal
#   {"cmd":"sign","wallet_id":"…","message":"<MESSAGE hex>","encoding":"hex","password":"demo"}
#   bob: {"cmd":"approve_signing","session_id":"sign_…","password":"demo"}
node scripts/demo/solana_onchain.mjs verify <signatureHex>             # -> web3.js tx.verifySignatures(): true
```
> 🎤 "标准的 Solana 库刚刚确认了我们的门限签名对于一笔真实的 Solana 交易是有效的
> ——完全不需要信任我们的代码。"

**(ii) 完整上链（如果地址已提前充值）：** 同样执行 `prepare` + MPC 签名，然后
```bash
node scripts/demo/solana_onchain.mjs finalize <signatureHex>          # submits; prints the explorer URL
```
在投影仪上打开打印出来的 `https://explorer.solana.com/tx/…?cluster=devnet` 链接。
> 🎤 "那笔交易刚刚在一条公开的区块链上完成了结算——它是由三台从未把私钥拼装到一起的机器中的两台
> 授权的。"

> **已验证：** 地址派生与 web3.js 一致；对于一笔真实的转账，MPC 签名能够通过 web3.js 的
> `verifySignatures()`；唯一卡住 `finalize` 的因素是账户余额（机制已被证明——详见脚本头部）。
> `prepare → sign → finalize` 必须在约 60 秒内完成（blockhash 的有效期），所以请让 MPC 节点
> 提前处于运行状态。

---

## 6. 可选但极具说服力：“单台设备无法独自签名”

这是关于密钥确实被分割开来的最直观的证明。

1. 在 **alice** 的终端中，发起一次签名**但让任何人都不去批准**：
   ```json
   {"cmd":"sign","wallet_id":"…","message":"alice alone","encoding":"utf8","password":"demo"}
   ```
2. 等待。在门限为 2 而只有 alice 参与的情况下，这个仪式**无法完成**
   ——它会超时，且没有产生任何签名。
3. 现在让 bob 也批准，再重复一次 → 它就完成了。

> 🎤 "单凭一台机器，是无能为力的。门限是由数学强制保障的，而不是靠制度规定。"

---

## 6b. 恢复与轮换：“丢失一台设备并不意味着丢失钱包”

这是每位投资人都会问到的、关于多设备钱包的问题。用于**密钥分片刷新 / 重新分享（resharing）**
的密码学引擎已经交付，并且可以用一条命令来演练——它无需任何网络配置就能证明整个恢复故事：

```bash
# Rotate all shares (proactive security) — same wallet, fresh shares:
mpc-wallet-cli reshare-simulate --nodes 3 --threshold 2 --curve ed25519

# Remove a lost/stolen device (2-of-3 → keep only devices 1 & 2):
mpc-wallet-cli reshare-simulate --nodes 3 --threshold 2 --curve ed25519 --keep 1,2
```

两者都会打印出相同的结构：
```json
{ "kept": [1,2], "group_public_key": "06833fdf…badb6ac8",
  "key_preserved": true, "refreshed_quorum_signs": true, "old_share_rejected": true, "ok": true }
```

需要重点指出的地方：
- **`group_public_key` 前后完全相同**，无论你是轮换还是剔除一台设备它都相同 → **地址永远不变**；
  无需移动资金，无需重新充值。
- **`refreshed_quorum_signs: true`** → 钱包在使用新的密钥分片后依然正常工作。
- **`old_share_rejected: true`** → 每一个刷新前的密钥分片现在都**已失效**——被盗设备上的碎片
  再也无法组合起来去签名了。

> 🎤 "丢了一台笔记本？把密钥分片刷新到剩下的设备上即可——地址不变，钱包照常工作，
> 而那台丢失设备上的密钥分片现在已一文不值。我们还可以按计划定期轮换，
> 这样一来，一个用数月时间收集碎片的攻击者也永远拼不出完整密钥。单一密钥的托管钱包
> 做不到这其中任何一点。"

> **如实说明范围：** `reshare-simulate` 在**单个进程内**运行真实的刷新过程（就像 §7 中的"核弹级"兜底方案一样）
> ——它证明的是密码学本身。**联网的**多设备重新分享仪式
> （通过 WebRTC 网状连接进行，类似 DKG）正在进行中；引擎、驱动、
> 标识符规则以及原子化的 keystore 替换均已构建并完成测试，异步网状连接的接线
> 以及多节点测试在 issue #56 中跟踪。在现场演示中，请将其呈现为"恢复所依赖的数学，正在运转"
> ——其分量与核弹级的 `simulate` 证明相当。完整的威胁模型 + 讲解要点见
> `docs/RECOVERY_AND_RESHARING.md`。

---

## 7. 兜底阶梯（如果网络出问题）

请准备好这些方案；你绝不应该陷入卡死的境地。

| 档位 | 适用时机 | 操作 |
|---|---|---|
| **0. 现场联网** | 正常情况 | `wss://panda.qzz.io` + 一个共享的 `--room`（即上文的步骤）。 |
| **1. 局域网服务器** | 互联网不稳定 | 由一台笔记本运行服务器：`MPC_SIGNAL_BIND=0.0.0.0:9000 cargo run --release -p webrtc-signal-server`。所有人使用 `--signal-server ws://<that-laptop-LAN-ip>:9000`（本地服务器**不需要**房间）。同样的演示，无需互联网。 |
| **2. 核弹级（不可能失败）** | 一切都乱套了 | 在一台机器上运行：`./target/release/mpc-wallet-cli simulate --nodes 3 --threshold 2 --curve ed25519 --sign "we closed the round"`。在约 3 秒内完成完整的 DKG + 签名 + 一个可验证的签名，自成一体。然后用 §5 验证。震撼力稍弱（只有一台机器），但**密码学完全相同，并且依然可独立验证**。 |

**飞行前检查（在每台机器上提前 10 分钟运行）：**
```bash
SIGNAL=wss://panda.qzz.io scripts/demo/preflight.sh
# proves the local crypto stack AND a real ceremony through the live server. Green = go.
```

---

## 8. 故障排查

| 现象 | 原因 / 解决办法 |
|---|---|
| `WebSocket … 400` / 连接被拒绝 | `--room` 缺失或强度不够（托管服务器要求 ≥16 个字符）。用 `uuidgen \| tr -d -` 生成；所有人使用**同一个**值。 |
| 一直停在 "Waiting for participants" | 并非所有人都在**同一个房间**和**同一个服务器**上；或者 `--device-id` 重复。请检查全部三台。 |
| bob/carol 始终看不到会话 | 他们在 alice 宣布会话之后才连上来——他们只需用 alice 打印出的 id 发送 `join_session` 即可。（即使迟连，加入也能成功。） |
| 验证器返回 **false** | `MSG` 的字节不对（必须是被签名的那条**确切**消息），或者 `SIG` 开头的 `0x` 没去掉，或者 GK/SIG 在誊抄时打错了字。请重新复制。 |
| 签名能通过验证但地址看起来很奇怪 | `address` 是 base58 编码（Solana 格式）；`group_public_key` 是十六进制。它们是同一个密钥的两种编码。 |

---

## 9. 底层原理（面向会追问的技术型投资人）

- **DKG（密钥生成）：** FROST 分布式密钥生成，Pedersen 变体——**没有可信的发牌方**。
  每台设备都贡献随机性；私钥在任何地方都不会被拼装出来。每台设备最终得到一个*密钥分片（share）*；
  组公钥是公开的。
- **签名：** FROST 门限 Schnorr。`n` 台设备中的 `t` 台各自产生一个部分签名；
  这些部分签名聚合成**一个普通的签名**，它能在组公钥下被一个标准的验证器验证通过。
  没有任何设备会看到其他设备的密钥分片。
- **曲线：** ed25519（RFC 8032）。组公钥就是一个普通的 Ed25519 公钥（一个 Solana
  地址）；签名就是一个普通的 Ed25519 签名——这正是 §5 中得以独立验证的原因。同一套软件
  也能运行 secp256k1（以太坊/比特币系）；我们之所以专门演示 ed25519，是因为它能被现成工具验证。
- **传输：** 用一个信令服务器做发现 + WebRTC 来承载密钥分片所流经的那条加密的点对点
  网状连接（mesh）。信令服务器永远看不到任何密钥材料。
- **恢复 / 托管说明：** 由于没有发牌方，单台设备的种子本身无法重建出它的密钥分片
  ——每台设备上那个加密的 keystore 才是备份的单元。
  （见 `docs/MULTI_CURVE_DERIVATION.md`。）

---

## 10. 速查卡（把它打印出来）

```
SETUP   ROOM=$(uuidgen | tr -d -)               # share this exact value
NODE    mpc-wallet-cli serve --curve ed25519 --device-id <name> \
        --keystore ~/.frost_<name> --signal-server wss://panda.qzz.io --room "$ROOM"
CREATE  (alice) {"cmd":"create_wallet","threshold":2,"total":3,"password":"demo"}
JOIN    (bob,carol) {"cmd":"join_session","session_id":"dkg_…","password":"demo"}
SIGN    (alice) {"cmd":"sign","wallet_id":"…","message":"<investor's words>","encoding":"utf8","password":"demo"}
APPROVE (bob)   {"cmd":"approve_signing","session_id":"sign_…","password":"demo"}
VERIFY  node -e '…'   # §5 — investor runs it; → VERIFIED: true
RECOVER mpc-wallet-cli reshare-simulate --nodes 3 --threshold 2 --curve ed25519 [--keep 1,2]
        # §6b — same address, refreshed shares, old share dead → "ok": true
NUKE    mpc-wallet-cli simulate --nodes 3 --threshold 2 --curve ed25519 --sign "…"   # §7 rung 2
```
