import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { ethers } from "ethers";

const DEFAULT_GAS_LIMIT = "1000000";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { ethers } = hre;
  const { isAddress } = hre.ethers.utils;

  let signer;

  if (args.rpc) {
    const provider = new ethers.providers.JsonRpcProvider(args.rpc);
    signer = new ethers.Wallet(process.env.PRIVATE_KEY || "", provider);
  } else {
    [signer] = await ethers.getSigners();
  }

  if (!isAddress(args.to) || !isAddress(args.revertAddress)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const contract = await ethers.getContractAt(
    "ZetaChainUniversalToken",
    args.from,
    signer
  );

  const value = ethers.utils.parseUnits(args.amount, 18);
  const tokenApprove = await contract.approve(args.from, value, {
    gasLimit: args.gasLimit,
  });
  await tokenApprove.wait();

  const gasAmount = ethers.utils.parseUnits(args.gasAmount, 18);
  const receiver = args.receiver || signer.address;

  const tx = await (contract as any).transferCrossChain(
    args.to,
    receiver,
    args.amount,
    {
      gasLimit: args.gasLimit,
      value: gasAmount,
    }
  );

  const receipt = await tx.wait();

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.from,
        transferTransactionHash: tx.hash,
        sender: signer.address,
        amount: args.amount,
        gasUsed: receipt.gasUsed.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully transferred token.
📜 Contract address: ${args.from}
👤 Amount: ${args.amount}
🔗 Transaction hash: ${tx.hash}
⛽ Gas used: ${receipt.gasUsed.toString()}`);
  }
};

export const tokenTransfer = task(
  "token:transfer",
  "Transfer and lock a token",
  main
)
  .addParam("from", "The contract being transferred from")
  .addOptionalParam(
    "gasLimit",
    "Gas limit for the transaction",
    DEFAULT_GAS_LIMIT
  )
  .addFlag("callOnRevert", "Whether to call on revert")
  .addOptionalParam(
    "revertAddress",
    "The address to call on revert",
    "0x0000000000000000000000000000000000000000"
  )
  .addOptionalParam("revertMessage", "The message to send on revert", "0x")
  .addOptionalParam(
    "onRevertGasLimit",
    "The gas limit for the revert transaction",
    "7000000"
  )
  .addOptionalParam("receiver", "The receiver of the token")
  .addFlag("isArbitraryCall", "Whether the call is arbitrary")
  .addFlag("json", "Output the result in JSON format")
  .addOptionalParam(
    "to",
    "ZRC-20 of the gas token of the destination chain",
    "0x0000000000000000000000000000000000000000"
  )
  .addOptionalParam("rpc", "Custom RPC URL to use for the transaction")
  .addParam("gasAmount", "The amount of gas to transfer", "0")
  .addParam("amount", "The amount of tokens to transfer", "0");
