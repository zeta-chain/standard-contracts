const { ethers } = require("hardhat");

async function main() {
  const [signer] = await ethers.getSigners();
  const address = signer.address;
  const balance = await signer.getBalance();
  
  console.log("Wallet Address:", address);
  console.log("Balance:", ethers.utils.formatEther(balance), "ETH");
  console.log("Network:", hre.network.name);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
