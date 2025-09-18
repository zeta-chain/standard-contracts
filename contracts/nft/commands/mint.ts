import { Command } from "commander";
import { ethers } from "ethers";
import isURL from "validator/lib/isURL";
import { loadContractArtifacts } from "./common";

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  if (!ethers.utils.isAddress(opts.contract)) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const supportedProtocols = ["https", "ipfs"];
  const isValidTokenUri = isURL(opts.tokenUri, {
    protocols: supportedProtocols,
    require_protocol: true,
  });

  if (!isValidTokenUri) {
    throw new Error(
      `Invalid token URI: ${
        opts.tokenUri
      }. Supported protocols are: ${supportedProtocols.join(", ")}.`
    );
  }

  const { abi } = loadContractArtifacts(opts.name);
  const contract = new ethers.Contract(opts.contract, abi, signer);

  const recipient = opts.to || signer.address;

  const tx = await contract.safeMint(recipient, opts.tokenUri, {
    gasLimit: opts.gasLimit,
  });

  const receipt = await tx.wait();

  const transferEventTopic = ethers.utils.id(
    "Transfer(address,address,uint256)"
  );
  const transferLog = receipt.logs.find(
    (log: any) => log.topics?.[0] === transferEventTopic
  );

  if (!transferLog) {
    throw new Error("Transfer event not found in transaction logs.");
  }

  const tokenId = ethers.BigNumber.from(transferLog.topics[3]);

  const output = {
    contractAddress: opts.contract,
    gasUsed: receipt.gasUsed?.toString?.() ?? String(receipt.gasUsed),
    mintTransactionHash: tx.hash,
    recipient: recipient,
    tokenId: tokenId.toString(),
    tokenURI: opts.tokenUri,
  };

  console.log(JSON.stringify(output));
};

export const mint = new Command("mint")
  .description("Mint an NFT on a deployed Universal NFT contract")
  .requiredOption("-r, --rpc <url>", "RPC URL")
  .requiredOption("-k, --private-key <key>", "Private key")
  .requiredOption("-c, --contract <address>", "Deployed NFT contract address")
  .option("-t, --to <address>", "Recipient address (defaults to the signer)")
  .requiredOption(
    "-u, --token-uri <uri>",
    "Token metadata URI (https:// or ipfs://)"
  )
  .option("-n, --name <name>", "Contract name", "ZetaChainUniversalNFT")
  .option("-g, --gas-limit <number>", "Gas limit", "300000")
  .action(main);
