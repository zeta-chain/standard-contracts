import { task, types } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import ZRC20ABI from "@zetachain/protocol-contracts/abi/ZRC20.sol/ZRC20.json";

const main = async (args: any, hre: HardhatRuntimeEnvironment) => {
  const { ethers } = hre;
  const [signer] = await ethers.getSigners();

  const txOptions = {
    gasPrice: args.txOptionsGasPrice,
    gasLimit: args.txOptionsGasLimit,
  };

  const callOptions = {
    isArbitraryCall: args.callOptionsIsArbitraryCall,
    gasLimit: args.callOptionsGasLimit,
  };

  if (args.callOptionsIsArbitraryCall && !args.function) {
    throw new Error("You must provide a function to call");
  }

  let message;

  const valuesArray = args.values.map((value: any, index: any) => {
    const type = args.types[index];

    if (type === "bool") {
      try {
        return JSON.parse(value.toLowerCase());
      } catch (e) {
        throw new Error(`Invalid boolean value: ${value}`);
      }
    } else if (type.startsWith("uint") || type.startsWith("int")) {
      return ethers.BigNumber.from(value);
    } else {
      return value;
    }
  });

  const encodedParameters = hre.ethers.utils.defaultAbiCoder.encode(
    JSON.parse(args.types),
    valuesArray
  );

  if (args.callOptionsIsArbitraryCall && args.function) {
    const functionSignature = hre.ethers.utils.id(args.function).slice(0, 10);
    message = hre.ethers.utils.hexlify(
      hre.ethers.utils.concat([functionSignature, encodedParameters])
    );
  } else {
    message = encodedParameters;
  }

  const revertOptions = {
    abortAddress: "0x0000000000000000000000000000000000000000", // not used
    callOnRevert: args.callOnRevert,
    onRevertGasLimit: args.onRevertGasLimit,
    revertAddress: args.revertAddress,
    revertMessage: ethers.utils.hexlify(
      ethers.utils.toUtf8Bytes(args.revertMessage)
    ),
  };

  let tx;

  let contract;
  try {
    contract = await ethers.getContractAt("Universal", args.from);
    await (contract as any).isUniversal();
    const gasLimit = hre.ethers.BigNumber.from(args.txOptionsGasLimit);
    const zrc20 = new ethers.Contract(args.to, ZRC20ABI.abi, signer);
    const [, gasFee] = await zrc20.withdrawGasFeeWithGasLimit(gasLimit);
    const zrc20TransferTx = await zrc20.approve(args.from, gasFee, txOptions);
    await zrc20TransferTx.wait();
  } catch (e) {
    contract = await ethers.getContractAt("Example", args.from);
  }

  const gasAmount = ethers.utils.parseUnits(args.gasAmount, 18);

  tx = await (contract as any).sendMessage(
    args.to,
    message,
    callOptions,
    revertOptions,
    { ...txOptions, value: gasAmount }
  );

  await tx.wait();
};

task("transfer", "Make a cross-chain call", main)
  .addParam("from", "The contract being transferred from")
  .addOptionalParam(
    "txOptionsGasPrice",
    "The gas price for the transaction",
    10000000000,
    types.int
  )
  .addOptionalParam(
    "txOptionsGasLimit",
    "The gas limit for the transaction",
    7000000,
    types.int
  )
  .addFlag("callOnRevert", "Whether to call on revert")
  .addOptionalParam(
    "revertAddress",
    "The address to call on revert",
    "0x0000000000000000000000000000000000000000"
  )
  .addOptionalParam("revertMessage", "The message to send on revert", "0x")
  .addOptionalParam(
    "onRevertGasLimit",
    "The gas limit for the revert transaction",
    7000000,
    types.int
  )
  .addFlag("json", "Output the result in JSON format")
  .addOptionalParam(
    "to",
    "ZRC-20 of the gas token of the destination chain",
    "0x0000000000000000000000000000000000000000"
  )
  .addParam("gasAmount", "The amount of gas to transfer", "0")
  .addParam("types", `The types of the parameters (example: '["string"]')`)
  .addFlag("callOptionsIsArbitraryCall", "Call any function")
  .addOptionalParam(
    "callOptionsGasLimit",
    "The gas limit for the call",
    7000000,
    types.int
  )
  .addOptionalParam(
    "function",
    "The function to call on the destination chain (only for arbitrary calls)"
  )
  .addOptionalParam("erc20", "The address of the ERC20 token to transfer")
  .addVariadicPositionalParam("values", "The values of the parameters");
