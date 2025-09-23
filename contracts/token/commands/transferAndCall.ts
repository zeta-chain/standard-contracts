import { Command } from "commander";
import { ethers } from "ethers";
import { loadContractArtifacts } from "./common";

const normalizeBytes = (input?: string): string => {
  if (!input || input === "0x" || input === "") return "0x";

  if (input.startsWith("0x") || input.startsWith("0X")) {
    const hex = input.slice(2);
    return hex.length % 2 === 0 ? "0x" + hex : "0x0" + hex;
  }

  return ethers.utils.hexlify(ethers.utils.toUtf8Bytes(input));
};

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.to)) {
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

  const fnSignature: string = opts.function;
  const iface = new ethers.utils.Interface([`function ${fnSignature}`]);
  const fnName = fnSignature.slice(0, fnSignature.indexOf("("));

  const raw = (opts.payload ?? "").trim();
  const encodedMessage = iface.encodeFunctionData(fnName, [
    normalizeBytes(raw),
  ]);

  const tx = await contract.transferCrossChainAndCall(
    opts.to,
    receiver,
    String(opts.amount),
    encodedMessage,
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

export const transferAndCall = new Command("transfer-and-call")
  .description("Transfer token cross-chain and call a function on the receiver")
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-f, --from <address>", "Token contract address")
  .option("--gas-limit <number>", "Gas limit", "1000000")
  .option(
    "-t, --to <address>",
    "Destination ZRC-20",
    "0x0000000000000000000000000000000000000000"
  )
  .requiredOption(
    "-F, --function <signature>",
    "Function signature, e.g. hello(bytes)"
  )
  .option("-p, --payload <hexOrText>", "Payload to pass as bytes", "")
  .option("-a, --gas-amount <amount>", "Gas amount (in ZETA wei)", "0")
  .option("--receiver <address>", "Receiver address (defaults to signer)")
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalToken")
  .requiredOption("-A, --amount <amount>", "Amount of tokens to transfer", "0")
  .action(main);
