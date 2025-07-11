import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { EVMUniversalToken } from "../typechain-types";
import { ethers } from "ethers";

const DEFAULT_GAS_LIMIT = "1000000";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { isAddress } = hre.ethers.utils;

  let signer;

  if (args.rpc) {
    const provider = new ethers.providers.JsonRpcProvider(args.rpc);
    signer = new ethers.Wallet(process.env.PRIVATE_KEY || "", provider);
  } else {
    [signer] = await hre.ethers.getSigners();
  }

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
    args.contract,
    signer
  );

  const tx = await contract.setUniversal(args.universal, {
    gasLimit: args.gasLimit,
  });
  const receipt = await tx.wait();

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        universalContract: args.universal,
        transactionHash: tx.hash,
        gasUsed: receipt.gasUsed.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully set the universal contract.
📜 Contract address: ${args.contract}
🔗 Universal contract address: ${args.universal}
🔗 Transaction hash: ${tx.hash}
⛽ Gas used: ${receipt.gasUsed.toString()}`);
  }
};

export const tokenSetUniversal = task(
  "token:set-universal",
  "Sets the universal contract address",
  main
)
  .addParam("contract", "The address of the deployed contract")
  .addParam("universal", "The address of the universal contract to set")
  .addOptionalParam(
    "gasLimit",
    "Gas limit for the transaction",
    DEFAULT_GAS_LIMIT
  )
  .addOptionalParam("rpc", "Custom RPC URL to use for the transaction")
  .addFlag("json", "Output the result in JSON format");
