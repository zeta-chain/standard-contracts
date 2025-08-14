# ZetaChain Standard Contracts 🚀

> **⚠️ Important**: For Solana integration, please read [DEVNET_TESTING_NOTICE.md](../DEVNET_TESTING_NOTICE.md) - the current version of ZetaChain localnet may be buggy with Solana.

ZetaChain Standard Contracts enable cross-chain-ready ERC-721 (NFT) and ERC-20
(Token) deployments. By using ZetaChain as a hub, your tokens and NFTs can move
seamlessly between multiple EVM chains.

## Contents 📦

- [Universal NFT
  Documentation](https://www.zetachain.com/docs/developers/standards/nft/)
- [Universal Token
  Documentation](https://www.zetachain.com/docs/developers/standards/token/)

## Installation ⚙️

```bash
npm install @zetachain/standard-contracts@v1.0.0-rc2
# or
yarn add @zetachain/standard-contracts@v1.0.0-rc2
```

## OpenZeppelin Integration 🏗️

Quickly add cross-chain functionality to an existing OpenZeppelin upgradeable
contract:

### For Universal NFT:

```solidity
import "@zetachain/standard-contracts/contracts/nft/contracts/zetachain/UniversalNFTCore.sol";
```

### For Universal Token:

```solidity
import "@zetachain/standard-contracts/contracts/token/contracts/zetachain/UniversalTokenCore.sol";
```

Then inherit from these in your ERC-721 or ERC-20 contract to enable cross-chain
transfers.

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
