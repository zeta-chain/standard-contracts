
# ZetaMint – Architecture Overview

ZetaMint is a universal NFT bridge program built on Solana that integrates with ZetaChain’s cross-chain messaging gateway to enable secure, trustless NFT minting, burning, and ownership verification across blockchain ecosystems.

---

## High-Level Architecture

```
+------------------+      Cross-Chain Msg       +-----------------------+
|  Solana Program  | <------------------------> |  ZetaChain Gateway    |
| (ZetaMint Anchor)|                            | (EVM/Non-EVM Compatible) |
+------------------+                            +-----------------------+
        |                                                  |
        |                      Relay                       |
        +-------------------- Message ---------------------+
                               (NFT Metadata + Owner)
```

---

## Cross-Chain NFT Lifecycle

### 1. **Mint on Solana**
- A user initiates `mint_nft` from the frontend/CLI
- The Solana program:
  - Mints an NFT
  - Creates associated token account
  - Uses `mpl-token-metadata` to assign metadata (URI, name, symbol)

### 2. **Burn to Initiate Transfer**
- The user calls `burn_nft`
- ZetaMint burns the NFT and emits a log/message with the NFT metadata and destination
- This message is picked up by ZetaChain’s Solana Gateway Contract

### 3. **ZetaChain Processes Message**
- ZetaChain receives the Solana event via its connected gateway
- ZetaChain relays the message to the destination chain (e.g., EVM)
- NFT is re-created with metadata on the target chain (in future extensions)

### 4. **(Optional) Return or Redeem on Solana**
- Same process in reverse to return NFTs to Solana

---

## Program Components

### Anchor Smart Contract (`program/src/lib.rs`)
- `mint_nft` – Mints a unique NFT with metadata and assigns it to a wallet
- `burn_nft` – Burns an NFT and emits a message/log for off-chain relayer
- Uses PDA (Program Derived Addresses) for mint authority
- Uses CPI to call Metaplex Metadata program

### ZetaChain Utility Layer (`zetachain/`)
- Message decoder from ZetaChain format (to be implemented)
- Replay protection (nonce or tx hash)
- Verifier logic to ensure message integrity and signature checks

### Client or Frontend (optional)
- Sends instructions to Solana program
- Displays NFT status
- Tracks cross-chain transfer state via ZetaChain Gateway

---

## Security Architecture

- Signer and authority checks on mint/burn
- Program Derived Addresses (PDAs) for authority
- Log-based message construction (instead of raw payloads)
- Future: hash-based validation, multisig integration

---

## Solana Specific Considerations

| Feature                 | ZetaMint Implementation             |
|------------------------|-------------------------------------|
| Compute Budget         | Uses `sol_compute_budget::request_units` (optional) |
| Rent Exemption         | Lamports allocated for accounts     |
| Token Account Creation | Uses `associated_token::create`     |
| Metadata               | Uses Metaplex's `create_metadata_accounts_v3` |
| Message Relay          | Via ZetaChain Gateway Logs          |

---

## Future Enhancements

- Bidirectional NFT bridging (Solana ↔ Ethereum)
- NFT metadata hashing for on-chain verification
- NFT metadata updater for mutable assets
- Universal NFT ID system (ZetaID)
- Game or Metaverse-specific extensions (e.g., skins, avatars)

---

## Summary

ZetaMint provides a robust, modular base for universal NFTs by combining the high-performance capabilities of Solana with ZetaChain’s powerful cross-chain message relay. It is built for developers, scalable for users, and ready to extend across chains.

