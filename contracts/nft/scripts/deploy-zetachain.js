const hre = require("hardhat");
const fs = require("fs");

async function main() {
  const [deployer] = await hre.ethers.getSigners();

  console.log("🚀 Deploying UniversalNFT to ZetaChain testnet with account:", deployer.address);
  console.log("💰 Account balance:", (await deployer.getBalance()).toString());

  // Deploy UniversalNFT contract
  const UniversalNFT = await hre.ethers.getContractFactory("UniversalNFT");
  const contract = await UniversalNFT.deploy(
    "UniversalNFT",           // name
    "UNFT",                   // symbol
    "0x6c533f7fe93fae114d0954697069df33c9b74fd7", // ZetaChain Gateway
    1000000                   // gas limit
  );

  await contract.deployed();

  console.log("✅ UniversalNFT deployed to:", contract.address);

  // Get deployment transaction details
  const deployTx = contract.deployTransaction;
  const receipt = await deployTx.wait();

  // Load existing results and add ZetaChain
  let results = {};
  try {
    results = JSON.parse(fs.readFileSync("results.json", "utf8"));
  } catch (e) {
    // File doesn't exist yet, start fresh
  }

  results.zetachain = {
    contract: contract.address,
    deployer: deployer.address,
    txHash: deployTx.hash,
    gasUsed: receipt.gasUsed.toString(),
    blockNumber: receipt.blockNumber,
    timestamp: new Date().toISOString()
  };

  fs.writeFileSync("results.json", JSON.stringify(results, null, 2));
  console.log("📝 Results updated in results.json");

  // Update markdown for PR comment
  let markdown = "";
  if (results.baseSepolia) {
    markdown += `## 🚀 Cross-Chain Deployment Complete

### Base Sepolia
**Contract Address:** \`${results.baseSepolia.contract}\`
**Transaction Hash:** [${results.baseSepolia.txHash}](https://sepolia.basescan.org/tx/${results.baseSepolia.txHash})

### ZetaChain Testnet
**Contract Address:** \`${contract.address}\`
**Transaction Hash:** [${deployTx.hash}](https://explorer.zetachain.com/tx/${deployTx.hash})

✅ Ready for cross-chain testing!
`;
  } else {
    markdown = `## 🚀 ZetaChain Testnet Deployment Complete

**Contract Address:** \`${contract.address}\`
**Deployer:** \`${deployer.address}\`
**Transaction Hash:** [${deployTx.hash}](https://explorer.zetachain.com/tx/${deployTx.hash})
**Gas Used:** ${receipt.gasUsed.toString()}
**Block:** ${receipt.blockNumber}

✅ Ready for cross-chain testing!
`;
  }

  fs.writeFileSync("results.md", markdown);
  console.log("📝 Markdown results updated in results.md");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  });
