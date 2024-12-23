import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { isAddress } = hre.ethers.utils;

  const network = hre.network.name;

  const [signer] = await hre.ethers.getSigners();
  if (signer === undefined) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  if (
    !isAddress(args.gateway) ||
    (args.uniswapRouter && !isAddress(args.uniswapRouter))
  ) {
    throw new Error("Invalid Ethereum address provided.");
  }

  const factory: any = await hre.ethers.getContractFactory(args.name);

  const contract = await hre.upgrades.deployProxy(factory, [
    signer.address,
    args.tokenName,
    args.tokenSymbol,
    args.gateway,
    args.gasLimit,
    ...(args.uniswapRouter ? [args.uniswapRouter] : []),
  ]);

  if (args.json) {
    console.log(
      JSON.stringify({
        contractAddress: contract.address,
        deployer: signer.address,
        network: network,
        transactionHash: contract.deployTransaction.hash,
      })
    );
  } else {
    console.log(`🚀 Successfully deployed "${args.name}" contract on ${network}.
📜 Contract address: ${contract.address}
🔗 Transaction hash: ${contract.deployTransaction.hash}`);
  }
};

export const tokenDeploy = task(
  "token:deploy",
  "Deploy a universal token contract",
  main
)
  .addFlag("json", "Output the result in JSON format")
  .addOptionalParam("tokenName", "Token name", "Universal Token")
  .addOptionalParam("tokenSymbol", "Token symbol", "UFT")
  .addOptionalParam(
    "name",
    "The contract name to deploy",
    "ZetaChainUniversalToken"
  )
  .addOptionalParam(
    "gasLimit",
    "Gas limit for the transaction",
    1000000,
    types.int
  )
  .addOptionalParam(
    "gateway",
    "Gateway address (default: ZetaChain Gateway)",
    "0x6c533f7fe93fae114d0954697069df33c9b74fd7"
  )
  .addOptionalParam("uniswapRouter", "Uniswap v2 Router address");
