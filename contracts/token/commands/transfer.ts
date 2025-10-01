import { Command } from "commander";
import { ethers } from "ethers";

import { loadContractArtifacts } from "./common";

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.destination)) {
    throw new Error("Invalid destination ZRC-20 address.");
  }
  if (!ethers.utils.isAddress(opts.from)) {
    throw new Error("Invalid source contract address.");
  }

  const { abi } = loadContractArtifacts(opts.name);
  const contract = new ethers.Contract(opts.from, abi, signer);

  const amountInWei = ethers.utils.parseUnits(String(opts.amount), 18);
  const approveTx = await contract.approve(opts.from, amountInWei, {
    gasLimit: opts.gasLimit,
  });
  await approveTx.wait();

  const gasAmount = ethers.utils.parseUnits(String(opts.gasAmount), 18);
  const receiver = opts.receiver || signer.address;

  const tx = await contract.transferCrossChain(
    opts.destination,
    receiver,
    String(opts.amount),
    {
      gasLimit: opts.gasLimit,
      value: gasAmount,
    }
  );

  const receipt = await tx.wait();

  console.log(
    JSON.stringify({
      amount: String(opts.amount),
      contractAddress: opts.from,
      gasUsed: receipt.gasUsed?.toString?.() ?? String(receipt.gasUsed),
      sender: signer.address,
      transferTransactionHash: tx.hash,
    })
  );
};

export const transfer = new Command("transfer")
  .description(
    "Transfer and lock a token cross-chain\n" +
      "Note: Requires compiled contracts. Run 'forge build' before using this command."
  )
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-f, --from <address>", "Token contract address")
  .option("--gas-limit <number>", "Gas limit", "1000000")
  .option(
    "-d, --destination <address>",
    "Destination ZRC-20",
    "0x0000000000000000000000000000000000000000"
  )
  .option("-a, --gas-amount <amount>", "Gas amount (in ZETA wei)", "0")
  .option("--receiver <address>", "Receiver address (defaults to signer)")
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalToken")
  .requiredOption("-A, --amount <amount>", "Amount of tokens to transfer", "0")
  .action(main);
