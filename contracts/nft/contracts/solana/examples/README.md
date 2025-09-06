# Universal NFT Client SDK Examples

This directory contains comprehensive examples and test scenarios for the Universal NFT program with NFT Origin system.

## Files Overview

- `client-integration.ts` - Complete client SDK with all program interactions
- `test-scenarios.ts` - Comprehensive test suite covering all edge cases
- `README.md` - This documentation file

## Quick Start

### 1. Setup Environment

```bash
# Install dependencies
npm install

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration
```

### 2. Basic Usage

```typescript
import { UniversalNftClient } from './examples/client-integration';

// Initialize client
const provider = anchor.AnchorProvider.env();
const program = anchor.workspace.UniversalNft;
const client = new UniversalNftClient(program, provider);

// Create collection
const { collection } = await client.initializeCollection(
  "My Collection",
  "MC",
  "https://example.com/collection.json",
  tssAddress
);

// Mint NFT with Origin tracking
const { mint, nftOrigin } = await client.mintNft(
  collection,
  "My NFT #1",
  "MC",
  "https://example.com/nft1.json"
);
```

## Core Features

### NFT Origin System
- **Deterministic Token IDs**: Generated from mint + authority hash
- **PDA Seeds**: `[b"nft_origin", token_id]`
- **Cross-Chain Tracking**: Preserves original mint and chain information
- **Two Scenarios**: New NFTs vs. returning NFTs

### Supported Operations

#### Collection Management
```typescript
// Initialize collection
await client.initializeCollection(name, symbol, uri, tssAddress);

// Set universal contract
await client.setUniversal(collection, universalAddress);

// Set connected contracts
await client.setConnected(collection, chainId, contractAddress);
```

#### NFT Operations
```typescript
// Mint with Origin tracking
await client.mintNft(collection, name, symbol, uri, recipient);

// Cross-chain transfer
await client.transferCrossChain(collection, mint, destinationChain, recipient);

// Handle incoming transfers
await client.onCall(collection, sender, sourceChain, message, nonce);
```

#### Data Retrieval
```typescript
// Get NFT Origin data
const origin = await client.getNftOrigin(nftOriginPda);

// Get collection data
const collection = await client.getCollection(collectionPda);

// Get connected contracts
const connected = await client.getConnected(connectedPda);
```

## PDA Derivation

### Collection PDA
```typescript
const [collection] = UniversalNftClient.deriveCollectionPda(
  authority,
  collectionName
);
```

### NFT Origin PDA
```typescript
const [nftOrigin] = UniversalNftClient.deriveNftOriginPda(tokenId);
```

### Connected Contract PDA
```typescript
const [connected] = UniversalNftClient.deriveConnectedPda(
  collection,
  chainId
);
```

## Testing

### Run All Tests
```bash
# With local validator
export PATH="/opt/homebrew/opt/gnu-tar/libexec/gnubin:$PATH"
anchor test

# Skip local validator (use existing)
anchor test --skip-local-validator
```

### Test Categories

1. **NFT Origin System Tests**
   - Collection initialization
   - NFT minting with Origin tracking
   - Cross-chain transfers
   - Connected contract management

2. **Cross-Chain Message Handling**
   - New NFT arrivals (Scenario A)
   - Returning NFTs (Scenario B)
   - Message parsing and validation

3. **Edge Cases & Security**
   - Duplicate prevention
   - Invalid message handling
   - Replay attack prevention
   - Nonce validation

4. **PDA Derivation Validation**
   - Consistency checks
   - Uniqueness validation
   - Deterministic generation

## Cross-Chain Message Format

### Outgoing Messages (Solana → Other Chains)
```
[token_id: 8 bytes] [uri_length: 4 bytes] [uri: variable] [recipient: 20/32 bytes]
```

### Incoming Messages (Other Chains → Solana)
```
[token_id: 8 bytes] [uri_length: 4 bytes] [uri: variable] [recipient: 32 bytes]
```

## Error Handling

Common errors and solutions:

### `InvalidMessage`
- Check message format and length
- Verify token ID encoding (little-endian)
- Ensure recipient address is valid

### `InvalidNonce`
- Use nonce > collection.nonce
- Prevent replay attacks
- Check nonce ordering

### `UnauthorizedGateway`
- Verify gateway program ID
- Check gateway account derivation
- Ensure proper CPI context

## Production Deployment

### Environment Variables
```bash
UNIVERSAL_NFT_PROGRAM_ID=6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json
```

### Devnet Deployment
```bash
# Set cluster
solana config set --url https://api.devnet.solana.com

# Deploy program
solana program deploy target/deploy/universal_nft.so \
  --program-id target/deploy/universal_nft-keypair.json
```

### Mainnet Considerations
- Security audit required
- Compute budget optimization
- Rate limiting for cross-chain operations
- Monitoring and alerting setup

## Integration Examples

### Frontend Integration
```typescript
// React/Next.js example
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { UniversalNftClient } from './universal-nft-client';

function MyComponent() {
  const { connection } = useConnection();
  const wallet = useWallet();
  
  const client = new UniversalNftClient(
    program,
    new AnchorProvider(connection, wallet, {})
  );
  
  const handleMint = async () => {
    const result = await client.mintNft(
      collection,
      "My NFT",
      "SYMBOL",
      "https://metadata.json"
    );
    console.log("Minted:", result.mint.toString());
  };
}
```

### Backend Integration
```typescript
// Node.js service example
import { Connection, Keypair } from '@solana/web3.js';
import { AnchorProvider, Wallet } from '@coral-xyz/anchor';

const connection = new Connection(process.env.RPC_URL);
const wallet = new Wallet(Keypair.fromSecretKey(secretKey));
const provider = new AnchorProvider(connection, wallet, {});
const client = new UniversalNftClient(program, provider);

// Handle cross-chain events
async function handleCrossChainTransfer(event) {
  await client.onCall(
    event.collection,
    event.sender,
    event.sourceChain,
    event.message,
    event.nonce
  );
}
```

## Support

For issues and questions:
1. Check test scenarios for examples
2. Verify PDA derivation is correct
3. Ensure proper account setup
4. Check compute budget limits
5. Validate message formats

## License

This code is part of the Universal NFT implementation and follows the same license terms.
