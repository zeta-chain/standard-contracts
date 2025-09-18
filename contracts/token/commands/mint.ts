import { Command } from "commander";
import { ethers } from "ethers";
import { loadContractArtifacts } from "./common";

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.contract)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const { abi } = loadContractArtifacts(opts.name);
  const contract = new ethers.Contract(opts.contract, abi, signer);

  const recipient = opts.to || signer.address;

  const tx = await contract.mint(recipient, opts.amount, {
    gasLimit: opts.gasLimit,
  });
  const receipt = await tx.wait();

  const output = {
    contractAddress: opts.contract,
    mintTransactionHash: tx.hash,
    recipient: recipient,
    gasUsed: receipt.gasUsed?.toString?.() ?? String(receipt.gasUsed),
  };

  console.log(JSON.stringify(output));
};

export const mint = new Command("mint")
  .description("Mint tokens on a deployed Universal Token contract")
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-c, --contract <address>", "Deployed token contract address")
  .option("-t, --to <address>", "Recipient address (defaults to the signer)")
  .requiredOption("-a, --amount <amount>", "Amount of tokens to mint")
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalToken")
  .option("-g, --gas-limit <number>", "Gas limit", "1000000")
  .action(main);
