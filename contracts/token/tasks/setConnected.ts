import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { ZetaChainUniversalToken } from "../typechain-types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { isAddress } = hre.ethers.utils;

  const [signer] = await hre.ethers.getSigners();
  if (!signer) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  if (!isAddress(args.contract) || !isAddress(args.zrc20)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const contract: ZetaChainUniversalToken = await hre.ethers.getContractAt(
    "ZetaChainUniversalToken",
    args.contract
  );

  const tx = await contract.setConnected(args.zrc20, args.connected);

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        zrc20: args.zrc20,
        connectedContractAddress: args.connected,
        transactionHash: tx.hash,
      })
    );
  } else {
    console.log(`🚀 Successfully set the connected contract.
📜 Contract address: ${args.contract}
🔗 ZRC20 address: ${args.zrc20}
🔗 Connected contract address: ${args.connected}
🔗 Transaction hash: ${tx.hash}`);
  }
};

export const tokenSetConnected = task(
  "token:set-connected",
  "Sets the connected contract address",
  main
)
  .addParam("contract", "The address of the deployed contract")
  .addParam("zrc20", "The ZRC20 address to link to the connected contract")
  .addParam(
    "connected",
    "The bytes representation of the connected contract to set"
  )
  .addFlag("json", "Output the result in JSON format");
