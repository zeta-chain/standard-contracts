import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { EVMUniversalToken } from "../typechain-types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { isAddress } = hre.ethers.utils;

  const [signer] = await hre.ethers.getSigners();
  if (!signer) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  if (!isAddress(args.contract) || !isAddress(args.universal)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const contract: EVMUniversalToken = await hre.ethers.getContractAt(
    "EVMUniversalToken",
    args.contract
  );

  const tx = await contract.setUniversal(args.universal);

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        universalContract: args.universal,
        transactionHash: tx.hash,
      })
    );
  } else {
    console.log(`🚀 Successfully set the universal contract.
📜 Contract address: ${args.contract}
🔗 Transaction hash: ${tx.hash}`);
  }
};

export const tokenSetUniversal = task(
  "token:set-universal",
  "Sets the universal contract address",
  main
)
  .addParam("contract", "The address of the deployed contract")
  .addParam("universal", "The address of the universal contract to set")
  .addFlag("json", "Output the result in JSON format");
