import { Command } from "commander";
import { ethers } from "ethers";

import { loadContractArtifacts, compileNote } from "./common";

const main = async (opts: any) => {
  const provider = new ethers.providers.JsonRpcProvider(opts.rpc);
  const signer = new ethers.Wallet(opts.privateKey, provider);

  const network = await provider.getNetwork();
  const networkInfo = (network as any).name ?? network.chainId;

  try {
    const { abi, bytecode } = loadContractArtifacts(opts.name);
    const factory = new ethers.ContractFactory(abi, bytecode, signer);

    const implementation = await factory.deploy();
    await implementation.deployed();

    const iface = new ethers.utils.Interface(abi);
    const initArgs = [
      signer.address,
      opts.tokenName,
      opts.tokenSymbol,
      opts.gateway,
      opts.gasLimit,
      ...(opts.uniswapRouter ? [opts.uniswapRouter] : []),
    ];

    const initData = iface.encodeFunctionData("initialize", initArgs);

    const { abi: proxyAbi, bytecode: proxyBytecode } = loadContractArtifacts(
      "ERC1967Proxy",
      "ERC1967Proxy.sol"
    );

    const proxyFactory = new ethers.ContractFactory(
      proxyAbi,
      proxyBytecode,
      signer
    );
    const proxy = await proxyFactory.deploy(implementation.address, initData);
    await proxy.deployed();

    console.log(
      JSON.stringify({
        contractAddress: proxy.address,
        deployer: signer.address,
        implementationAddress: implementation.address,
        network: networkInfo,
        transactionHash: proxy.deployTransaction?.hash,
      })
    );
  } catch (err) {
    console.error(
      "Deployment failed:",
      err instanceof Error ? err.message : err
    );
    process.exit(1);
  }
};

const summary = "Deploy the Universal NFT contract behind an ERC1967 proxy";

export const deploy = new Command("deploy")
  .summary(summary)
  .description(`${summary}\n${compileNote}`)
  .requiredOption(
    "-r, --rpc <url>",
    "RPC URL (default: testnet)",
    "https://zetachain-athens-evm.blockpi.network/v1/rpc/public"
  )
  .requiredOption("-k, --private-key <key>", "Private key")
  .option("-n, --name <name>", "Contract name", "Swap")
  .option(
    "-u, --uniswap-router <address>",
    "Uniswap V2 Router address (only for ZetaChain variant)"
  )
  .option(
    "-g, --gateway <address>",
    "Gateway address (default: testnet)",
    "0x6c533f7fe93fae114d0954697069df33c9b74fd7"
  )
  .option("--gas-limit <number>", "Gas limit for the transaction", "3000000")
  .option("-t, --token-name <name>", "Token name", "Universal NFT")
  .option("-s, --token-symbol <symbol>", "Token symbol", "UNFT")
  .action(main);
