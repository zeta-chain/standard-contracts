const hre = require("hardhat");
const fs = require("fs");

async function main() {
  const [deployer] = await hre.ethers.getSigners();

  console.log("🚀 Deploying UniversalNFT to Base Sepolia with account:", deployer.address);
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

  // Save results
  const results = {
    baseSepolia: {
      contract: contract.address,
      deployer: deployer.address,
      txHash: deployTx.hash,
      gasUsed: receipt.gasUsed.toString(),
      blockNumber: receipt.blockNumber,
      timestamp: new Date().toISOString()
    }
  };

  fs.writeFileSync("results.json", JSON.stringify(results, null, 2));
  console.log("📝 Results saved to results.json");

  // Also create markdown for PR comment
  const markdown = `## 🚀 Base Sepolia Deployment Complete

**Contract Address:** \`${contract.address}\`
**Deployer:** \`${deployer.address}\`
**Transaction Hash:** [${deployTx.hash}](https://sepolia.basescan.org/tx/${deployTx.hash})
**Gas Used:** ${receipt.gasUsed.toString()}
**Block:** ${receipt.blockNumber}

✅ Ready for cross-chain testing!
`;

  fs.writeFileSync("results.md", markdown);
  console.log("📝 Markdown results saved to results.md");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  });
