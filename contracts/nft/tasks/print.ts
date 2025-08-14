import { task } from "hardhat/config";

export const printNet = task("print:net", "Print network config", async (_, hre) => {
  // eslint-disable-next-line no-console
  console.log(JSON.stringify(hre.network.config, null, 2));
});
