import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { ethers } = hre;
  const [signer] = await ethers.getSigners();

  const { isAddress } = hre.ethers.utils;

  if (!isAddress(args.destination) || !isAddress(args.revertAddress)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const nftContract = await ethers.getContractAt("IERC721", args.contract);
  const approveTx = await nftContract
    .connect(signer)
    .approve(args.contract, args.tokenId);
  await approveTx.wait();

  const txOptions = {
    gasPrice: args.txOptionsGasPrice,
    gasLimit: args.txOptionsGasLimit,
  };

  const contract = await ethers.getContractAt(
    "ZetaChainUniversalNFT",
    args.contract
  );

  const gasAmount = ethers.utils.parseUnits(args.gasAmount, 18);

  const receiver = args.receiver || signer.address;

  const tx = await contract.transferCrossChain(
    args.tokenId,
    receiver,
    args.destination,
    { ...txOptions, value: gasAmount }
  );

  await tx.wait();
  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: args.contract,
        transferTransactionHash: tx.hash,
        sender: signer.address,
        tokenId: args.tokenId,
      })
    );
  } else {
    console.log(`🚀 Successfully transferred NFT to the contract.
  📜 Contract address: ${args.contract}
  🖼 NFT Contract address: ${args.nftContract}
  🆔 Token ID: ${args.tokenId}
  🔗 Transaction hash: ${tx.hash}`);
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
    "txOptionsGasPrice",
    "The gas price for the transaction",
    10000000000,
    types.int
  )
  .addOptionalParam(
    "txOptionsGasLimit",
    "The gas limit for the transaction",
    10000000,
    types.int
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
  .addParam("gasAmount", "The amount of gas to transfer", "0");
