"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const ethers_1 = require("ethers");
const config_1 = require("hardhat/config");
const main = async (args, hre) => {
    const [signer] = await hre.ethers.getSigners();
    if (!signer) {
        throw new Error(`Wallet not found. Please, run "npx hardhat account --save" or set PRIVATE_KEY env variable (for example, in a .env file)`);
    }
    const contract = await hre.ethers.getContractAt("Example", args.contract);
    const tx = await contract.setConnected(args.targetChainId, ethers_1.ethers.utils.arrayify(args.targetContract));
    await tx.wait();
    console.log("Successfully set the connected contract.");
};
(0, config_1.task)("connect", "Sets connected contract", main)
    .addParam("contract", "The address of the deployed contract")
    .addParam("targetChainId", "Chain ID of the destination chain")
    .addParam("targetContract", "The address of the connected contract to set")
    .addFlag("json", "Output the result in JSON format");
