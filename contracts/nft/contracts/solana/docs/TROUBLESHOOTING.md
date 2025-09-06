# Universal NFT Program - Troubleshooting Guide

## Common Issues and Solutions

### Deployment Issues

#### 1. "68 write transactions failed" Error
**Problem**: Program deployment fails with transaction errors.
**Causes**: Network congestion, rate limiting, insufficient compute budget.
**Solutions**:
```bash
# Check if program is already deployed
solana program show <PROGRAM_ID>

# Increase compute budget
solana config set --commitment confirmed

# Try deploying with buffer account
solana program write-buffer target/deploy/universal_nft.so
solana program set-buffer-authority <BUFFER_ADDRESS> --new-buffer-authority <UPGRADE_AUTHORITY>
solana program deploy --buffer <BUFFER_ADDRESS> --program-id <PROGRAM_ID>
```

#### 2. Node.js/Hardhat Compatibility Issues
**Problem**: "Node.js version not supported" or "ESM module" errors.
**Solutions**:
```bash
# Update Node.js to LTS version 22+
nvm install 22
nvm use 22

# Add ESM support to package.json
npm pkg set type="module"

# Update Hardhat configuration for ESM
# Use .mjs extension or update imports
```

#### 3. Anchor Version Mismatch
**Problem**: "anchor-cli version not correct" warnings.
**Solutions**:
```bash
# Install correct Anchor version locally
npm install @coral-xyz/anchor@0.30.1

# Or use global version
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
```

### Transaction Failures

#### 1. "Invalid Token ID" Error (6000)
**Problem**: Token ID collision or invalid generation.
**Debugging**:
```typescript
// Check collection state
const collection = await program.account.collection.fetch(collectionPda);
console.log("Next Token ID:", collection.nextTokenId.toString());

// Verify Origin PDA derivation
const [expectedOrigin] = PublicKey.findProgramAddressSync(
  [Buffer.from("nft_origin"), collection.nextTokenId.toArrayLike(Buffer, 'le', 8)],
  program.programId
);
```

#### 2. "Invalid Authority" Error (6001)
**Problem**: Wrong signer or authority mismatch.
**Solutions**:
```typescript
// Ensure correct authority is signing
const collectionData = await program.account.collection.fetch(collection);
console.log("Expected authority:", collectionData.authority.toString());
console.log("Provided authority:", authority.publicKey.toString());

// Check if authority matches
if (!collectionData.authority.equals(authority.publicKey)) {
  throw new Error("Authority mismatch");
}
```

#### 3. "Invalid Message Format" Error (6003)
**Problem**: Malformed cross-chain message.
**Debugging**:
```typescript
function validateMessage(message: number[]): void {
  if (message.length < 44) {
    throw new Error("Message too short");
  }
  
  // Check token ID (8 bytes)
  const tokenId = Buffer.from(message.slice(0, 8)).readBigUInt64LE();
  console.log("Token ID:", tokenId.toString());
  
  // Check URI length (4 bytes)
  const uriLength = Buffer.from(message.slice(8, 12)).readUInt32LE();
  console.log("URI Length:", uriLength);
  
  if (message.length < 12 + uriLength + 32) {
    throw new Error("Message length mismatch");
  }
}
```

#### 4. "Invalid Nonce" Error (6004)
**Problem**: Replay attack prevention or nonce ordering.
**Solutions**:
```typescript
// Check current nonce
const collection = await program.account.collection.fetch(collectionPda);
const currentNonce = collection.nonce;

// Use nonce + 1 for next operation
const nextNonce = currentNonce.add(new anchor.BN(1));
```

### PDA Derivation Issues

#### 1. "Seeds constraint was violated" Error
**Problem**: Incorrect PDA seeds or bump calculation.
**Solutions**:
```typescript
// Collection PDA
const [collection, collectionBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("collection"),
    authority.publicKey.toBuffer(),
    Buffer.from(collectionName, 'utf8')
  ],
  program.programId
);

// NFT Origin PDA (use little-endian token ID)
const [nftOrigin, originBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("nft_origin"),
    tokenId.toArrayLike(Buffer, 'le', 8)
  ],
  program.programId
);

// Connected PDA
const [connected, connectedBump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("connected"),
    collection.toBuffer(),
    Buffer.from(chainId)
  ],
  program.programId
);
```

