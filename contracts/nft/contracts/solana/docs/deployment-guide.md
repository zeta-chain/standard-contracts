# Universal NFT Solana Deployment Guide

This comprehensive guide covers the deployment, configuration, and maintenance of the Universal NFT protocol on Solana, enabling seamless cross-chain NFT transfers via ZetaChain.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Deployment Process](#deployment-process)
4. [Integration Guide](#integration-guide)
5. [Monitoring & Maintenance](#monitoring--maintenance)
6. [Security Considerations](#security-considerations)
7. [Troubleshooting](#troubleshooting)
8. [Appendix](#appendix)

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows with WSL2
- **Node.js**: Version 16.x or higher
- **Rust**: Latest stable version (1.70+)
- **Memory**: Minimum 8GB RAM (16GB recommended for mainnet)
- **Storage**: At least 50GB free space

### Required Tools and Dependencies

#### 1. Solana CLI Installation

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version
# Expected output: solana-cli 1.16.0 (or later)
```

#### 2. Anchor Framework Installation

```bash
# Install Anchor Version Manager (avm)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# Install and use Anchor v0.28.0
avm install 0.28.0
avm use 0.28.0

# Verify installation
anchor --version
# Expected output: anchor-cli 0.28.0
```

#### 3. Node.js Dependencies

```bash
# Install Node.js dependencies
npm install -g yarn
npm install -g @coral-xyz/anchor-cli
npm install -g @solana/web3.js

# Verify installations
node --version  # Should be 16.x or higher
yarn --version
```

#### 4. Required Rust Dependencies

```bash
# Install required Rust components
rustup component add rustfmt
rustup component add clippy

# Install additional tools
cargo install spl-token-cli
cargo install solana-verify
```

### Network Configuration

#### Solana Network Endpoints

```bash
# Configure for different networks
# Devnet (for development and testing)
solana config set --url https://api.devnet.solana.com

# Testnet (for staging)
solana config set --url https://api.testnet.solana.com

# Mainnet (for production)
solana config set --url https://api.mainnet-beta.solana.com

# Verify current configuration
solana config get
```

#### Wallet Setup

```bash
# Create a new wallet (for development)
solana-keygen new --outfile ~/.config/solana/id.json

# Or import existing wallet
solana-keygen recover --outfile ~/.config/solana/id.json

# Set as default wallet
solana config set --keypair ~/.config/solana/id.json

# Check wallet balance
solana balance

# Fund wallet on devnet (for testing)
solana airdrop 2
```

## Environment Setup

### Project Structure

```
universal-nft-solana/
├── programs/
│   └── universal-nft/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── state/
│       │   ├── instructions/
│       │   └── utils/
│       └── Cargo.toml
├── sdk/
│   └── client.ts
├── tests/
├── scripts/
│   ├── deploy-devnet.ts
│   ├── deploy-testnet.ts
│   ├── deploy-mainnet.ts
│   └── validate-deployment.ts
├── monitoring/
│   └── health-check.ts
├── config/
│   └── networks.json
├── Anchor.toml
├── package.json
└── README.md
```

### Environment Variables

Create a `.env` file in the project root:

```bash
# Network Configuration
SOLANA_NETWORK=devnet
RPC_ENDPOINT=https://api.devnet.solana.com

# Program Configuration
PROGRAM_ID=6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke
GATEWAY_PROGRAM_ID=GatewayAddress111111111111111111111111111

# TSS Configuration (ZetaChain)
TSS_ADDRESS_DEVNET=0x1234567890123456789012345678901234567890
TSS_ADDRESS_TESTNET=0x2345678901234567890123456789012345678901
TSS_ADDRESS_MAINNET=0x3456789012345678901234567890123456789012

# Connected Contracts
ETHEREUM_CONTRACT_SEPOLIA=0x4567890123456789012345678901234567890123
BSC_CONTRACT_TESTNET=0x5678901234567890123456789012345678901234
BASE_CONTRACT_SEPOLIA=0x6789012345678901234567890123456789012345

# Monitoring Configuration
MONITORING_ENABLED=true
HEALTH_CHECK_INTERVAL=30000
ALERT_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK/URL

# Security
ENABLE_SECURITY_CHECKS=true
MAX_COMPUTE_UNITS=400000
```

### Build Configuration

Update `Anchor.toml`:

```toml
[features]
seeds = false
skip-lint = false

[programs.devnet]
universal_nft = "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke"

[programs.testnet]
universal_nft = "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke"

[programs.mainnet]
universal_nft = "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"

[test]
startup_wait = 5000
shutdown_wait = 2000
upgradeable = false

[[test.genesis]]
address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
program = "metadata.so"

[test.validator]
url = "https://api.devnet.solana.com"
clone = [
  { address = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s" },
]
```

## Deployment Process

### Pre-Deployment Checklist

Before deploying to any network, ensure:

- [ ] All tests pass locally
- [ ] Code has been audited (for mainnet)
- [ ] TSS addresses are configured correctly
- [ ] Connected contract addresses are verified
- [ ] Sufficient SOL balance for deployment
- [ ] Backup and recovery procedures are in place

### 1. Devnet Deployment

#### Step 1: Build the Program

```bash
# Clean previous builds
anchor clean

# Build the program
anchor build

# Verify the build
ls -la target/deploy/
# Should see universal_nft.so and universal_nft-keypair.json
```

#### Step 2: Deploy to Devnet

```bash
# Set network to devnet
solana config set --url https://api.devnet.solana.com

# Deploy the program
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show 6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke
```

#### Step 3: Initialize Collection

```typescript
// scripts/deploy-devnet.ts
import { UniversalNftClient, Network } from '../sdk/client';
import { Keypair } from '@solana/web3.js';

async function deployDevnet() {
  // Load deployment wallet
  const wallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(process.env.DEPLOYER_PRIVATE_KEY!))
  );

  // Create client
  const client = await UniversalNftClient.create({
    network: Network.DEVNET,
    commitment: 'confirmed'
  }, { keypair: wallet });

  // Initialize collection
  const tssAddress = [
    0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90,
    0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90
  ];

  const { collection, signature } = await client.initializeCollection(
    "Universal NFT Collection",
    "UNFT",
    "https://api.example.com/collection-metadata.json",
    tssAddress
  );

  console.log('Collection initialized:', collection.toString());
  console.log('Transaction signature:', signature);

  // Set up connected contracts
  await setupConnectedContracts(client, collection);
  
  // Validate deployment
  await validateDeployment(client, collection);
}

async function setupConnectedContracts(client: UniversalNftClient, collection: PublicKey) {
  // Ethereum Sepolia
  const ethChainId = [17, 45, 0, 0, 0, 0, 0, 0]; // 11155111 in little-endian
  const ethContract = Array.from(Buffer.from(
    process.env.ETHEREUM_CONTRACT_SEPOLIA!.slice(2), 'hex'
  ));
  
  await client.setConnected(collection, ethChainId, ethContract);
  console.log('Ethereum Sepolia contract connected');

  // BSC Testnet
  const bscChainId = [97, 0, 0, 0, 0, 0, 0, 0]; // 97 in little-endian
  const bscContract = Array.from(Buffer.from(
    process.env.BSC_CONTRACT_TESTNET!.slice(2), 'hex'
  ));
  
  await client.setConnected(collection, bscChainId, bscContract);
  console.log('BSC Testnet contract connected');

  // Base Sepolia
  const baseChainId = [36, 70, 1, 0, 0, 0, 0, 0]; // 84532 in little-endian
  const baseContract = Array.from(Buffer.from(
    process.env.BASE_CONTRACT_SEPOLIA!.slice(2), 'hex'
  ));
  
  await client.setConnected(collection, baseChainId, baseContract);
  console.log('Base Sepolia contract connected');
}
```

#### Step 4: Run Deployment Script

```bash
# Install dependencies
yarn install

# Run devnet deployment
yarn deploy:devnet

# Expected output:
# Collection initialized: 8x7vR2...
# Ethereum Sepolia contract connected
# BSC Testnet contract connected
# Base Sepolia contract connected
# Deployment validation passed
```

### 2. Testnet Deployment

#### Step 1: Update Configuration

```bash
# Switch to testnet
solana config set --url https://api.testnet.solana.com

# Update environment variables
export SOLANA_NETWORK=testnet
export TSS_ADDRESS=$TSS_ADDRESS_TESTNET
```

#### Step 2: Deploy and Configure

```bash
# Deploy to testnet
anchor deploy --provider.cluster testnet

# Run testnet deployment script
yarn deploy:testnet

# Run comprehensive testing
yarn test:testnet
```

### 3. Mainnet Deployment

⚠️ **CRITICAL**: Mainnet deployment requires additional security measures and should only be performed after thorough testing and auditing.

#### Step 1: Security Audit

Before mainnet deployment:

- [ ] Complete security audit by certified auditors
- [ ] Penetration testing completed
- [ ] Bug bounty program concluded
- [ ] All critical and high-severity issues resolved
- [ ] Code freeze implemented

#### Step 2: Mainnet Deployment Process

```bash
# Switch to mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Verify wallet has sufficient SOL (minimum 10 SOL recommended)
solana balance

# Deploy with production configuration
export SOLANA_NETWORK=mainnet
export TSS_ADDRESS=$TSS_ADDRESS_MAINNET

# Deploy the program
anchor deploy --provider.cluster mainnet-beta

# Run production deployment script
yarn deploy:mainnet
```

#### Step 3: Production Validation

```typescript
// scripts/validate-deployment.ts
import { UniversalNftClient, Network } from '../sdk/client';

async function validateProduction() {
  const client = await UniversalNftClient.create({
    network: Network.MAINNET,
    commitment: 'finalized'
  });

  // Test basic functionality
  await testBasicFunctionality(client);
  
  // Test cross-chain integration
  await testCrossChainIntegration(client);
  
  // Test security measures
  await testSecurityMeasures(client);
  
  // Test performance
  await testPerformance(client);
  
  console.log('✅ Production validation completed successfully');
}

async function testBasicFunctionality(client: UniversalNftClient) {
  // Test collection retrieval
  const collections = await client.getCollectionsByAuthority(
    client.provider.wallet.publicKey
  );
  
  if (collections.length === 0) {
    throw new Error('No collections found');
  }
  
  console.log('✅ Basic functionality test passed');
}
```

## Integration Guide

### ZetaChain Gateway Integration

#### 1. Gateway Configuration

The Universal NFT program integrates with ZetaChain's gateway for cross-chain communication:

```typescript
// Gateway integration example
import { PublicKey } from '@solana/web3.js';

const ZETACHAIN_GATEWAY_PROGRAM_ID = new PublicKey(
  "GatewayAddress111111111111111111111111111"
);

// Derive gateway PDA
const [gatewayPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("meta")],
  ZETACHAIN_GATEWAY_PROGRAM_ID
);
```

#### 2. Cross-Chain Message Format

The protocol supports multiple message formats for cross-chain compatibility:

```typescript
// ZetaChain message format
interface ZetaChainMessage {
  token_id: number;
  uri: string;
  destination_address: number[];
  destination_chain_id: number;
  sender: number[];
}

// ABI-encoded format (for EVM chains)
interface ABIMessage {
  token_id: number;      // 32 bytes (big-endian)
  uri_offset: number;    // 32 bytes
  recipient: number[];   // 32 bytes
  sender: number[];      // 20 bytes + 12 bytes padding
  uri_length: number;    // 32 bytes
  uri: string;          // variable length
}

// Borsh-encoded format (for Solana)
interface CrossChainMessage {
  token_id: number;
  uri: string;
  recipient: number[];
  destination_chain: number[];
  sender: number[];
}
```

### EVM Contract Integration

#### 1. Contract Addresses

Configure connected contracts for each supported chain:

```json
{
  "networks": {
    "ethereum": {
      "chainId": 1,
      "testnetChainId": 11155111,
      "contracts": {
        "mainnet": "0x...",
        "sepolia": "0x4567890123456789012345678901234567890123"
      }
    },
    "bsc": {
      "chainId": 56,
      "testnetChainId": 97,
      "contracts": {
        "mainnet": "0x...",
        "testnet": "0x5678901234567890123456789012345678901234"
      }
    },
    "base": {
      "chainId": 8453,
      "testnetChainId": 84532,
      "contracts": {
        "mainnet": "0x...",
        "sepolia": "0x6789012345678901234567890123456789012345"
      }
    }
  }
}
```

#### 2. Message Encoding/Decoding

```typescript
// Encode message for EVM chains
function encodeForEVM(message: CrossChainMessage): Buffer {
  const tokenIdBuffer = Buffer.alloc(32);
  tokenIdBuffer.writeBigUInt64BE(BigInt(message.token_id), 24);
  
  const uriBuffer = Buffer.from(message.uri, 'utf8');
  const uriLengthBuffer = Buffer.alloc(32);
  uriLengthBuffer.writeUInt32BE(uriBuffer.length, 28);
  
  const recipientBuffer = Buffer.alloc(32);
  Buffer.from(message.recipient).copy(recipientBuffer, 12);
  
  const senderBuffer = Buffer.alloc(32);
  Buffer.from(message.sender).copy(senderBuffer, 12);
  
  return Buffer.concat([
    tokenIdBuffer,
    Buffer.alloc(32), // URI offset placeholder
    recipientBuffer,
    senderBuffer,
    uriLengthBuffer,
    uriBuffer
  ]);
}

// Decode message from EVM chains
function decodeFromEVM(data: Buffer): CrossChainMessage {
  let offset = 0;
  
  // Extract token ID
  const tokenId = data.readBigUInt64BE(24);
  offset += 64; // Skip token ID and URI offset
  
  // Extract recipient and sender
  const recipient = Array.from(data.slice(offset + 12, offset + 32));
  offset += 32;
  
  const sender = Array.from(data.slice(offset + 12, offset + 32));
  offset += 32;
  
  // Extract URI
  const uriLength = data.readUInt32BE(offset + 28);
  offset += 32;
  
  const uri = data.slice(offset, offset + uriLength).toString('utf8');
  
  return {
    token_id: Number(tokenId),
    uri,
    recipient,
    destination_chain: [103, 0, 0, 0, 0, 0, 0, 0], // Solana devnet
    sender
  };
}
```

### SDK Usage Examples

#### 1. Basic NFT Operations

```typescript
import { UniversalNftClient, Network } from './sdk/client';
import { Keypair } from '@solana/web3.js';

// Initialize client
const client = await UniversalNftClient.create({
  network: Network.DEVNET,
  commitment: 'confirmed'
}, {
  keypair: Keypair.generate() // or load from file
});

// Mint an NFT
const mintResult = await client.mintNft(
  collectionPubkey,
  "My NFT",
  "MNFT",
  "https://api.example.com/metadata/1.json"
);

console.log('NFT minted:', mintResult.mint.toString());

// Transfer cross-chain to Ethereum
const transferResult = await client.transferCrossChain(
  collectionPubkey,
  mintResult.mint,
  11155111, // Ethereum Sepolia chain ID
  [0x12, 0x34, ...] // Recipient address (20 bytes)
);

console.log('Cross-chain transfer initiated:', transferResult.signature);
```

#### 2. Event Monitoring

```typescript
// Monitor NFT events
const subscription = await client.onNftMinted(
  (event) => {
    console.log('NFT minted:', {
      collection: event.collection.toString(),
      mint: event.mint.toString(),
      recipient: event.recipient.toString(),
      tokenId: event.tokenId.toString()
    });
  },
  collectionPubkey // Optional: filter by collection
);

// Monitor cross-chain transfers
const transferSub = await client.onCrossChainTransfer(
  (event) => {
    console.log('Cross-chain transfer:', {
      mint: event.mint.toString(),
      destinationChain: event.destinationChain,
      recipient: event.recipient
    });
  }
);

// Cleanup subscriptions
setTimeout(() => {
  subscription.unsubscribe();
  transferSub.unsubscribe();
}, 60000);
```

#### 3. Error Handling

```typescript
import { 
  UniversalNftError, 
  TransactionError, 
  AccountNotFoundError 
} from './sdk/client';

try {
  const result = await client.mintNft(
    collection,
    "Test NFT",
    "TEST",
    "https://example.com/metadata.json"
  );
} catch (error) {
  if (error instanceof TransactionError) {
    console.error('Transaction failed:', error.message);
    console.error('Signature:', error.signature);
  } else if (error instanceof AccountNotFoundError) {
    console.error('Account not found:', error.message);
  } else if (error instanceof UniversalNftError) {
    console.error('Universal NFT error:', error.message);
    console.error('Error code:', error.code);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Monitoring & Maintenance

### Health Monitoring Setup

#### 1. Initialize Health Monitor

```typescript
import { 
  UniversalNFTHealthMonitor, 
  createDefaultConfig 
} from './monitoring/health-check';

// Create monitoring configuration
const config = createDefaultConfig(
  process.env.RPC_ENDPOINT!,
  process.env.PROGRAM_ID!
);

// Customize alert thresholds
config.alertConfig = {
  maxErrorRate: 5,        // 5% maximum error rate
  minSuccessRate: 95,     // 95% minimum success rate
  maxResponseTime: 3000,  // 3 second maximum response time
  maxReplayAttempts: 3,   // Maximum replay attack attempts
  maxInvalidSignatures: 5 // Maximum invalid signature attempts
};

// Initialize and start monitoring
const monitor = new UniversalNFTHealthMonitor(config);
await monitor.startMonitoring();

console.log('Health monitoring started');
```

#### 2. Dashboard Integration

```typescript
// Get dashboard data for frontend
const dashboardData = monitor.getDashboardData();

// Example dashboard data structure:
{
  overview: {
    status: 'green',
    isOperational: true,
    totalCollections: 5,
    totalTransfers: 1250,
    activeAlerts: 0
  },
  charts: {
    successRate: 98.5,
    crossChainBreakdown: [
      { chainId: 1, chainName: 'Ethereum', transfers: 450, successRate: 99.1 },
      { chainId: 56, chainName: 'BSC', transfers: 380, successRate: 97.8 }
    ],
    performanceMetrics: {
      latency: 245,
      throughput: 15.2,
      blockHeight: 185432
    }
  }
}
```

#### 3. Alert Configuration

```typescript
// Set up Slack webhook for alerts
process.env.ALERT_WEBHOOK_URL = 'https://hooks.slack.com/services/...';

// Custom alert handler
monitor.onAlert = async (alert) => {
  if (alert.level === 'critical' || alert.level === 'red') {
    await sendSlackAlert(alert);
    await sendEmailAlert(alert);
  }
  
  // Log to monitoring service
  await logToDatadog(alert);
};

async function sendSlackAlert(alert) {
  const webhook = process.env.ALERT_WEBHOOK_URL;
  const payload = {
    text: `🚨 Universal NFT Alert: ${alert.message}`,
    attachments: [{
      color: alert.level === 'critical' ? 'danger' : 'warning',
      fields: [
        { title: 'Level', value: alert.level, short: true },
        { title: 'Category', value: alert.category, short: true },
        { title: 'Time', value: new Date(alert.timestamp).toISOString(), short: true }
      ]
    }]
  };
  
  await fetch(webhook, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });
}
```

### Performance Optimization

#### 1. Compute Unit Optimization

```typescript
// Set compute unit limit for transactions
import { ComputeBudgetProgram } from '@solana/web3.js';

const computeUnitLimit = ComputeBudgetProgram.setComputeUnitLimit({
  units: 400_000 // Adjust based on instruction complexity
});

const computeUnitPrice = ComputeBudgetProgram.setComputeUnitPrice({
  microLamports: 1000 // Adjust based on network congestion
});

// Add to transaction
transaction.add(computeUnitLimit, computeUnitPrice);
```

#### 2. RPC Optimization

```typescript
// Use multiple RPC endpoints for redundancy
const rpcEndpoints = [
  'https://api.devnet.solana.com',
  'https://devnet.helius-rpc.com',
  'https://solana-devnet.g.alchemy.com/v2/your-api-key'
];

class LoadBalancedConnection {
  private connections: Connection[];
  private currentIndex = 0;

  constructor(endpoints: string[]) {
    this.connections = endpoints.map(url => new Connection(url, 'confirmed'));
  }

  getConnection(): Connection {
    const connection = this.connections[this.currentIndex];
    this.currentIndex = (this.currentIndex + 1) % this.connections.length;
    return connection;
  }

  async getAccountInfo(pubkey: PublicKey): Promise<AccountInfo<Buffer> | null> {
    for (const connection of this.connections) {
      try {
        return await connection.getAccountInfo(pubkey);
      } catch (error) {
        console.warn('RPC endpoint failed, trying next:', error);
        continue;
      }
    }
    throw new Error('All RPC endpoints failed');
  }
}
```

#### 3. Caching Strategy

```typescript
// Implement caching for frequently accessed data
import NodeCache from 'node-cache';

const cache = new NodeCache({ 
  stdTTL: 300, // 5 minutes default TTL
  checkperiod: 60 // Check for expired keys every minute
});

class CachedUniversalNftClient extends UniversalNftClient {
  async getCollection(collection: PublicKey): Promise<CollectionData> {
    const cacheKey = `collection:${collection.toString()}`;
    
    let data = cache.get<CollectionData>(cacheKey);
    if (data) {
      return data;
    }
    
    data = await super.getCollection(collection);
    cache.set(cacheKey, data, 60); // Cache for 1 minute
    
    return data;
  }
  
  async getNftOrigin(nftOrigin: PublicKey): Promise<NftOriginData> {
    const cacheKey = `nft_origin:${nftOrigin.toString()}`;
    
    let data = cache.get<NftOriginData>(cacheKey);
    if (data) {
      return data;
    }
    
    data = await super.getNftOrigin(nftOrigin);
    cache.set(cacheKey, data, 300); // Cache for 5 minutes
    
    return data;
  }
}
```

### Maintenance Procedures

#### 1. Regular Health Checks

```bash
#!/bin/bash
# scripts/health-check.sh

echo "Running Universal NFT health check..."

# Check program account
solana program show $PROGRAM_ID

# Check recent transactions
solana transaction-history $PROGRAM_ID --limit 10

# Run automated tests
yarn test:health

# Generate health report
node scripts/generate-health-report.js

echo "Health check completed"
```

#### 2. Backup Procedures

```typescript
// scripts/backup-accounts.ts
import { Connection, PublicKey } from '@solana/web3.js';
import fs from 'fs';

async function backupAccounts() {
  const connection = new Connection(process.env.RPC_ENDPOINT!);
  const programId = new PublicKey(process.env.PROGRAM_ID!);
  
  // Get all program accounts
  const accounts = await connection.getProgramAccounts(programId);
  
  const backup = {
    timestamp: Date.now(),
    programId: programId.toString(),
    accounts: accounts.map(account => ({
      pubkey: account.pubkey.toString(),
      data: account.account.data.toString('base64'),
      owner: account.account.owner.toString(),
      lamports: account.account.lamports,
      executable: account.account.executable
    }))
  };
  
  const filename = `backup-${new Date().toISOString().split('T')[0]}.json`;
  fs.writeFileSync(`backups/${filename}`, JSON.stringify(backup, null, 2));
  
  console.log(`Backup saved to ${filename}`);
  console.log(`Total accounts backed up: ${accounts.length}`);
}

// Run backup
backupAccounts().catch(console.error);
```

#### 3. Update Procedures

```bash
#!/bin/bash
# scripts/update-program.sh

echo "Starting program update procedure..."

# 1. Backup current state
./scripts/backup-accounts.sh

# 2. Run tests on new version
anchor test

# 3. Deploy to testnet first
anchor deploy --provider.cluster testnet

# 4. Validate testnet deployment
yarn test:testnet

# 5. Deploy to mainnet (if testnet validation passes)
read -p "Deploy to mainnet? (y/N): " confirm
if [[ $confirm == [yY] ]]; then
  anchor deploy --provider.cluster mainnet-beta
  yarn validate:mainnet
fi

echo "Update procedure completed"
```

## Security Considerations

### TSS Signature Validation

#### 1. Signature Verification Process

The program implements robust TSS signature verification:

```rust
// From lib.rs - TSS signature verification
pub fn verify_tss_signature(
    message_hash: &[u8; 32],
    signature: &[u8; 64],
    recovery_id: u8,
    expected_tss_address: &[u8; 20],
) -> Result<()> {
    // Validate recovery ID range
    require!(recovery_id <= 3, UniversalNftError::InvalidTssSignature);
    
    // Recover public key from signature
    let recovered_pubkey = solana_program::secp256k1_recover::secp256k1_recover(
        message_hash,
        recovery_id,
        signature
    ).map_err(|_| UniversalNftError::InvalidTssSignature)?;
    
    // Convert to Ethereum address
    let recovered_address = pubkey_to_eth_address(&recovered_pubkey.0)?;
    
    // Verify against expected TSS address
    require!(
        recovered_address == *expected_tss_address,
        UniversalNftError::UnauthorizedTssAddress
    );
    
    Ok(())
}
```

#### 2. TSS Address Management

```typescript
// Secure TSS address configuration
const TSS_ADDRESSES = {
  devnet: '0x1234567890123456789012345678901234567890',
  testnet: '0x2345678901234567890123456789012345678901',
  mainnet: '0x3456789012345678901234567890123456789012'
};

// Validate TSS address format
function validateTssAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
}

// Convert to bytes array for program
function tssAddressToBytes(address: string): number[] {
  if (!validateTssAddress(address)) {
    throw new Error('Invalid TSS address format');
  }
  return Array.from(Buffer.from(address.slice(2), 'hex'));
}
```

### Replay Attack Prevention

#### 1. Nonce-Based Protection

```rust
// Enhanced nonce validation from lib.rs
pub fn validate_nonce(
    collection: &mut Collection,
    provided_nonce: u64,
    message_data: &[u8],
) -> Result<()> {
    // Strict nonce ordering
    require!(
        provided_nonce > collection.nonce,
        UniversalNftError::InvalidNonce
    );
    
    // Prevent nonce gaps (max 1000 ahead)
    require!(
        provided_nonce <= collection.nonce.saturating_add(1000),
        UniversalNftError::InvalidNonce
    );
    
    // Bind nonce to message content
    let mut nonce_bound_message = Vec::new();
    nonce_bound_message.extend_from_slice(message_data);
    nonce_bound_message.extend_from_slice(&provided_nonce.to_le_bytes());
    
    // Update nonce after validation
    collection.nonce = provided_nonce;
    
    Ok(())
}
```

#### 2. Message Hash Integrity

```typescript
// Client-side message hash generation
import { keccak256 } from 'js-sha3';

function generateMessageHash(
  tokenId: number,
  uri: string,
  recipient: string,
  nonce: number
): Buffer {
  const message = Buffer.concat([
    Buffer.from(tokenId.toString()),
    Buffer.from(uri, 'utf8'),
    Buffer.from(recipient.slice(2), 'hex'),
    Buffer.from(nonce.toString())
  ]);
  
  return Buffer.from(keccak256(message), 'hex');
}

// Verify message hash on receive
function verifyMessageHash(
  receivedHash: Buffer,
  tokenId: number,
  uri: string,
  recipient: string,
  nonce: number
): boolean {
  const computedHash = generateMessageHash(tokenId, uri, recipient, nonce);
  return receivedHash.equals(computedHash);
}
```

### Access Control and Authorization

#### 1. Authority Validation

```rust
// Authority checks in instructions
#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(
        mut,
        has_one = authority @ UniversalNftError::UnauthorizedAuthority
    )]
    pub collection: Account<'info, Collection>,
    
    pub authority: Signer<'info>,
    // ... other accounts
}

// Additional runtime checks
pub fn validate_authority(
    collection: &Collection,
    signer: &Pubkey,
) -> Result<()> {
    require!(
        collection.authority == *signer,
        UniversalNftError::UnauthorizedAuthority
    );
    Ok(())
}
```

#### 2. Token Ownership Verification

```typescript
// Verify token ownership before transfer
async function verifyTokenOwnership(
  connection: Connection,
  mint: PublicKey,
  owner: PublicKey
): Promise<boolean> {
  try {
    const tokenAccount = await getAssociatedTokenAddress(mint, owner);
    const accountInfo = await getAccount(connection, tokenAccount);
    
    return accountInfo.amount > 0n && accountInfo.owner.equals(owner);
  } catch (error) {
    return false;
  }
}

// Use in transfer operations
if (!await verifyTokenOwnership(connection, nftMint, sender)) {
  throw new Error('Sender does not own the NFT');
}
```

### Security Audit Checklist

#### Pre-Deployment Security Checklist

- [ ] **Code Review**
  - [ ] All functions have proper access controls
  - [ ] Input validation on all parameters
  - [ ] No integer overflow/underflow vulnerabilities
  - [ ] Proper error handling and recovery

- [ ] **Cryptographic Security**
  - [ ] TSS signature verification implemented correctly
  - [ ] Message hash generation is deterministic
  - [ ] Nonce-based replay protection active
  - [ ] Secure random number generation where needed

- [ ] **Account Security**
  - [ ] PDA derivations are deterministic and secure
  - [ ] Account ownership checks in all instructions
  - [ ] Rent exemption properly handled
  - [ ] No account confusion vulnerabilities

- [ ] **Cross-Chain Security**
  - [ ] Message format validation
  - [ ] Chain ID verification
  - [ ] Recipient address validation
  - [ ] Gateway integration security

- [ ] **Economic Security**
  - [ ] Gas fee calculations are correct
  - [ ] No economic exploits possible
  - [ ] Proper handling of failed transactions
  - [ ] Refund mechanisms work correctly

#### Runtime Security Monitoring

```typescript
// Security monitoring integration
class SecurityMonitor {
  private suspiciousPatterns = new Map<string, number>();
  
  async monitorTransaction(signature: string) {
    const tx = await this.connection.getTransaction(signature);
    
    if (tx?.meta?.logMessages) {
      this.analyzeTransactionLogs(tx.meta.logMessages, signature);
    }
  }
  
  private analyzeTransactionLogs(logs: string[], signature: string) {
    // Check for security-related errors
    const securityErrors = [
      'InvalidNonce',
      'InvalidTssSignature',
      'UnauthorizedTssAddress',
      'NotTokenOwner',
      'InvalidRecipient'
    ];
    
    for (const error of securityErrors) {
      if (logs.some(log => log.includes(error))) {
        this.recordSuspiciousActivity(error, signature);
      }
    }
  }
  
  private recordSuspiciousActivity(type: string, signature: string) {
    const count = this.suspiciousPatterns.get(type) || 0;
    this.suspiciousPatterns.set(type, count + 1);
    
    // Alert if threshold exceeded
    if (count > 10) {
      this.triggerSecurityAlert(type, count, signature);
    }
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Deployment Issues

**Issue**: Program deployment fails with "Insufficient funds"
```bash
Error: Insufficient funds for fee (0.00123 SOL), balance: 0.00089 SOL
```

**Solution**:
```bash
# Check current balance
solana balance

# Fund wallet (devnet/testnet)
solana airdrop 2

# For mainnet, transfer SOL from another wallet
solana transfer <recipient> <amount> --from <source-keypair>
```

**Issue**: Program deployment fails with "Program already exists"
```bash
Error: Program already exists at address 6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke
```

**Solution**:
```bash
# Use upgrade instead of deploy
anchor upgrade target/deploy/universal_nft.so --program-id 6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke

# Or generate new program ID
solana-keygen new --outfile target/deploy/universal_nft-keypair.json
```

#### 2. Transaction Failures

**Issue**: Transaction fails with "Compute budget exceeded"
```
Error: Transaction simulation failed: Error processing Instruction 0: Program failed to complete
```

**Solution**:
```typescript
// Increase compute unit limit
import { ComputeBudgetProgram } from '@solana/web3.js';

const computeUnitLimit = ComputeBudgetProgram.setComputeUnitLimit({
  units: 400_000 // Increase from default 200k
});

transaction.add(computeUnitLimit);
```

**Issue**: Cross-chain transfer fails with "Invalid TSS signature"
```
Error: Invalid TSS signature
```

**Solution**:
```typescript
// Verify TSS address configuration
const collection = await client.getCollection(collectionPubkey);
console.log('TSS Address:', collection.tssAddress);

// Ensure message hash is generated correctly
const messageHash = generateMessageHash(tokenId, uri, recipient, nonce);

// Verify signature format (64 bytes)
if (signature.length !== 64) {
  throw new Error('Invalid signature length');
}
```

#### 3. Account Issues

**Issue**: Account not found errors
```
Error: Account not found: 8x7vR2...
```

**Solution**:
```typescript
// Check if account exists before accessing
const accountExists = await client.accountExists(accountPubkey);
if (!accountExists) {
  console.log('Account does not exist, creating...');
  // Initialize account or handle gracefully
}

// Use proper error handling
try {
  const account = await client.getCollection(collectionPubkey);
} catch (error) {
  if (error instanceof AccountNotFoundError) {
    // Handle missing account
    console.log('Collection not found, please initialize first');
  }
}
```

#### 4. Network Issues

**Issue**: RPC rate limiting
```
Error: 429 Too Many Requests
```

**Solution**:
```typescript
// Implement retry logic with exponential backoff
async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3
): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      
      const delay = Math.pow(2, i) * 1000; // Exponential backoff
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  throw new Error('Max retries exceeded');
}

// Use multiple RPC endpoints
const rpcEndpoints = [
  'https://api.devnet.solana.com',
  'https://devnet.helius-rpc.com'
];
```

### Debug Tools and Techniques

#### 1. Transaction Analysis

```bash
# Analyze failed transaction
solana transaction <signature>

# Get detailed logs
solana logs --follow

# Monitor program logs
solana logs 6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke
```

#### 2. Account Inspection

```bash
# Inspect account data
solana account <account-pubkey>

# Get program accounts
solana program show 6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke

# Check token account
spl-token account-info <token-account>
```

#### 3. Program Testing

```typescript
// Unit test for specific functionality
describe('Universal NFT', () => {
  it('should mint NFT correctly', async () => {
    const result = await client.mintNft(
      collection,
      "Test NFT",
      "TEST",
      "https://example.com/metadata.json"
    );
    
    expect(result.mint).to.be.instanceOf(PublicKey);
    expect(result.signature).to.be.a('string');
  });
  
  it('should handle cross-chain transfer', async () => {
    // Test cross-chain functionality
    const transferResult = await client.transferCrossChain(
      collection,
      nftMint,
      11155111, // Ethereum Sepolia
      ethRecipientBytes
    );
    
    expect(transferResult.signature).to.be.a('string');
  });
});
```

### Performance Debugging

#### 1. Compute Unit Analysis

```typescript
// Measure compute units used
async function measureComputeUnits(
  connection: Connection,
  signature: string
): Promise<number> {
  const tx = await connection.getTransaction(signature, {
    maxSupportedTransactionVersion: 0
  });
  
  return tx?.meta?.computeUnitsConsumed || 0;
}

// Optimize instruction ordering
const instructions = [
  computeUnitLimit,
  computeUnitPrice,
  ...programInstructions
];
```

#### 2. RPC Performance

```typescript
// Measure RPC latency
async function measureRPCLatency(connection: Connection): Promise<number> {
  const start = Date.now();
  await connection.getSlot();
  return Date.now() - start;
}

// Connection health check
async function checkConnectionHealth(connection: Connection): Promise<boolean> {
  try {
    const latency = await measureRPCLatency(connection);
    return latency < 5000; // 5 second threshold
  } catch (error) {
    return false;
  }
}
```

## Appendix

### A. Network Configuration Reference

```json
{
  "networks": {
    "devnet": {
      "endpoint": "https://api.devnet.solana.com",
      "programId": "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke",
      "gatewayProgramId": "GatewayAddress111111111111111111111111111",
      "tssAddress": "0x1234567890123456789012345678901234567890",
      "supportedChains": {
        "ethereum": {
          "chainId": 11155111,
          "contractAddress": "0x4567890123456789012345678901234567890123",
          "rpcUrl": "https://sepolia.infura.io/v3/YOUR_API_KEY"
        },
        "bsc": {
          "chainId": 97,
          "contractAddress": "0x5678901234567890123456789012345678901234",
          "rpcUrl": "https://data-seed-prebsc-1-s1.binance.org:8545"
        },
        "base": {
          "chainId": 84532,
          "contractAddress": "0x6789012345678901234567890123456789012345",
          "rpcUrl": "https://sepolia.base.org"
        }
      }
    },
    "testnet": {
      "endpoint": "https://api.testnet.solana.com",
      "programId": "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke",
      "gatewayProgramId": "GatewayAddress111111111111111111111111111",
      "tssAddress": "0x2345678901234567890123456789012345678901",
      "supportedChains": {
        "ethereum": {
          "chainId": 11155111,
          "contractAddress": "0x...",
          "rpcUrl": "https://sepolia.infura.io/v3/YOUR_API_KEY"
        }
      }
    },
    "mainnet": {
      "endpoint": "https://api.mainnet-beta.solana.com",
      "programId": "6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke",
      "gatewayProgramId": "GatewayAddress111111111111111111111111111",
      "tssAddress": "0x3456789012345678901234567890123456789012",
      "supportedChains": {
        "ethereum": {
          "chainId": 1,
          "contractAddress": "0x...",
          "rpcUrl": "https://mainnet.infura.io/v3/YOUR_API_KEY"
        },
        "bsc": {
          "chainId": 56,
          "contractAddress": "0x...",
          "rpcUrl": "https://bsc-dataseed1.binance.org"
        },
        "base": {
          "chainId": 8453,
          "contractAddress": "0x...",
          "rpcUrl": "https://mainnet.base.org"
        }
      }
    }
  }
}
```

### B. Error Code Reference

| Error Code | Description | Solution |
|------------|-------------|----------|
| `InvalidTssSignature` | TSS signature verification failed | Check signature format and TSS address |
| `InvalidMessageHash` | Message hash doesn't match expected | Verify message encoding and hash generation |
| `InvalidNonce` | Nonce validation failed | Ensure nonce is greater than previous |
| `NotTokenOwner` | Sender doesn't own the NFT | Verify token ownership before transfer |
| `UnsupportedChain` | Destination chain not supported | Add chain to connected contracts |
| `InvalidRecipient` | Recipient address format invalid | Check address format for target chain |
| `InsufficientGasAmount` | Gas amount too low for transfer | Increase gas amount for transaction |
| `UnauthorizedGateway` | Gateway not authorized | Verify gateway program ID |

### C. Gas Fee Reference

| Destination Chain | Base Gas | Typical Range | Notes |
|------------------|----------|---------------|-------|
| Ethereum Mainnet | 150,000 | 0.1-1.0 SOL | High during congestion |
| Ethereum Sepolia | 150,000 | 0.01-0.1 SOL | Testnet rates |
| BSC Mainnet | 80,000 | 0.05-0.2 SOL | Generally lower fees |
| BSC Testnet | 80,000 | 0.01-0.05 SOL | Testnet rates |
| Base Mainnet | 100,000 | 0.05-0.3 SOL | L2 efficiency |
| Base Sepolia | 100,000 | 0.01-0.05 SOL | Testnet rates |
| ZetaChain | 50,000 | 0.02-0.1 SOL | Native integration |

### D. Useful Commands

```bash
# Development Commands
anchor build                    # Build program
anchor test                     # Run tests
anchor deploy                   # Deploy program
anchor upgrade <program.so>     # Upgrade existing program

# Solana CLI Commands
solana config get              # Show current configuration
solana balance                 # Check wallet balance
solana airdrop 2               # Request devnet SOL
solana program show <id>       # Show program info
solana transaction <sig>       # Show transaction details
solana logs <program-id>       # Monitor program logs

# SPL Token Commands
spl-token create-token         # Create new token
spl-token create-account       # Create token account
spl-token mint                 # Mint tokens
spl-token transfer             # Transfer tokens

# Monitoring Commands
yarn monitor:start             # Start health monitoring
yarn monitor:dashboard         # View monitoring dashboard
yarn monitor:alerts            # Check active alerts
yarn monitor:report            # Generate health report
```

### E. Support and Resources

- **Documentation**: [Solana Docs](https://docs.solana.com/)
- **Anchor Framework**: [Anchor Book](https://book.anchor-lang.com/)
- **ZetaChain**: [ZetaChain Docs](https://docs.zetachain.com/)
- **Community**: [Solana Discord](https://discord.gg/solana)
- **Issues**: [GitHub Issues](https://github.com/your-repo/issues)

---

*This deployment guide is maintained by the Universal NFT development team. For updates and additional resources, visit our [documentation repository](https://github.com/your-repo/docs).*