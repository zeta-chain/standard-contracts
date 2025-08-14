# ZetaMint – Solana ↔ ZetaChain Universal NFT Program

> **Mint once, roam everywhere – NFTs without borders.**

ZetaMint is a **cross-chain NFT program** built on **Solana** that enables seamless minting, burning, and transferring of NFTs across multiple blockchains using **ZetaChain's universal messaging gateway**.

---

## 🚀 Features
- **Mint NFTs** on Solana with complete metadata (URI, title, symbol)
- **Burn NFTs** to trigger cross-chain migration
- Prepare **ZetaChain gateway-compatible messages**
- Scalable, modular, and developer-friendly architecture
- Security-first design with **replay protection** and **signer validation**

---

## 🛠 Installation

### 1. Install Prerequisites
```bash
anchor --version           # Requires Anchor 0.29 or higher
solana --version           # Solana CLI
```

### 2. Clone and Build
```bash
git clone https://github.com/YOUR_USERNAME/ZetaMint.git
cd ZetaMint
anchor build
```

---

## 🧪 Local Testing

### 1. Start Local Validator
```bash
solana-test-validator
```

### 2. Deploy the Program
```bash
anchor deploy
```

### 3. Run Tests (Optional)
```bash
anchor test
```

---

## 📂 Project Structure
```
ZetaMint/
├── program/      # Anchor smart contract
├── client/       # JavaScript client for CLI testing
├── zetachain/    # Message decoding utilities for ZetaChain
├── docs/         # Architecture diagrams, security notes, tutorials
├── demo/         # Video & screenshot placeholders
└── README.md     # Project documentation
```

---

## 🔒 Security
See [`SECURITY.md`](./SECURITY.md) for full threat model, including:
- Replay attack prevention
- Signer verification
- Planned metadata integrity checks

---

## 🌉 Cross-Chain Workflow
1. **Mint NFT** on Solana with metadata
2. **Burn NFT** on Solana to initiate transfer
3. Emit cross-chain message → ZetaChain Gateway receives it
4. On the destination chain → Mint an equivalent NFT with the same metadata

---

## 🎥 Demo
Demo video coming soon: `demo/demo.mp4`

---

## 📜 License
Licensed under the **MIT License**.

---

## 🤝 Contributing
We welcome contributions!  
Open an issue, submit a PR, or join our developer community.

---

## 🙏 Acknowledgments
- [ZetaChain](https://www.zetachain.com/) – Universal chain messaging
- [Metaplex](https://www.metaplex.com/) – Token metadata standard
- [Solana Foundation](https://solana.org/) – High-performance blockchain infrastructure
