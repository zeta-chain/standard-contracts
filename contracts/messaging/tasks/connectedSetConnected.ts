import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { Example } from "../typechain-types";
import { ethers } from "ethers";

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

  await contract.setConnected(args.zrc20, ethers.utils.hexlify(args.connected));
  console.log("Successfully set the connected contract.");
};

task("connected-set-connected", "Sets connected contract", main)
  .addParam("contract", "The address of the deployed contract")
  .addParam("connected", "The address of the connected contract to set")
  .addParam("zrc20", "ZRC-20 address of the gas token of destination chain")
  .addFlag("json", "Output the result in JSON format");
