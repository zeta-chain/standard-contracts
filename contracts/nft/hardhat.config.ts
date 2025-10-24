import "@zetachain/localnet/tasks";
import "@nomicfoundation/hardhat-toolbox";
import "@zetachain/toolkit/tasks";

import { HardhatUserConfig } from "hardhat/config";
// @ts-ignore
import { getHardhatConfig } from "@zetachain/toolkit/utils";

import "@nomiclabs/hardhat-ethers";
import "@openzeppelin/hardhat-upgrades";

const config: HardhatUserConfig = {
  ...getHardhatConfig({ accounts: [process.env.PRIVATE_KEY] }),
  solidity: {
    compilers: [
      {
        settings: {
          evmVersion: "cancun",
          optimizer: {
            enabled: true,
            runs: 1000,
          },
          viaIR: true,
        },
        version: "0.8.26",
      },
    ],
  },
};

export default config;
