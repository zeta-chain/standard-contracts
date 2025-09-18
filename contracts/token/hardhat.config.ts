import "@zetachain/localnet/tasks";
import "@nomicfoundation/hardhat-toolbox";
import "@zetachain/toolkit/tasks";
import "@nomiclabs/hardhat-ethers";
import "@openzeppelin/hardhat-upgrades";

// @ts-ignore
import { getHardhatConfig } from "@zetachain/toolkit/client";
import { HardhatUserConfig } from "hardhat/config";

const config: HardhatUserConfig = {
  ...getHardhatConfig({ accounts: [process.env.PRIVATE_KEY] }),
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

export default config;
