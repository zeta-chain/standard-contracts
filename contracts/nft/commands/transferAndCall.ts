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

  if (!ethers.utils.isAddress(opts.contract)) {
    throw new Error("Invalid contract address.");
  }
  if (!ethers.utils.isAddress(opts.destination)) {
    throw new Error("Invalid destination address.");
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

  const fnSignature: string = opts.function;
  const iface = new ethers.utils.Interface([`function ${fnSignature}`]);
  const fnName = fnSignature.slice(0, fnSignature.indexOf("("));

  const raw = (opts.payload ?? "").trim();
  const encodedMessage = iface.encodeFunctionData(fnName, [
    normalizeBytes(raw),
  ]);

  const tx = await contract.transferCrossChainAndCall(
    opts.tokenId,
    receiver,
    opts.destination,
    encodedMessage,
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

export const transferAndCall = new Command("transfer-and-call")
  .description(
    "Transfer an NFT cross-chain and call a function on the receiver"
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
  .requiredOption(
    "-f, --function <signature>",
    "Function signature, e.g. hello(bytes)"
  )
  .option("-p, --payload <hexOrText>", "Payload to pass as bytes", "")
  .option("-a, --gas-amount <amount>", "Gas amount (in ZETA wei)", "0")
  .option("-t, --receiver <address>", "Receiver address (defaults to signer)")
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalNFT")
  .action(main);
