# feat(solana): Universal NFT program with on-chain on_call, Metaplex metadata, devnet/testnet tooling

Implements the Solana Universal NFT program per [Issue #72](https://github.com/zeta-chain/standard-contracts/issues/72).

## Scope
- On-chain entrypoints: `mint_new_nft`, `burn_for_transfer`, `handle_incoming`, `setup_gateway`, `on_call`, `send_to_zeta` (stub).
- Full on-chain handling in `on_call` (no client branching): gateway config PDA verification, replay protection, mint to recipient, `nft_origin` persistence.
- Burn-and-mint flow with replay PDAs.
- PDA derivations centralized in `utils.rs` with seed order `[token_id, "nft_origin"]`.
- Metaplex metadata + master edition created on mint and incoming restore.
- Devnet script to deploy EVM side and Solana program; CI workflow for devnet deploy; toolchain updates.

## Files of interest
- Program: `protocol-contracts-solana/programs/universal_nft/src/lib.rs`
- Instructions: `.../ix/on_call.rs`, `.../ix/handle_incoming.rs`, `.../ix/mint.rs`, `.../ix/burn.rs`, `.../ix/send_to_zeta.rs`, `.../ix/setup_gateway.rs`
- State/PDAs: `.../state/nft_origin.rs`, `.../state/replay.rs`, `.../state/gateway.rs`
- Utils/Seeds/Metaplex CPIs: `.../utils.rs`
- Devnet script: `contracts/nft/scripts/devnet.sh`
- CI: `protocol-contracts-solana/.github/workflows/deploy-devnet.yml`
- Docs: `protocol-contracts-solana/README.md`

## Network requirements (per owner guidance)
- Works on Solana devnet and ZetaChain testnet.
- Flows to validate:
  - Solana devnet → Base Sepolia
  - ZetaChain testnet → Solana devnet

## Security/validation
- Gateway config PDA enforced in `on_call` and `handle_incoming`.
- Replay protection PDAs per `token_id + nonce` with marker initialization.
- Seed ordering/spec centralized and documented in `utils.rs`.

## TODO / Follow-ups
- Enforce invoker authenticity via Gateway CPI signer seeds once the interface is finalized.
- Implement real outbound CPI in `send_to_zeta` (currently stub).
- Provide recorded devnet/testnet transaction links for flows above.
- Ensure each deployment forms a separate Metaplex collection (metadata already created; collection setting can be extended).

## How to test (devnet)
Requirements: Node 18+, Anchor 0.29, Solana 1.18.x.

```
cd contracts/nft/scripts
chmod +x devnet.sh
./devnet.sh
```

Scenarios:
1) Mint on Solana devnet → send to Base Sepolia
2) Mint on ZetaChain testnet → send to Solana devnet
3) Base Sepolia → Solana devnet
4) ZetaChain → Base Sepolia → Solana → ZetaChain


