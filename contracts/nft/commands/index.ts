#!/usr/bin/env npx tsx

import { Command } from "commander";
import { deploy } from "./deploy";
import { mint } from "./mint";

const program = new Command().addCommand(deploy).addCommand(mint);

program.parse();
