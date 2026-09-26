# Lucky Garden ABI

Program ID: `E86BYgK5ZUVNwegBJWXxyCk79XB6dsB5debp3QuuEa9Q`（预留地址；资金到位并实际部署后才成为有效链上地址）。所有 tag 均为 `u64 little-endian`。

| tag | 指令 | 参数 | 账户（按顺序） |
|---:|---|---|---|
| 0 | InitializeConfig | `count:u8`，随后每项 `weight:u16 LE + name:[u8;16]` | payer `signer,w`; config PDA `w`; system program |
| 1 | UpdateConfig | 同上；权重总和必须为 10000 | admin `signer`; config PDA `w` |
| 2 | InitializePlayer | 无 | payer `signer,w`; player PDA `w`; system program |
| 3 | RequestDraw | `game:u8`（0 转盘/1 对对碰），`client_seed:u8` | payer `signer,w`; player `w`; config; VRF queue `w`; program identity `w`; VRF program; slot hashes; system program |
| 4 | CallbackDraw | VRF 注入 `randomness:[u8;32]`，随后原 `game:u8` | scoped VRF identity `signer`; config; player `w` |

PDA：config = `["config"]`；player = `["player", wallet]`；VRF program identity = `["identity"]`。Config 固定 179 bytes：`version + admin + count + weights[8] + names[8][16]`。Player 固定 32 bytes：`version,bump,status,game,prize,reserved[3],utc_day:i64,nonce:u64,requested_slot:u64`。状态 0/1/2 分别为 idle/pending/ready。

安全约束：全部账户数量、owner、PDA、signer、数据长度和参数长度均检查；配置只允许初始化管理员修改；概率总和必须为 10000；每天限制在 VRF 请求前写入，失败交易会原子回滚；回调必须由该程序的 scoped VRF identity 签名，并只消费 pending 且游戏类型匹配的请求。

