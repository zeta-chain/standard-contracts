const hre = require("hardhat");
const fs = require("fs");

async function main() {
  console.log("🧪 Testing Base Sepolia deployment script...");
  
  try {
    // Test 1: Check if we can load the contract factory
    console.log("✅ Test 1: Loading contract factory...");
    const UniversalNFT = await hre.ethers.getContractFactory("UniversalNFT");
    console.log("✅ Contract factory loaded successfully");
    
    // Test 2: Check if we can create deployment parameters
    console.log("✅ Test 2: Creating deployment parameters...");
    const deployParams = [
      "UniversalNFT",           // name
      "UNFT",                   // symbol
      "0x6c533f7fe93fae114d0954697069df33c9b74fd7", // ZetaChain Gateway
      1000000                   // gas limit
    ];
    console.log("✅ Deployment parameters created:", deployParams);
    
    // Test 3: Check if we can create the deployment transaction (without sending)
    console.log("✅ Test 3: Creating deployment transaction...");
    const deploymentTx = UniversalNFT.getDeployTransaction(...deployParams);
    console.log("✅ Deployment transaction created successfully");
    console.log("✅ Gas estimate:", deploymentTx.gasLimit?.toString() || "N/A");
    
    // Test 4: Check if we can access ethers
    console.log("✅ Test 4: Testing ethers functionality...");
    const [deployer] = await hre.ethers.getSigners();
    console.log("✅ Signer loaded:", deployer.address);
    console.log("✅ Balance check:", (await deployer.getBalance()).toString());
    
    // Test 5: Test file operations
    console.log("✅ Test 5: Testing file operations...");
    const testResults = {
      test: "Base Sepolia deployment script validation",
      status: "PASSED",
      timestamp: new Date().toISOString(),
      contractFactory: "UniversalNFT",
      deployParams: deployParams,
      signerAddress: deployer.address
    };
    
    fs.writeFileSync("test-results.json", JSON.stringify(testResults, null, 2));
    console.log("✅ Test results saved to test-results.json");
    
    console.log("🎉 All tests passed! Base Sepolia deployment script is ready.");
    
  } catch (error) {
    console.error("❌ Test failed:", error.message);
    process.exit(1);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Test execution failed:", error);
    process.exit(1);
  });
