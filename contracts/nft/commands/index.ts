#!/usr/bin/env npx tsx

import { Command } from "commander";
import { deploy } from "./deploy";
import { mint } from "./mint";
import { transfer } from "./transfer";
import { transferAndCall } from "./transferAndCall";

const program = new Command()
  .addCommand(deploy)
  .addCommand(mint)
  .addCommand(transfer)
  .addCommand(transferAndCall);

program.parse();

export { deploy, mint, transfer, transferAndCall };
