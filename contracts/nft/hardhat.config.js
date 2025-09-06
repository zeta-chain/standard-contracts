const { getHardhatConfig } = require("@zetachain/toolkit/client");

require("@nomicfoundation/hardhat-toolbox");
require("@zetachain/toolkit/tasks");
require("@nomiclabs/hardhat-ethers");
require("@openzeppelin/hardhat-upgrades");
require("@zetachain/localnet/tasks");

require("ts-node/register");
require("./tasks/deploy");
require("./tasks/mint");
require("./tasks/transfer");
require("./tasks/setConnected");
require("./tasks/setUniversal");

const config = {
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

module.exports = config;
