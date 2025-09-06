const { nftSetUniversal } = require("./setUniversal");
const { nftMint } = require("./mint");
const { nftTransfer } = require("./transfer");
const { nftSetConnected } = require("./setConnected");
const { nftDeploy } = require("./deploy");

module.exports = {
  nftSetUniversal,
  nftMint,
  nftTransfer,
  nftSetConnected,
  nftDeploy
};
