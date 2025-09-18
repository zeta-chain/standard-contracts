"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadContractArtifacts = void 0;
const path_1 = __importDefault(require("path"));
const fs_1 = __importDefault(require("fs"));
/**
 * Load contract artifacts (ABI & bytecode) compiled by Foundry (out/** path)
 */
const loadContractArtifacts = (contractName, sourceName) => {
    const sourcePath = sourceName || `${contractName}.sol`;
    const artifactPath = path_1.default.join(__dirname, `../out/${sourcePath}/${contractName}.json`);
    try {
        const artifact = JSON.parse(fs_1.default.readFileSync(artifactPath, "utf8"));
        return {
            abi: artifact.abi,
            bytecode: artifact.bytecode,
        };
    }
    catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(`Unable to load contract artifacts for ${contractName}: ${message}`);
    }
};
exports.loadContractArtifacts = loadContractArtifacts;