#### 2. "Account does not exist" Error
**Problem**: PDA not initialized or wrong derivation.
**Debugging**:
```typescript
// Check if account exists
const accountInfo = await connection.getAccountInfo(pdaAddress);
if (!accountInfo) {
  console.log("Account does not exist, needs initialization");
}

// Verify PDA derivation
const [derivedPda, bump] = PublicKey.findProgramAddressSync(seeds, programId);
console.log("Derived PDA:", derivedPda.toString());
console.log("Expected PDA:", expectedPda.toString());
console.log("Match:", derivedPda.equals(expectedPda));
```

### Metaplex Integration Issues

#### 1. "Invalid metadata account" Error
**Problem**: Incorrect Metaplex metadata PDA derivation.
**Solutions**:
```typescript
const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

// Correct metadata PDA
const [metadata] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    mint.publicKey.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

// Master edition PDA
const [masterEdition] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    mint.publicKey.toBuffer(),
    Buffer.from("edition"),
  ],
  TOKEN_METADATA_PROGRAM_ID
);
```

### Cross-Chain Integration Issues

#### 1. Gateway Connection Failures
**Problem**: Cannot connect to ZetaChain gateway.
**Debugging**:
```typescript
// Check gateway account
const gatewayInfo = await connection.getAccountInfo(gatewayPda);
if (!gatewayInfo) {
  console.log("Gateway account not found");
}

// Verify gateway program ID
const EXPECTED_GATEWAY_ID = new PublicKey("GatewayProgram1111111111111111111111111111");
console.log("Gateway program ID:", EXPECTED_GATEWAY_ID.toString());
```

#### 2. Message Format Incompatibility
**Problem**: EVM chains reject Solana messages.
**Solutions**:
```typescript
// Ensure proper address format conversion
function convertSolanaToEvm(solanaAddress: PublicKey): Buffer {
  // Take last 20 bytes for EVM compatibility
  const solanaBytes = solanaAddress.toBuffer();
  return solanaBytes.slice(12, 32);
}

function convertEvmToSolana(evmAddress: Buffer): PublicKey {
  // Pad EVM address (20 bytes) to Solana format (32 bytes)
  const padded = Buffer.alloc(32);
  evmAddress.copy(padded, 12);
  return new PublicKey(padded);
}
```

### Performance Issues

#### 1. Transaction Timeouts
**Problem**: Transactions take too long or timeout.
**Solutions**:
```typescript
// Increase confirmation timeout
const connection = new Connection(rpcUrl, {
  commitment: 'confirmed',
  confirmTransactionInitialTimeout: 60000,
});

// Use priority fees for faster processing
const priorityFee = 1000; // microlamports
const instruction = ComputeBudgetProgram.setComputeUnitPrice({
  microLamports: priorityFee,
});
```

#### 2. Compute Budget Exceeded
**Problem**: "Program failed to complete" due to compute limits.
**Solutions**:
```typescript
// Add compute budget instruction
const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
  units: 400_000, // Increase as needed
});

// Add to transaction
const transaction = new Transaction()
  .add(computeBudgetIx)
  .add(yourInstruction);
```

### Testing Issues

#### 1. Local Validator Startup Failures
**Problem**: solana-test-validator fails to start.
**Solutions**:
```bash
# Kill existing validators
pkill -f solana-test-validator

# Clear ledger data
rm -rf .anchor/test-ledger

# Use GNU tar on macOS
export PATH="/opt/homebrew/opt/gnu-tar/libexec/gnubin:$PATH"

# Start with specific configuration
solana-test-validator \
  --reset \
  --quiet \
  --ledger .anchor/test-ledger \
  --bind-address 127.0.0.1 \
  --rpc-port 8899 \
  --faucet-port 9900
```

