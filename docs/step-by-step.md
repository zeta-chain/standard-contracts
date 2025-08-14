# Step-by-Step Deployment Guide for ZetaMint

## 1. Generate a New Program Keypair
Run the following command to create a new program keypair and save it:
```bash
solana-keygen new --outfile target/deploy/zetamint-keypair.json --no-passphrase
```
You will get an output like:
```
Wrote new keypair to target/deploy/zetamint-keypair.json
pubkey: EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU
```
This **pubkey** is your **Program ID**.

---

## 2. Update `Anchor.toml`
In the root `Anchor.toml`, update:
```toml
[programs.localnet]
zetamint = "EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU"
```

If you are using **devnet** or **mainnet**, also update their respective sections.

---

## 3. Update `lib.rs` in Rust Program
In your `programs/zetamint/src/lib.rs` file:
```rust
declare_id!("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");
```

---

## 4. Update `index.js` in Client
In your `app/src/index.js` or similar client entry file:
```javascript
const PROGRAM_ID = new anchor.web3.PublicKey("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");
```

---

## 5. Build and Deploy
Run:
```bash
anchor build
anchor deploy
```

---

## 6. Test the Program
To run tests:
```bash
anchor test
```

---

✅ **Tip**: Every time you make structural changes to the program, you will need to repeat these steps starting from generating a new keypair.
