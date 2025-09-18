"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deploy = void 0;
const commander_1 = require("commander");
const ethers_1 = require("ethers");
const common_1 = require("./common");
const main = async (opts) => {
    const provider = new ethers_1.ethers.providers.JsonRpcProvider(opts.rpc);
    const signer = new ethers_1.ethers.Wallet(opts.privateKey, provider);
    const network = await provider.getNetwork();
    const networkInfo = network.name ?? network.chainId;
    try {
        const { abi, bytecode } = (0, common_1.loadContractArtifacts)(opts.name);
        const factory = new ethers_1.ethers.ContractFactory(abi, bytecode, signer);
        const implementation = await factory.deploy();
        await implementation.deployed();
        const iface = new ethers_1.ethers.utils.Interface(abi);
        const initArgs = [
            signer.address,
            opts.tokenName,
            opts.tokenSymbol,
            opts.gateway,
            opts.gasLimit,
            ...(opts.uniswapRouter ? [opts.uniswapRouter] : []),
        ];
        const initData = iface.encodeFunctionData("initialize", initArgs);
        const { abi: proxyAbi, bytecode: proxyBytecode } = (0, common_1.loadContractArtifacts)("ERC1967Proxy", "ERC1967Proxy.sol");
        const proxyFactory = new ethers_1.ethers.ContractFactory(proxyAbi, proxyBytecode, signer);
        const proxy = await proxyFactory.deploy(implementation.address, initData);
        await proxy.deployed();
        console.log(JSON.stringify({
            contractAddress: proxy.address,
            implementationAddress: implementation.address,
            deployer: signer.address,
            network: networkInfo,
            transactionHash: proxy.deployTransaction?.hash,
        }));
    }
    catch (err) {
        console.error("Deployment failed:", err instanceof Error ? err.message : err);
        process.exit(1);
    }
};
exports.deploy = new commander_1.Command("deploy")
    .description("Deploy the Universal Token contract behind an ERC1967 proxy")
    .requiredOption("-r, --rpc <url>", "RPC URL (default: testnet)", "https://zetachain-athens-evm.blockpi.network/v1/rpc/public")
    .requiredOption("-k, --private-key <key>", "Private key")
    .option("-n, --name <name>", "Contract name", "ZetaChainUniversalToken")
    .option("-u, --uniswap-router <address>", "Uniswap V2 Router address (only for ZetaChain variant)")
    .option("-g, --gateway <address>", "Gateway address (default: testnet)", "0x6c533f7fe93fae114d0954697069df33c9b74fd7")
    .option("--gas-limit <number>", "Gas limit for the transaction", "3000000")
    .option("-t, --token-name <name>", "Token name", "Universal Token")
    .option("-s, --token-symbol <symbol>", "Token symbol", "UFT")
    .action(main);
