#!/usr/bin/env npx tsx
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const commander_1 = require("commander");
const deploy_1 = require("./deploy");
const mint_1 = require("./mint");
const transfer_1 = require("./transfer");
const transferAndCall_1 = require("./transferAndCall");
const program = new commander_1.Command()
    .addCommand(deploy_1.deploy)
    .addCommand(mint_1.mint)
    .addCommand(transfer_1.transfer)
    .addCommand(transferAndCall_1.transferAndCall);
program.parse();
