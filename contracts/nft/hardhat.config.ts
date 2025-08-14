import "./tasks";
import "@zetachain/localnet/tasks";
import "@nomicfoundation/hardhat-toolbox";
import "@zetachain/toolkit/tasks";

import { HardhatUserConfig } from "hardhat/config";
import { getHardhatConfigNetworks } from "@zetachain/networks";

import "@nomiclabs/hardhat-ethers";
import "@openzeppelin/hardhat-upgrades";

const baseNetworks: any = getHardhatConfigNetworks();

const config: HardhatUserConfig = {
  networks: {
    ...baseNetworks,
    zeta_testnet: {
      ...(baseNetworks?.zeta_testnet || {}),
      chainId: 7001,
    },
    zeta_testnet_7001: {
      url: baseNetworks?.zeta_testnet?.url,
      accounts: baseNetworks?.zeta_testnet?.accounts,
      chainId: 7001,
      gas: baseNetworks?.zeta_testnet?.gas,
      gasPrice: baseNetworks?.zeta_testnet?.gasPrice,
      gasMultiplier: baseNetworks?.zeta_testnet?.gasMultiplier,
      timeout: baseNetworks?.zeta_testnet?.timeout,
    },
  },
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
