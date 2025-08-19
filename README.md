# CFISH Solana 智能合约项目

## 项目概述

CFISH 是一个基于 Solana 区块链的去中心化 NFT 市场和质押治理平台。该项目包含了完整的智能合约实现，支持 NFT 铸造、交易、质押奖励和去中心化治理功能。

## 智能合约部署信息

- **网络**: Solana Devnet
- **程序 ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`
- **部署状态**: 已成功部署

## 文件说明

### 智能合约代码

1. **cfish_full_contract.rs**
   - 主要的智能合约代码文件
   - 包含 NFT 铸造、交易、质押、治理等核心功能
   - 使用 Anchor 框架开发

2. **cfish_staking_program.rs**
   - 质押相关的独立程序模块
   - 处理代币质押和奖励分发逻辑

### 文档

3. **cfish_smart_contract_summary.md**
   - 智能合约功能和架构的详细说明
   - 包含合约接口和使用方法

## 主要功能

### NFT 功能
- NFT 铸造 (mint_nft)
- NFT 上架销售 (list_nft)
- NFT 购买 (buy_nft)

### 质押功能
- CFISH 代币质押 (stake)
- 解除质押和领取奖励 (unstake)
- 奖励分发 (distribute_reward)
- 锁定奖励释放 (release_vested_reward)

### 治理功能
- 创建治理提案 (create_proposal)
- 投票 (vote)

## 开发环境

- **语言**: Rust
- **框架**: Anchor
- **区块链**: Solana
- **工具链**: anchor-cli 0.31.1

## 注意事项

1. **IDL 文件**: 由于技术限制，IDL 文件未包含在此包中。如需 IDL 文件，请在 Anchor 项目中重新构建生成。

2. **程序 ID**: 智能合约已部署到 Solana Devnet，程序 ID 为 `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`。

3. **前端集成**: 前端应用需要使用此程序 ID 和相应的 IDL 文件来与智能合约交互。

## 使用方法

1. 在 Anchor 项目中使用这些 Rust 文件
2. 配置 `Anchor.toml` 文件中的程序 ID
3. 使用 `anchor build` 构建项目
4. 使用 `anchor deploy` 部署到目标网络

## 联系信息

如有问题或需要技术支持，请参考项目文档或联系开发团队。

