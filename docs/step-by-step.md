# Step-by-Step Guide

This guide walks you through the process to deploy and interact with the ZetaMint program.

## 1. Prerequisites
- Install Rust and Solana CLI.
- Install Anchor framework.
- Ensure you have a valid Solana wallet keypair.

## 2. Generate Program ID
Run:
```bash
solana-keygen new --outfile target/deploy/zetamint-keypair.json --no-passphrase
```
Then get the public key:
```bash
solana-keygen pubkey target/deploy/zetamint-keypair.json
```
Replace `NFTBridge11111111111111111111111111111111111` in all relevant files with this generated public key.

## 3. Build the Program
```bash
anchor build
```

## 4. Deploy the Program
```bash
anchor deploy
```

## 5. Run Tests
```bash
anchor test
```

## Notes
- Ensure `target/deploy/zetamint-keypair.json` is in `.gitignore`.
- Keep your private keys safe and never commit them to source control.
