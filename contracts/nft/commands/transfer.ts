import { Command } from "commander";
import { ethers } from "ethers";

import { loadContractArtifacts } from "./common";

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.destination)) {
    throw new Error("Invalid destination address.");
  }
  if (!ethers.utils.isAddress(opts.contract)) {
    throw new Error("Invalid contract address.");
  }

  const { abi: erc721Abi } = loadContractArtifacts(
    "ERC721Upgradeable",
    "ERC721Upgradeable.sol"
  );
  const nftContract = new ethers.Contract(opts.contract, erc721Abi, signer);
  const approveTx = await nftContract.approve(opts.contract, opts.tokenId, {
    gasLimit: opts.gasLimit,
  });
  await approveTx.wait();

  const { abi } = loadContractArtifacts(opts.name);
  const contract = new ethers.Contract(opts.contract, abi, signer);
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
  .description(
    "Transfer and lock an NFT cross-chain\n" +
      "Note: Requires compiled contracts. Run 'forge build' before using this command."
  )
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-c, --contract <address>", "NFT contract address")
  .requiredOption("-i, --token-id <id>", "Token ID")
  .option("--gas-limit <number>", "Gas limit", "1000000")
  .option(
    "-d, --destination <address>",
    "Destination ZRC-20",
    "0x0000000000000000000000000000000000000000"
  )
  .option("-a, --gas-amount <amount>", "Gas amount (in ZETA wei)", "0")
  .option("-t, --receiver <address>", "Receiver address (defaults to signer)")
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalNFT")
  .action(main);
