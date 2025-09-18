import fs from "fs";
import path from "path";
import { Command } from "commander";
import { ethers } from "ethers";
import { loadContractArtifacts } from "./common";

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
    const initEntry = (abi as any[]).find(
      (e: any) => e.type === "function" && e.name === "initialize"
    );
    if (!initEntry) {
      throw new Error("initialize function not found in ABI");
    }

    const initInputs = initEntry.inputs || [];
    let initArgs: any[] = [];
    // Support both EVM (5 args) and ZetaChain (6 args) variants
    if (initInputs.length === 6) {
      // (initialOwner, name, symbol, gatewayAddress, gas, uniswapRouterAddress)
      if (!opts.uniswapRouter) {
        throw new Error(
          "uniswapRouter is required by this contract's initialize signature"
        );
      }
      initArgs = [
        signer.address,
        opts.tokenName,
        opts.tokenSymbol,
        opts.gateway,
        opts.gasLimit,
        opts.uniswapRouter,
      ];
    } else if (initInputs.length === 5) {
      // (initialOwner, name, symbol, gatewayAddress, gas)
      initArgs = [
        signer.address,
        opts.tokenName,
        opts.tokenSymbol,
        opts.gateway,
        opts.gasLimit,
      ];
    } else {
      throw new Error(
        `Unsupported initialize signature with ${initInputs.length} arguments`
      );
    }

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
        implementationAddress: implementation.address,
        deployer: signer.address,
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

export const deploy = new Command("deploy")
  .description("Deploy the Universal NFT contract behind an ERC1967 proxy")
  .requiredOption(
    "-r, --rpc <url>",
    "RPC URL (default: testnet)",
    "https://zetachain-athens-evm.blockpi.network/v1/rpc/public"
  )
  .requiredOption("-k, --private-key <key>", "Private key")
  .option("-n, --name <name>", "Contract name", "Swap")
  .option(
    "-u, --uniswap-router <address>",
    "Uniswap V2 Router address (only for ZetaChain variant)",
    "0x2ca7d64A7EFE2D62A725E2B35Cf7230D6677FfEe"
  )
  .option(
    "-g, --gateway <address>",
    "Gateway address (default: testnet)",
    "0x6c533f7fe93fae114d0954697069df33c9b74fd7"
  )
  .option("--gas-limit <number>", "Gas limit for the transaction", "3000000")
  .option("-t, --token-name <name>", "Token name", "Universal NFT")
  .option("-s, --token-symbol <symbol>", "Token symbol", "UNFT")
  .action((opts) => {
    opts.gasLimit = Number(opts.gasLimit);
    main(opts).catch((err) => {
      console.error("Unhandled error:", err);
      process.exit(1);
    });
  });
