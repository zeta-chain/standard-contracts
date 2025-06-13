import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
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

  if (signer === undefined) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  if (!isAddress(args.contract)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const contract: any = await hre.ethers.getContractAt(
    args.name,
    args.contract,
    signer
  );

  const recipient = args.to || signer.address;

  const tx = await contract.mint(recipient, args.amount, {
    gasLimit: args.gasLimit,
  });
  const receipt = await tx.wait();

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        mintTransactionHash: tx.hash,
        recipient: recipient,
        gasUsed: receipt.gasUsed.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully minted token.
📜 Contract address: ${args.contract}
👤 Recipient: ${recipient}
🔗 Transaction hash: ${tx.hash}
⛽ Gas used: ${receipt.gasUsed.toString()}`);
  }
};

export const tokenMint = task("token:mint", "Mint a universal token", main)
  .addParam("contract", "The address of the deployed token contract")
  .addOptionalParam(
    "to",
    "The recipient address, defaults to the signer address"
  )
  .addParam("amount", "The amount of tokens to mint")
  .addOptionalParam(
    "name",
    "The contract name to interact with",
    "ZetaChainUniversalToken"
  )
  .addOptionalParam(
    "gasLimit",
    "Gas limit for the transaction",
    DEFAULT_GAS_LIMIT
  )
  .addOptionalParam("rpc", "Custom RPC URL to use for the transaction")
  .addFlag("json", "Output the result in JSON format");
