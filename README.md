
# ZetaMint – Solana ↔ ZetaChain Universal NFT Program

> **Mint once, roam everywhere – NFTs without borders.**

ZetaMint is a cross-chain NFT program built on Solana that enables seamless minting, burning, and transferring of NFTs across chains using **ZetaChain's universal messaging gateway**.

---

## Features

- Mint NFTs on Solana with full metadata (URI, title, symbol)
- Burn NFTs on Solana to trigger cross-chain migration
- Prepare messages compatible with ZetaChain's gateway protocol
- Scalable, modular, and developer-friendly
- Security-first with replay protection and signer validation

---

## Installation

### 1. Install Dependencies
```bash
anchor --version           # Anchor 0.29 or higher
solana --version           # Solana CLI
```

### 2. Clone and Build
```bash
git clone https://github.com/YOUR_USERNAME/ZetaMint.git
cd ZetaMint
anchor build
```

---

## Testing Locally

### 1. Start local validator
```bash
solana-test-validator
```

### 2. Deploy program
```bash
anchor deploy
```

### 3. Run tests (optional)
```bash
anchor test
```

---

## Project Structure

```
ZetaMint/
├── program/                # Anchor smart contract
├── client/                 # JS client for CLI testing
├── zetachain/              # Message decoding utils for ZetaChain
├── docs/                   # Architecture, security, tutorials
├── demo/                   # Video & screenshot placeholders
└── README.md               # This file
```

---

## Security

See [`docs/SECURITY.md`](./docs/SECURITY.md) for full threat model:
- Replay attack protection
- Signer verification
- Metadata integrity checks (planned)

---

## Cross-Chain Workflow

1. **Mint NFT** on Solana with metadata
2. **Burn NFT** on Solana to initiate transfer
3. Emit message → ZetaChain Gateway receives it
4. On destination chain → Mint equivalent NFT with same metadata

---

## Demo

See [`demo/demo.mp4`](./demo/demo.mp4) for a full walkthrough (placeholder)

---

## License

This project is licensed under the MIT License.

---

## Contributing

Want to help expand multi-chain NFT tooling? Open issues, submit PRs, or join our dev Telegram.

---

## Acknowledgments

- [ZetaChain](https://www.zetachain.com/) for their groundbreaking universal chain messaging
- [Metaplex](https://www.metaplex.com/) for the token metadata standard
- [Solana Foundation](https://solana.org/) for high-performance blockchain infra

