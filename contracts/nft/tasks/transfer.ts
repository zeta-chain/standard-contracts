import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

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

  if (!isAddress(args.destination) || !isAddress(args.revertAddress)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const nftContract = await ethers.getContractAt(
    "IERC721",
    args.contract,
    signer
  );

  const approveTx = await nftContract.approve(args.contract, args.tokenId, {
    gasLimit: args.gasLimit,
  });

  await approveTx.wait();

  const contract = await ethers.getContractAt(
    "ZetaChainUniversalNFT",
    args.contract,
    signer
  );

  const gasAmount = ethers.utils.parseUnits(args.gasAmount, 18);
  const receiver = args.receiver || signer.address;

  const tx = await contract.transferCrossChain(
    args.tokenId,
    receiver,
    args.destination,
    {
      gasLimit: args.gasLimit,
      value: gasAmount,
    }
  );

  const receipt = await tx.wait();

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        transferTransactionHash: tx.hash,
        sender: signer.address,
        tokenId: args.tokenId,
        gasUsed: receipt.gasUsed.toString(),
      })
    );
  } else {
    console.log(`🚀 Successfully transferred NFT to the contract.
📜 Contract address: ${args.contract}
🖼 NFT Contract address: ${args.contract}
🆔 Token ID: ${args.tokenId}
🔗 Transaction hash: ${tx.hash}
⛽ Gas used: ${receipt.gasUsed.toString()}`);
  }
};

export const nftTransfer = task(
  "nft:transfer",
  "Transfer and lock an NFT",
  main
)
  .addOptionalParam("receiver", "The address to receive the NFT")
  .addParam("contract", "The contract being transferred from")
  .addParam("tokenId", "The ID of the NFT to transfer")
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
    7000000,
    types.int
  )
  .addFlag("isArbitraryCall", "Whether the call is arbitrary")
  .addFlag("json", "Output the result in JSON format")
  .addOptionalParam(
    "destination",
    "ZRC-20 of the gas token of the destination chain",
    "0x0000000000000000000000000000000000000000"
  )
  .addOptionalParam("rpc", "Custom RPC URL to use for the transaction")
  .addParam("gasAmount", "The amount of gas to transfer", "0");
