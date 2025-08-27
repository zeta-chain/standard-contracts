# 🚀 Cross-Chain NFT CI Pipeline

This repository includes a complete GitHub Actions pipeline that automatically deploys Universal NFT contracts to multiple chains and runs cross-chain transfer tests.

## 📋 What This Pipeline Does

1. **Deploys contracts** to Base Sepolia, ZetaChain testnet, and Solana devnet
2. **Runs cross-chain tests** to verify NFT transfers between chains
3. **Captures all details** including contract addresses and transaction hashes
4. **Auto-comments results** on your PR for reviewers to verify

## 🔧 Required GitHub Secrets

Add these secrets to your repository settings:

| Secret Name | Description | Example |
|-------------|-------------|---------|
| `EVM_PRIVATE_KEY` | Private key for EVM deployments | `0x1234...` |
| `SOLANA_KEYPAIR` | Solana keypair JSON content | `{"type":"...","data":[...]}` |
| `BASE_SEPOLIA_RPC` | Base Sepolia RPC URL | `https://sepolia.base.org` |
| `ZETACHAIN_RPC` | ZetaChain testnet RPC | `https://rpc.ankr.com/zetachain_evm_testnet` |
| `SOLANA_DEVNET_RPC` | Solana devnet RPC | `https://api.devnet.solana.com` |

## 🚀 How to Use

### 1. Set Up Secrets
- Go to your repo → Settings → Secrets and variables → Actions
- Add all required secrets listed above

### 2. Push to Trigger Pipeline
- Create or update a PR
- The pipeline will automatically run on:
  - PR opened
  - PR updated
  - PR reopened
  - PR marked as "ready for review"

### 3. View Results
- Check the Actions tab for pipeline status
- Results will be auto-commented on your PR
- Download artifacts for detailed logs

## 📁 Pipeline Files

- `.github/workflows/crosschain-ci.yml` - Main workflow
- `contracts/nft/scripts/deploy-base-sepolia.js` - Base Sepolia deployment
- `contracts/nft/scripts/deploy-zetachain.js` - ZetaChain deployment
- `contracts/nft/scripts/deploy-solana.js` - Solana deployment capture
- `contracts/nft/scripts/crosschain-test.js` - Cross-chain transfer tests

## 🔍 What Reviewers Will See

The pipeline automatically posts a comment like this:

```markdown
## ✅ Cross-Chain NFT Test Results

### Contract Addresses
| Chain | Contract Address |
|-------|------------------|
| Base Sepolia | `0x1234...` |
| ZetaChain Testnet | `0x5678...` |
| Solana Devnet | `9ABC...` |

### Cross-Chain Transfer Proofs
- Solana → Base Sepolia: [Burn TX](link) / [Mint TX](link)
- Base Sepolia → Solana: [Burn TX](link) / [Mint TX](link)
- ZetaChain → Solana: [Burn TX](link) / [Mint TX](link)
```

## 🛠️ Customization

### Add New Chains
1. Create new deployment script in `scripts/`
2. Add deployment step in workflow
3. Update cross-chain test script

### Modify Test Flows
Edit `crosschain-test.js` to:
- Use real bridge SDKs (Wormhole, LayerZero)
- Add more transfer directions
- Include gas cost analysis

## 🚨 Troubleshooting

### Pipeline Fails
- Check secret names match exactly
- Verify RPC URLs are accessible
- Ensure sufficient funds in deployer wallets

### Missing Results
- Check if all deployment scripts ran successfully
- Verify file paths in workflow steps
- Check GitHub Actions logs for errors

## 📚 Next Steps

1. **Set up secrets** in your repository
2. **Push a PR** to trigger the pipeline
3. **Review results** in the auto-generated comment
4. **Customize** for your specific needs

## 🤝 Contributing

Feel free to improve this pipeline by:
- Adding more chains
- Implementing real bridge integrations
- Enhancing error handling
- Adding performance metrics

---

**Note:** This pipeline currently uses mock transaction hashes for cross-chain transfers. Replace with actual bridge SDK calls for production use.
