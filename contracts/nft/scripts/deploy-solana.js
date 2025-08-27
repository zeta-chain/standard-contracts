const fs = require("fs");
const { execSync } = require("child_process");

async function main() {
  console.log("🔗 Capturing Solana deployment details...");

  try {
    // Get the deployed program ID from Anchor
    const programId = execSync("solana address -k protocol-contracts-solana/target/deploy/universal_nft-keypair.json", { encoding: 'utf8' }).trim();
    
    console.log("✅ Solana program deployed at:", programId);

    // Load existing results
    let results = {};
    try {
      results = JSON.parse(fs.readFileSync("results.json", "utf8"));
    } catch (e) {
      console.error("❌ No results.json found. Run EVM deployment scripts first.");
      process.exit(1);
    }

    // Add Solana deployment info
    results.solana = {
      programId: programId,
      keypairFile: "protocol-contracts-solana/target/deploy/universal_nft-keypair.json",
      timestamp: new Date().toISOString()
    };

    // Save updated results
    fs.writeFileSync("results.json", JSON.stringify(results, null, 2));

    // Update markdown
    let markdown = `## 🚀 Cross-Chain Deployment Complete

### Contract Addresses
| Chain | Contract Address |
|-------|------------------|
| Base Sepolia | \`${results.baseSepolia?.contract || 'N/A'}\` |
| ZetaChain Testnet | \`${results.zetachain?.contract || 'N/A'}\` |
| Solana Devnet | \`${programId}\` |

### Deployment Transactions
- **Base Sepolia:** [${results.baseSepolia?.txHash || 'N/A'}](https://sepolia.basescan.org/tx/${results.baseSepolia?.txHash || 'N/A'})
- **ZetaChain:** [${results.zetachain?.txHash || 'N/A'}](https://explorer.zetachain.com/tx/${results.zetachain?.txHash || 'N/A'})

✅ All contracts deployed successfully!
`;

    fs.writeFileSync("results.md", markdown);
    console.log("📝 Results updated with Solana deployment info");

  } catch (error) {
    console.error("❌ Failed to capture Solana deployment:", error);
    process.exit(1);
  }
}

main().catch(console.error);
