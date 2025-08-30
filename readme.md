# ZetaChain Standard Contracts 🚀

ZetaChain Standard Contracts enable cross-chain-ready NFT and Token deployments across multiple blockchain ecosystems. By using ZetaChain as a hub, your tokens and NFTs can move seamlessly between EVM chains and Solana.

## 🌐 Supported Blockchains

- **EVM Chains**: Ethereum, BNB Chain, Polygon, Avalanche, Arbitrum, and more
- **Solana**: Native Solana program with SPL Token integration
- **ZetaChain**: Universal hub for cross-chain coordination

## Contents 📦

### EVM Contracts
- [Universal NFT Documentation](https://www.zetachain.com/docs/developers/standards/nft/)
- [Universal Token Documentation](https://www.zetachain.com/docs/developers/standards/token/)

### Solana Programs  
- [Universal NFT Program](./contracts/solana/universal-nft/) - Complete Solana implementation
- [Solana Integration Guide](./contracts/solana/README.md) - Setup and usage documentation
- [Cross-Chain Demo](./contracts/solana/universal-nft/CROSS_CHAIN_DEMONSTRATION.md) - Working cross-chain transfers

## Installation ⚙️

### For EVM Development

```bash
npm install @zetachain/standard-contracts@v1.0.0-rc2
# or
yarn add @zetachain/standard-contracts@v1.0.0-rc2
```

### For Solana Development

```bash
# Requirements: Rust 1.70+, Solana CLI 1.18+, Anchor 0.30+
git clone https://github.com/zeta-chain/standard-contracts.git
cd standard-contracts/contracts/solana/universal-nft

# Build and test
anchor build
anchor test
```

## Integration Examples 🏗️

### EVM (OpenZeppelin) Integration

Quickly add cross-chain functionality to an existing OpenZeppelin upgradeable contract:

```solidity
// For Universal NFT
import "@zetachain/standard-contracts/contracts/nft/contracts/zetachain/UniversalNFTCore.sol";

// For Universal Token  
import "@zetachain/standard-contracts/contracts/token/contracts/zetachain/UniversalTokenCore.sol";
```

Then inherit from these in your ERC-721 or ERC-20 contract to enable cross-chain transfers.

### Solana Integration

Use the Universal NFT Program for cross-chain NFT functionality:

```rust
// Program ID (Devnet)
declare_id!("Gc1BJg4sYAYGnKBStAHLTdVRLR3fA7DPc7t9G7vjKa1i");

// Cross-chain NFT transfer
burn_for_cross_chain(
    ctx: Context<BurnForCrossChain>,
    destination_chain_id: u64,
    destination_address: Vec<u8>,
)

// Mint from cross-chain message
mint_from_cross_chain(
    ctx: Context<MintFromCrossChain>,
    source_chain_id: u64,
    metadata: CrossChainNftMetadata,
    signature: [u8; 64],
    recovery_id: u8,
)
```

## Using ThirdWeb 🌐

You can also deploy Universal NFTs and Tokens using
[ThirdWeb](https://thirdweb.com/), a powerful web3 development platform. This
allows for easy deployment and interaction with smart contracts across
EVM-compatible blockchains.

- [Deploy Universal NFT on
  ZetaChain](https://thirdweb.com/docs.zetachain.com/ZetaChainUniversalNFT)
- [Deploy Universal NFT on
  EVM](https://thirdweb.com/docs.zetachain.com/EVMUniversalNFT)
- [Deploy Universal Token on
  ZetaChain](https://thirdweb.com/docs.zetachain.com/ZetaChainUniversalToken)
- [Deploy Universal Token on
  EVM](https://thirdweb.com/docs.zetachain.com/EVMUniversalToken)

For examples and more details, check out the [NFT
Docs](https://www.zetachain.com/docs/developers/standards/nft/) and [Token
Docs](https://www.zetachain.com/docs/developers/standards/token/).
