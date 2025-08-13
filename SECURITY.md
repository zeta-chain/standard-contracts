
# Security Overview – ZetaMint

ZetaMint is built with security as a top priority, especially considering its cross-chain functionality and interaction with external gateway systems. This document outlines the key security features and protection mechanisms used in the ZetaMint Solana program.

---

## Security Objectives

- **Prevent replay attacks** from malicious actors relaying the same message multiple times.
- **Validate signer authority** for NFT minting and burning operations.
- **Ensure message integrity** when interacting with ZetaChain gateway protocols.
- **Safeguard token accounts and metadata** from unauthorized manipulation.

---

## Key Security Features

### 1. **Replay Protection**
- Plan to use **message IDs, timestamps**, or **nonces** included in ZetaChain messages.
- These nonces can be stored in Solana PDAs to ensure each message is executed only once.

### 2. **Signer Verification**
- All minting and burning actions require the user's **valid signer signature** (`Signer<'info>`).
- Program Derived Addresses (PDAs) are used for internal authority control (e.g., mint authority).

### 3. **Cross-Chain Message Verification (Planned)**
- Future support to verify message authenticity from ZetaChain using:
  - Gateway-signed messages
  - Checksum/hash validation
  - Multi-signature (TSS) based confirmation

### 4. **Token Account Security**
- Token accounts are **created with Associated Token Program** to prevent spoofing.
- Rent-exemption ensures accounts are not garbage collected.

### 5. **Safe Metadata Handling**
- Uses **CPI** to call `mpl-token-metadata` for secure metadata setup.
- Metadata can only be set at mint time with proper authority.

---

## Threat Model

| Threat                        | Mitigation Strategy                          |
|------------------------------|----------------------------------------------|
| Replay Attacks               | Nonce system (planned)                       |
| Unauthorized Minting         | Signer + PDA authority checks                |
| Fake ZetaChain Messages      | Signature/Hash verification (future plan)    |
| Account Tampering            | Rent exemption + proper initialization       |
| Metadata Spoofing            | Controlled via `create_metadata_accounts_v3` |

---

## Best Practices Followed

- Anchor framework with strict type enforcement
- Separation of concerns in accounts
- Use of Seeds + Bumps for secure PDAs
- Modular structure for future auditing and extensibility

---

## Future Improvements

- Full message verification with ZetaChain gateway signatures
- On-chain hash validation of NFT metadata
- NFT blacklist or freeze list for malicious assets

---

## Reporting Security Issues

If you discover a vulnerability, please contact the maintainer directly via GitHub or email before disclosing it publicly.

---

Stay secure. Build decentralized. 
