import { ethers } from "ethers";
import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

import { Example } from "../typechain-types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const [signer] = await hre.ethers.getSigners();
  if (!signer) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  const contract: Example = await hre.ethers.getContractAt(
    "Example",
    args.contract
  );

  const tx = await contract.setConnected(
    args.targetChainId,
    ethers.utils.arrayify(args.targetContract)
  );

  await tx.wait();

  console.log("Successfully set the connected contract.");
};

task("connect", "Sets connected contract", main)
  .addParam("contract", "The address of the deployed contract")
  .addParam("targetChainId", "Chain ID of the destination chain")
  .addParam("targetContract", "The address of the connected contract to set")
  .addFlag("json", "Output the result in JSON format");
