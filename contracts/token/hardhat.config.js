"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
require("@zetachain/localnet/tasks");
require("@nomicfoundation/hardhat-toolbox");
require("@zetachain/toolkit/tasks");
require("@nomiclabs/hardhat-ethers");
require("@openzeppelin/hardhat-upgrades");
// @ts-ignore
const client_1 = require("@zetachain/toolkit/client");
const config = {
    ...(0, client_1.getHardhatConfig)({ accounts: [process.env.PRIVATE_KEY] }),
    solidity: {
        compilers: [
            {
                settings: {
                    optimizer: {
                        enabled: true,
                        runs: 1000,
                    },
                },
                version: "0.8.26",
            },
        ],
    },
};
exports.default = config;
