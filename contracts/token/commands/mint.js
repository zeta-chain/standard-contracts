"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mint = void 0;
const commander_1 = require("commander");
const ethers_1 = require("ethers");
const common_1 = require("./common");
const main = async (opts) => {
    const provider = new ethers_1.ethers.providers.JsonRpcProvider(opts.rpc);
    const signer = new ethers_1.ethers.Wallet(opts.privateKey, provider);
    if (!ethers_1.ethers.utils.isAddress(opts.contract)) {
        throw new Error("Invalid Ethereum address provided.");
    }
    const { abi } = (0, common_1.loadContractArtifacts)(opts.name);
    const contract = new ethers_1.ethers.Contract(opts.contract, abi, signer);
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
exports.mint = new commander_1.Command("mint")
    .description("Mint tokens on a deployed Universal Token contract")
    .requiredOption("-r, --rpc <url>", "RPC URL")
    .requiredOption("-k, --private-key <key>", "Private key")
    .requiredOption("-c, --contract <address>", "Deployed token contract address")
    .option("-t, --to <address>", "Recipient address (defaults to the signer)")
    .requiredOption("-a, --amount <amount>", "Amount of tokens to mint")
    .option("-n, --name <name>", "Contract name", "ZetaChainUniversalToken")
    .option("-g, --gas-limit <number>", "Gas limit", "1000000")
    .action(main);
