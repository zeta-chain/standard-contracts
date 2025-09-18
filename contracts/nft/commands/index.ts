#!/usr/bin/env npx tsx

import { Command } from "commander";
import { deploy } from "./deploy";
import { mint } from "./mint";
import { transfer } from "./transfer";

const program = new Command()
  .addCommand(deploy)
  .addCommand(mint)
  .addCommand(transfer);

program.parse();
