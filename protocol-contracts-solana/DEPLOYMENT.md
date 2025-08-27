# Solana Universal NFT Program - Deployment Guide

## Overview
This guide provides step-by-step instructions for deploying and testing the Solana Universal NFT program on Solana networks.

## Prerequisites

### 1. Solana CLI Tools
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.14/install)"

# Verify installation
solana --version
```

### 2. Anchor Framework
```bash
# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Verify installation
anchor --version
```

### 3. Node.js and TypeScript
```bash
# Install Node.js (v18+)
# Download from https://nodejs.org/

# Install TypeScript
npm install -g typescript ts-node
```

## Build Configuration

### 1. Anchor.toml Configuration
```toml
[features]
seeds = false
skip-lint = false

[programs.localnet]
universal_nft = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[programs.devnet]
universal_nft = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[programs.mainnet]
universal_nft = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

### 2. Program ID
The program ID is: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`

## Deployment Steps

### 1. Local Development
```bash
# Build the program
anchor build

# Start local validator
solana-test-validator

# Deploy to localnet
anchor deploy --provider.cluster localnet
```

### 2. Devnet Deployment
```bash
# Switch to devnet
solana config set --url devnet

# Airdrop SOL for deployment
solana airdrop 2

# Build and deploy
anchor build
anchor deploy --provider.cluster devnet
```

### 3. Mainnet Deployment
```bash
# Switch to mainnet
solana config set --url mainnet-beta

# Build and deploy (ensure sufficient SOL balance)
anchor build
anchor deploy --provider.cluster mainnet-beta
```

## Testing Strategy

### 1. Unit Tests
```bash
# Run unit tests
anchor test

# Run specific test
anchor test --skip-local-validator test_mint_nft
```

### 2. Integration Tests
```bash
# Run integration tests with local validator
anchor test --skip-local-validator
```

### 3. Manual Testing
```bash
# Test mint instruction
anchor run test-mint

# Test cross-chain transfer
anchor run test-cross-chain
```

## Test Scenarios

### 1. Basic NFT Minting
- Mint new NFT with metadata
- Verify NFT creation
- Check metadata accuracy

### 2. Cross-Chain Transfer
- Simulate incoming cross-chain message
- Verify NFT restoration
- Check replay protection

### 3. Security Tests
- Test unauthorized access
- Verify PDA validation
- Test replay attack prevention

### 4. Gateway Integration
- Test gateway authentication
- Verify payload validation
- Test error handling

## Monitoring and Logging

### 1. Program Logs
```bash
# Monitor program logs
solana logs <PROGRAM_ID>

# Filter specific instructions
solana logs <PROGRAM_ID> | grep "mint_new_nft"
```

### 2. Transaction Monitoring
```bash
# Get transaction details
solana confirm <SIGNATURE>

# View account data
solana account <ACCOUNT_ADDRESS>
```

## Integration with ZetaChain Gateway

### 1. Gateway Configuration
```typescript
const gatewayConfig = {
  programId: "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
  gatewayProgram: "ZETA_GATEWAY_PROGRAM_ID",
  // Add other configuration
};
```

### 2. Cross-Chain Message Format
```typescript
interface CrossChainNftPayload {
  token_id: [u8; 32];
  origin_chain_id: u16;
  origin_mint: PublicKey;
  recipient: PublicKey;
  metadata_uri: string;
  nonce: u64;
}
```

### 3. Integration Testing
```bash
# Test gateway integration
anchor run test-gateway-integration
```

## Security Considerations

### 1. Access Control
- Gateway program validation
- PDA-based account creation
- Replay attack prevention

### 2. Data Validation
- Payload deserialization
- Metadata URI validation
- Account state verification

### 3. Error Handling
- Comprehensive error codes
- Graceful failure handling
- User-friendly error messages

## Performance Optimization

### 1. Gas Optimization
- Efficient PDA derivation
- Minimal account creation
- Optimized CPI calls

### 2. Storage Optimization
- Compact data structures
- Efficient serialization
- Minimal account space usage

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   - Check Anchor version compatibility
   - Verify dependency versions
   - Clean and rebuild: `anchor clean && anchor build`

2. **Deployment Failures**
   - Ensure sufficient SOL balance
   - Check network connectivity
   - Verify program ID configuration

3. **Runtime Errors**
   - Check account validation
   - Verify PDA derivation
   - Review transaction logs

### Debug Commands
```bash
# Check program account
solana account <PROGRAM_ID>

# Verify deployment
anchor verify <PROGRAM_ID>

# Check build artifacts
ls target/deploy/
```

## Next Steps

1. **Complete Testing**: Run comprehensive test suite
2. **Security Audit**: Conduct security review
3. **Performance Testing**: Benchmark program performance
4. **Documentation**: Update user documentation
5. **Integration**: Complete ZetaChain Gateway integration

## Support

For issues and questions:
- GitHub Issues: [Repository Issues](https://github.com/zeta-chain/standard-contracts/issues)
- Documentation: [Project Documentation](https://docs.zeta.tech)
- Community: [Discord Community](https://discord.gg/zetachain)
