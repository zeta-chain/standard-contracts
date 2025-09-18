import { Command } from "commander";
import { ethers } from "ethers";

const DEFAULT_GAS_LIMIT = 1000000;

const erc721Abi = ["function approve(address to, uint256 tokenId) external"];

const universalNftAbi = [
  "function transferCrossChain(uint256 tokenId, address receiver, address destination) payable",
];

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.destination)) {
    throw new Error("Invalid destination address.");
  }
  if (!ethers.utils.isAddress(opts.contract)) {
    throw new Error("Invalid contract address.");
  }

  const nftContract = new ethers.Contract(opts.contract, erc721Abi, signer);
  const approveTx = await nftContract.approve(opts.contract, opts.tokenId, {
    gasLimit: opts.gasLimit,
  });
  await approveTx.wait();

  const contract = new ethers.Contract(opts.contract, universalNftAbi, signer);
  const gasAmount = ethers.utils.parseUnits(opts.gasAmount, 18);
  const receiver = opts.receiver || signer.address;

  const tx = await contract.transferCrossChain(
    opts.tokenId,
    receiver,
    opts.destination,
    {
      gasLimit: opts.gasLimit,
      value: gasAmount,
    }
  );

  const receipt = await tx.wait();

  console.log(
    JSON.stringify({
      contractAddress: opts.contract,
      gasUsed: receipt.gasUsed?.toString?.() ?? String(receipt.gasUsed),
      sender: signer.address,
      tokenId: String(opts.tokenId),
      transferTransactionHash: tx.hash,
    })
  );
};

export const transfer = new Command("transfer")
  .description("Transfer and lock an NFT cross-chain")
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-c, --contract <address>", "NFT contract address")
  .requiredOption("-i, --token-id <id>", "Token ID")
  .option("--gas-limit <number>", "Gas limit", String(DEFAULT_GAS_LIMIT))
  .option(
    "-d, --destination <address>",
    "Destination ZRC-20",
    "0x0000000000000000000000000000000000000000"
  )
  .option("-a, --gas-amount <amount>", "Gas amount (in ZETA wei)", "0")
  .option("-t, --receiver <address>", "Receiver address (defaults to signer)")
  .action((opts) => {
    opts.gasLimit = Number(opts.gasLimit);
    main(opts).catch((err) => {
      console.error(
        "Transfer failed:",
        err instanceof Error ? err.message : err
      );
      process.exit(1);
    });
  });
