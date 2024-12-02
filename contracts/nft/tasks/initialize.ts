import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const network = hre.network.name;

  const [signer] = await hre.ethers.getSigners();
  if (signer === undefined) {
    throw new Error(
      `Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`
    );
  }

  const contract: any = await hre.ethers.getContractAt(
    args.name,
    args.contract
  );

  const tx = await contract.initialize(
    signer.address,
    args.tokenName,
    args.tokenSymbol,
    args.gateway,
    args.gasLimit,
    ...(args.uniswapRouter ? [args.uniswapRouter] : []),
    {
      gasLimit: args.initializeGasLimit,
    }
  );

  await tx.wait();
};

task("initialize", "Initialize the NFT contract", main)
  .addOptionalParam("name", "The contract name to deploy", "Universal")
  .addFlag("json", "Output the result in JSON format")
  .addParam("contract", "The address of the deployed contract")
  .addOptionalParam("tokenName", "NFT name", "Universal NFT")
  .addOptionalParam("tokenSymbol", "NFT symbol", "UNFT")
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
  .addOptionalParam(
    "initializeGasLimit",
    "Gas limit for initialize transaction",
    10000000,
    types.int
  )
  .addOptionalParam("uniswapRouter", "Uniswap v2 Router address");
