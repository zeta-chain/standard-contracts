const { getHardhatConfig } = require("@zetachain/toolkit/client");

require("@nomicfoundation/hardhat-toolbox");
require("@zetachain/toolkit/tasks");
require("@openzeppelin/hardhat-upgrades");
require("@zetachain/localnet/tasks");

require("ts-node/register");
require("./tasks/deploy");
require("./tasks/mint");
require("./tasks/transfer");
require("./tasks/setConnected");
require("./tasks/setUniversal");

// Support multiple private keys from PRIVATE_KEYS environment variable
const getPrivateKeys = () => {
  const privateKeys = process.env.PRIVATE_KEYS;
  if (privateKeys) {
    return privateKeys.split(',').map(key => key.trim()).filter(key => key.length > 0);
  }
  return process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [];
};

const config = {
  ...getHardhatConfig({ accounts: getPrivateKeys() }),
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
