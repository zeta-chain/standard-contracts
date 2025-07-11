import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { ZetaChainUniversalNFT } from "@/typechain-types";
import { ethers } from "ethers";

const DEFAULT_GAS_LIMIT = "300000";

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

  if (!isAddress(args.contract) || !isAddress(args.zrc20)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const contract: ZetaChainUniversalNFT = await hre.ethers.getContractAt(
    "ZetaChainUniversalNFT",
    args.contract,
    signer
  );

  const tx = await contract.setConnected(args.zrc20, args.connected, {
    gasLimit: args.gasLimit,
  });
  const receipt = await tx.wait();

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        zrc20: args.zrc20,
        connectedContractAddress: args.connected,
        transactionHash: tx.hash,
        gasUsed: receipt.gasUsed.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully set the connected contract.
📜 Contract address: ${args.contract}
🔗 ZRC20 address: ${args.zrc20}
🔗 Connected contract address: ${args.connected}
🔗 Transaction hash: ${tx.hash}
⛽ Gas used: ${receipt.gasUsed.toString()}`);
  }
};

export const nftSetConnected = task(
  "nft:set-connected",
  "Sets the connected contract address",
  main
)
  .addParam("contract", "The address of the deployed contract")
  .addParam("zrc20", "The ZRC20 address to link to the connected contract")
  .addParam(
    "connected",
    "The bytes representation of the connected contract to set"
  )
  .addOptionalParam(
    "gasLimit",
    "Gas limit for the transaction",
    DEFAULT_GAS_LIMIT
  )
  .addOptionalParam("rpc", "Custom RPC URL to use for the transaction")
  .addFlag("json", "Output the result in JSON format");
