import "./tasks";
import "@zetachain/localnet/tasks";
import "@nomicfoundation/hardhat-toolbox";
import "@zetachain/toolkit/tasks";
import "hardhat-deploy";

import { HardhatUserConfig } from "hardhat/config";
import { getHardhatConfig } from "@zetachain/toolkit/client";

import "@openzeppelin/hardhat-upgrades";

// Support multiple private keys from PRIVATE_KEYS environment variable
const getPrivateKeys = (): string[] => {
  const privateKeys = process.env.PRIVATE_KEYS;
  if (privateKeys) {
    return privateKeys.split(',').map(key => key.trim()).filter(key => key.length > 0);
  }
  return process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [];
};

const config: HardhatUserConfig = {
  ...getHardhatConfig({ 
    accounts: getPrivateKeys()
  }),
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
