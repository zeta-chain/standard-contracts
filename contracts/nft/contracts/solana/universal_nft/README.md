# Solana Universal NFT  
Cross-Chain NFT Program for ZetaChain Interoperability
---

## Overview  
Universal NFT lets any NFT move freely between Solana, ZetaChain, and all connected EVM / non-EVM chains.

* Burn-and-mint model – NFT is burned on the source chain, re-minted on the destination.  
* Metadata, collection and ownership are preserved across chains.  
* Each deployment forms its own **collection**; NFTs minted by that program are members of that collection.

---

## Architecture  

| Component | Purpose |
|-----------|---------|
| **Anchor program `universal_nft`** | Core on-chain logic (Solana). |
| **PDAs** | `config`, `treasury`, `mint_auth`, `nft_origin`, `processed_message`. |
| **Collection mint** | Created in `initialize`, supplies 1 token held by `treasury`. |
| **Payload (Borsh)** | ```struct Payload { token_id\[32\]; name; symbol; uri; recipient; origin_chain_id; nonce; original_solana_mint? }``` – passed through Gateway `withdrawAndCall`. |
| **Gateway program** | `ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis` – verified in every inbound call. |

### Instruction Flow  

1. **initialize** → deploy program, create collection mint, PDAs.  
2. **mint_local** → mint NFT on Solana, create `nft_origin` PDA.  
3. **burn_and_prepare** → burn, emit event; client encodes `Payload`, calls Gateway `deposit_and_call`.  
4. **Gateway → on_call** → program validates caller, decodes payload, mints NFT to `recipient`.  
5. **on_revert** (optional) → handles revert paths from ZetaChain.

---

## Gateway Integration  

* Program implements `on_call` & `on_revert` per Gateway spec.  
* Verification:
  * Previous instruction’s `program_id` == Gateway program id.
  * `gateway_meta` PDA (seed `b"meta"`) matches expected address.
  * Replay protected with `processed_message` PDA (seeded by `sender|token_id|nonce`).

Expected accounts for `on_call`:
```
config, mint_auth, nft_origin (init/exists), processed_message (init/exists),
mint (init), token_account (init ATA),
gateway_meta, payer, system, token, associated_token, rent, sysvar::instructions
```

---

## Setup  

### Prerequisites  
* Rust `stable` + `cargo-build-sbf` (v1.79 or newer)  
* Solana CLI ≥ 1.18  
* Anchor CLI ≥ 0.30  
* Node ≥ 18 (for scripts)  
* A funded Solana **testnet** keypair (`~/.config/solana/id.json` by default)

### Build & Deploy (testnet default)  
```bash
git clone https://github.com/zeta-chain/standard-contracts.git
cd standard-contracts/contracts/nft/contracts/solana/universal_nft
anchor build          # produces target/deploy/universal_nft.so
anchor deploy         # deploys to Solana testnet as per Anchor.toml
```
To switch cluster: `anchor deploy --provider.cluster devnet`.

### Funding Wallet  
```
solana airdrop 2 --url https://api.testnet.solana.com
```

---

## Usage  

### 1. Initialize collection  
```bash
anchor run initialize \
  -- --collection-name "Universal Apes" \
     --collection-symbol "UAPE" \
     --collection-uri "https://example.com/metadata.json"
```

### 2. Mint local NFT  
```bash
anchor run mint_local \
  -- --name "Ape #1" --symbol "UAPE" --uri "https://example.com/1.json" \
     --recipient <RECIPIENT_PUBKEY>
```

### 3. Burn & send cross-chain  
```bash
anchor run burn_and_prepare -- --mint <NFT_MINT>
# Script prints emitted token_id, then:
ts-node scripts/sendToZeta.ts <token_id> <name> <symbol> <uri> <recipientEvmAddr>
```
`sendToZeta.ts`:
```ts
import { Payload } from "./payload";
const payload = Payload.encode({...});
gateway.depositAndCall(ZRC20_NULL, 0, payload, ...);
```

### 4. Receive on Solana (automatic)  
ZetaChain observers execute `withdrawAndCall`; Gateway calls `on_call`; NFT appears in recipient ATA.

---

## Security Notes  

* **Replay protection** – processed messages recorded in PDA.  
* **Caller auth** – verifies Gateway program & meta PDA via `sysvar::instructions`.  
* **Signer model** – mint authority & treasury are PDAs signed with seeds.  
* **Rent** – treasury PDA prefunded by admin (lamports flow from `payer`).  
* **Compute budget** – heavy minting CPIs require ~200k CU; add before minting:  
  `solana compute-budget increase-cu 200000`.

---

## Configuration  

Create `.env` (not committed):

```
SOLANA_CLUSTER=https://api.testnet.solana.com
ZETA_RPC_TESTNET=https://zetachain-testnet.node
ETH_RPC_BASE_SEPOLIA=<url>
WALLET_PRIVATE_KEY=<base58>
```

Scripts read env for cross-chain tests.

---

## Devnet vs Testnet  

* **Default** Anchor.toml targets `testnet` (closer to ZetaChain testnet).  
* For purely local demos or free airdrops, deploy to `devnet`:
  `anchor deploy --provider.cluster devnet`.  
* Gateway id is identical on devnet/testnet/mainnet.

---

## Example TS Payload Encoding  

```ts
import { struct, u32, u64, blob, str } from "@solana/buffer-layout";
import { publicKey } from "@solana/buffer-layout-utils";

export const Payload = struct([
  blob(32, "token_id"),
  str("name"),
  str("symbol"),
  str("uri"),
  publicKey("recipient"),
  u32("origin_chain_id"),
  u64("nonce"),
  publicKey("original_solana_mint", true) // option
]);

const data = Payload.encode({
  token_id,
  name: "Ape #1",
  symbol: "UAPE",
  uri: "https://example.com/1.json",
  recipient: new PublicKey("<SOL_PUBKEY>").toBytes(),
  origin_chain_id: 7000,   // example ZEVM chain id
  nonce: Date.now(),
  original_solana_mint: null
});
```

Send `data` as the `message` argument to Gateway `depositAndCall`.

---

## License  
MIT  
© 2025 ZetaChain contributors