#### 2. Test Account Funding Issues
**Problem**: Insufficient SOL for test operations.
**Solutions**:
```typescript
// Request larger airdrop amounts
const airdropAmount = 10 * LAMPORTS_PER_SOL;
const signature = await connection.requestAirdrop(
  keypair.publicKey,
  airdropAmount
);
await connection.confirmTransaction(signature);

// Check balance before operations
const balance = await connection.getBalance(keypair.publicKey);
console.log("Balance:", balance / LAMPORTS_PER_SOL, "SOL");
```

## Debugging Tools

### 1. Program Logs
```bash
# Follow program logs
solana logs <PROGRAM_ID>

# Get specific transaction logs
solana confirm <TRANSACTION_SIGNATURE> -v
```

### 2. Account Inspection
```typescript
// Inspect account data
const accountInfo = await connection.getAccountInfo(address);
console.log("Account owner:", accountInfo?.owner.toString());
console.log("Account data length:", accountInfo?.data.length);

// Decode program account
const accountData = await program.account.collection.fetch(collectionPda);
console.log("Collection data:", accountData);
```

### 3. Transaction Simulation
```typescript
// Simulate transaction before sending
const simulation = await connection.simulateTransaction(transaction);
console.log("Simulation result:", simulation);
console.log("Logs:", simulation.value.logs);
```

## Best Practices

### 1. Error Handling
```typescript
try {
  await program.methods.mintNft(name, symbol, uri)
    .accounts({ /* accounts */ })
    .rpc();
} catch (error) {
  if (error.code === 6000) {
    console.error("Invalid Token ID:", error.message);
  } else if (error.code === 6001) {
    console.error("Invalid Authority:", error.message);
  } else {
    console.error("Unknown error:", error);
  }
}
```

### 2. Account Validation
```typescript
// Always validate accounts before operations
async function validateCollection(collection: PublicKey): Promise<void> {
  const accountInfo = await connection.getAccountInfo(collection);
  if (!accountInfo) {
    throw new Error("Collection does not exist");
  }
  
  if (!accountInfo.owner.equals(program.programId)) {
    throw new Error("Invalid collection owner");
  }
}
```

### 3. Retry Logic
```typescript
async function retryTransaction(
  operation: () => Promise<string>,
  maxRetries: number = 3
): Promise<string> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await operation();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }
  throw new Error("Max retries exceeded");
}
```

## Environment Setup

### Development Environment
```bash
# Required versions
node --version    # v22.0.0+
anchor --version  # 0.30.1
solana --version  # 1.18.0+

# Environment variables
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="~/.config/solana/id.json"
export UNIVERSAL_NFT_PROGRAM_ID="6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke"
```

### Testing Environment
```bash
# Local testing
export ANCHOR_PROVIDER_URL="http://127.0.0.1:8899"
export ANCHOR_WALLET="~/.config/solana/id.json"

# Use test keypairs
solana-keygen new --outfile test-keypair.json --no-bip39-passphrase
```

## Support Resources

- **Anchor Documentation**: https://www.anchor-lang.com/
- **Solana Documentation**: https://docs.solana.com/
- **Metaplex Documentation**: https://docs.metaplex.com/
- **Program Explorer**: https://explorer.solana.com/address/6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke?cluster=devnet

## Emergency Procedures

### 1. Program Upgrade
```bash
# Deploy new version
solana program deploy target/deploy/universal_nft.so \
  --program-id target/deploy/universal_nft-keypair.json \
  --upgrade-authority ~/.config/solana/id.json
```

### 2. Account Recovery
```bash
# Close buffer accounts to recover rent
solana program close <BUFFER_ADDRESS>

# Recover failed deployment funds
solana program show --buffers
```

### 3. Emergency Stop
If critical issues are discovered:
1. Update program authority to multisig
2. Implement emergency pause functionality
3. Coordinate with ZetaChain team for gateway updates
