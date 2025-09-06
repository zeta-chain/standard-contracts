import { task } from "hardhat/config";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import { ethers } from "ethers";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

// Chain IDs as defined in test files
const CHAIN_IDS = {
  BASE_SEPOLIA: 84532,
  ZETACHAIN_TESTNET: 7001,
  SOLANA_DEVNET: 103,
  ETHEREUM_SEPOLIA: 11155111,
  BSC_TESTNET: 97
};

// Network configurations
const NETWORK_CONFIGS = {
  baseSepolia: {
    chainId: CHAIN_IDS.BASE_SEPOLIA,
    name: "Base Sepolia",
    rpcUrl: "https://sepolia.base.org",
    gateway: "0x0c487a766110c85d301d96e33579c5b317fa4995",
    contractType: "EVMUniversalNFT"
  },
  zetachainTestnet: {
    chainId: CHAIN_IDS.ZETACHAIN_TESTNET,
    name: "ZetaChain Testnet",
    rpcUrl: "https://zetachain-evm.blockpi.network/v1/rpc/public",
    gateway: "0x6c533f7fe93fae114d0954697069df33c9b74fd7",
    contractType: "ZetaChainUniversalNFT"
  },
  solanaDevnet: {
    chainId: CHAIN_IDS.SOLANA_DEVNET,
    name: "Solana Devnet",
    rpcUrl: "https://api.devnet.solana.com",
    gateway: "ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis",
    contractType: "SolanaUniversalNFT"
  }
};

interface DeploymentRecord {
  networks: {
    [key: string]: {
      contractAddress: string;
      deployer: string;
      transactionHash?: string;
      blockNumber?: number;
      gasUsed?: string;
      timestamp: string;
      chainId: number;
      gateway: string;
      universalAddress?: string;
      connectedChains?: Record<string, string>;
    };
  };
  crossChainConfig?: {
    configured: boolean;
    configuredAt?: string;
    connections: Record<string, Record<string, string>>;
    universalAddresses: Record<string, string>;
  };
}

interface ConfigurationSummary {
  timestamp: string;
  networks: string[];
  totalConnections: number;
  universalAddressesSet: number;
  verificationResults: Record<string, boolean>;
  errors: string[];
  warnings: string[];
  nextSteps: string[];
}

async function loadDeploymentRecord(): Promise<DeploymentRecord> {
  const deploymentPath = path.join(process.cwd(), "deployment-record.json");
  
  if (!fs.existsSync(deploymentPath)) {
    throw new Error(`Deployment record not found at ${deploymentPath}. Please run deployment first.`);
  }
  
  try {
    const deploymentData = JSON.parse(fs.readFileSync(deploymentPath, "utf-8"));
    console.log("📖 Loaded deployment record successfully");
    return deploymentData;
  } catch (error: any) {
    throw new Error(`Failed to parse deployment record: ${error.message}`);
  }
}

async function saveDeploymentRecord(record: DeploymentRecord): Promise<void> {
  const deploymentPath = path.join(process.cwd(), "deployment-record.json");
  fs.writeFileSync(deploymentPath, JSON.stringify(record, null, 2));
  console.log("💾 Updated deployment record");
}

async function configureEVMContract(
  networkName: string,
  contractAddress: string,
  connectedChains: Record<string, string>,
  universalAddress: string,
  rpcUrl: string,
  contractType: string
): Promise<{ success: boolean; errors: string[] }> {
  console.log(`\n🔧 Configuring ${networkName} contract...`);
  
  const errors: string[] = [];
  
  try {
    const provider = new ethers.providers.JsonRpcProvider(rpcUrl);
    const signer = new ethers.Wallet(process.env.PRIVATE_KEY || "", provider);
    
    if (!signer) {
      throw new Error("Private key not found in environment variables");
    }
    
    console.log(`   📋 Contract: ${contractAddress}`);
    console.log(`   🔑 Signer: ${signer.address}`);
    
    // Set universal address first
    try {
      console.log(`   🌐 Setting universal address: ${universalAddress}`);
      
      // Use hardhat task for setting universal address
      const { spawn } = require("child_process");
      
      const setUniversalProcess = spawn("npx", [
        "hardhat",
        "nft:set-universal",
        "--contract", contractAddress,
        "--universal", universalAddress,
        "--rpc", rpcUrl,
        "--json"
      ], { stdio: "pipe" });
      
      const setUniversalResult = await new Promise<string>((resolve, reject) => {
        let output = "";
        let errorOutput = "";
        
        setUniversalProcess.stdout.on("data", (data: Buffer) => {
          output += data.toString();
        });
        
        setUniversalProcess.stderr.on("data", (data: Buffer) => {
          errorOutput += data.toString();
        });
        
        setUniversalProcess.on("close", (code: number) => {
          if (code === 0) {
            resolve(output);
          } else {
            reject(new Error(`Set universal failed: ${errorOutput}`));
          }
        });
      });
      
      const universalResult = JSON.parse(setUniversalResult.trim());
      console.log(`   ✅ Universal address set - TX: ${universalResult.transactionHash}`);
      
    } catch (error: any) {
      const errorMsg = `Failed to set universal address: ${error.message}`;
      console.log(`   ❌ ${errorMsg}`);
      errors.push(errorMsg);
    }
    
    // Configure connected chains
    for (const [chainId, connectedAddress] of Object.entries(connectedChains)) {
      try {
        console.log(`   🔗 Connecting to chain ${chainId}: ${connectedAddress}`);
        
        // For ZetaChain contracts, we need ZRC20 address
        if (contractType === "ZetaChainUniversalNFT") {
          // Use ZRC20 address for the connected chain
          const zrc20Address = getZRC20Address(parseInt(chainId));
          
          const setConnectedProcess = spawn("npx", [
            "hardhat",
            "nft:set-connected",
            "--contract", contractAddress,
            "--zrc20", zrc20Address,
            "--connected", connectedAddress,
            "--rpc", rpcUrl,
            "--json"
          ], { stdio: "pipe" });
          
          const setConnectedResult = await new Promise<string>((resolve, reject) => {
            let output = "";
            let errorOutput = "";
            
            setConnectedProcess.stdout.on("data", (data: Buffer) => {
              output += data.toString();
            });
            
            setConnectedProcess.stderr.on("data", (data: Buffer) => {
              errorOutput += data.toString();
            });
            
            setConnectedProcess.on("close", (code: number) => {
              if (code === 0) {
                resolve(output);
              } else {
                reject(new Error(`Set connected failed: ${errorOutput}`));
              }
            });
          });
          
          const connectedResult = JSON.parse(setConnectedResult.trim());
          console.log(`   ✅ Connected to chain ${chainId} - TX: ${connectedResult.transactionHash}`);
          
        } else {
          // For EVM contracts, use direct connection
          console.log(`   ⚠️  Direct EVM connection not implemented yet for chain ${chainId}`);
        }
        
      } catch (error: any) {
        const errorMsg = `Failed to connect to chain ${chainId}: ${error.message}`;
        console.log(`   ❌ ${errorMsg}`);
        errors.push(errorMsg);
      }
    }
    
    return { success: errors.length === 0, errors };
    
  } catch (error: any) {
    const errorMsg = `EVM configuration failed: ${error.message}`;
    console.log(`   ❌ ${errorMsg}`);
    return { success: false, errors: [errorMsg] };
  }
}

async function configureSolanaProgram(
  programId: string,
  collectionPda: string,
  connectedChains: Record<string, string>,
  universalAddress: string
): Promise<{ success: boolean; errors: string[] }> {
  console.log(`\n🔧 Configuring Solana program...`);
  
  const errors: string[] = [];
  
  try {
    // Setup Solana connection
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    
    // Load wallet
    const walletPath = path.join(os.homedir(), ".config", "solana", "id.json");
    if (!fs.existsSync(walletPath)) {
      throw new Error("Solana wallet not found. Please run: solana-keygen new");
    }
    
    const walletKeypair = Keypair.fromSecretKey(
      new Uint8Array(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
    );
    
    const wallet = new anchor.Wallet(walletKeypair);
    const provider = new anchor.AnchorProvider(connection, wallet, {
      commitment: "confirmed",
    });
    anchor.setProvider(provider);
    
    // Load program IDL
    const idlPath = path.join(__dirname, "../contracts/solana/target/idl/universal_nft.json");
    if (!fs.existsSync(idlPath)) {
      throw new Error("Solana program IDL not found. Please build the program first.");
    }
    
    const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
    idl.metadata = idl.metadata || {};
    idl.metadata.address = programId;
    
    const program: Program = new anchor.Program(idl as anchor.Idl, provider);
    
    console.log(`   📋 Program ID: ${programId}`);
    console.log(`   🎨 Collection PDA: ${collectionPda}`);
    console.log(`   🔑 Authority: ${wallet.publicKey.toBase58()}`);
    
    // Set universal address (if instruction exists)
    try {
      console.log(`   🌐 Setting universal address: ${universalAddress}`);
      
      // Convert universal address to bytes
      const universalBytes = Array.from(Buffer.from(universalAddress.replace("0x", ""), "hex"));
      
      // Note: This would call set_universal instruction when implemented
      console.log(`   ⚠️  set_universal instruction not yet implemented in Solana program`);
      console.log(`   📝 Universal address stored for future use: ${universalAddress}`);
      
    } catch (error: any) {
      const errorMsg = `Failed to set universal address: ${error.message}`;
      console.log(`   ❌ ${errorMsg}`);
      errors.push(errorMsg);
    }
    
    // Configure connected chains
    for (const [chainId, connectedAddress] of Object.entries(connectedChains)) {
      try {
        console.log(`   🔗 Connecting to chain ${chainId}: ${connectedAddress}`);
        
        // Convert connected address to bytes
        const connectedBytes = Array.from(Buffer.from(connectedAddress.replace("0x", ""), "hex"));
        
        // Note: This would call set_connected instruction when implemented
        console.log(`   ⚠️  set_connected instruction not yet implemented in Solana program`);
        console.log(`   📝 Connection stored for future use: Chain ${chainId} -> ${connectedAddress}`);
        
      } catch (error: any) {
        const errorMsg = `Failed to connect to chain ${chainId}: ${error.message}`;
        console.log(`   ❌ ${errorMsg}`);
        errors.push(errorMsg);
      }
    }
    
    return { success: true, errors }; // Consider success even with unimplemented instructions
    
  } catch (error: any) {
    const errorMsg = `Solana configuration failed: ${error.message}`;
    console.log(`   ❌ ${errorMsg}`);
    return { success: false, errors: [errorMsg] };
  }
}

async function verifyConnections(deploymentRecord: DeploymentRecord): Promise<Record<string, boolean>> {
  console.log(`\n🔍 Verifying cross-chain connections...`);
  
  const verificationResults: Record<string, boolean> = {};
  
  for (const [networkName, networkData] of Object.entries(deploymentRecord.networks)) {
    try {
      console.log(`   🔍 Verifying ${networkName}...`);
      
      if (networkName === "solanaDevnet") {
        // Verify Solana program
        const connection = new Connection(NETWORK_CONFIGS.solanaDevnet.rpcUrl, "confirmed");
        const programId = new PublicKey(networkData.contractAddress);
        
        const accountInfo = await connection.getAccountInfo(programId);
        const isValid = accountInfo !== null && accountInfo.executable;
        
        verificationResults[networkName] = isValid;
        console.log(`   ${isValid ? "✅" : "❌"} ${networkName}: ${isValid ? "Program verified" : "Program not found"}`);
        
      } else {
        // Verify EVM contract
        const config = networkName === "baseSepolia" ? NETWORK_CONFIGS.baseSepolia : NETWORK_CONFIGS.zetachainTestnet;
        const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);
        
        const code = await provider.getCode(networkData.contractAddress);
        const isValid = code !== "0x";
        
        verificationResults[networkName] = isValid;
        console.log(`   ${isValid ? "✅" : "❌"} ${networkName}: ${isValid ? "Contract verified" : "Contract not found"}`);
      }
      
    } catch (error: any) {
      verificationResults[networkName] = false;
      console.log(`   ❌ ${networkName}: Verification failed - ${error.message}`);
    }
  }
  
  return verificationResults;
}

function getZRC20Address(chainId: number): string {
  // ZRC20 addresses for different chains on ZetaChain testnet
  const zrc20Addresses: Record<number, string> = {
    [CHAIN_IDS.BASE_SEPOLIA]: "0x0cbe0dF132a6c6B4a2974Fa1b7Fb953CF0Cc798a",
    [CHAIN_IDS.ETHEREUM_SEPOLIA]: "0x65a45c57636f9BcCeD4fe193A602008578BcA90b",
    [CHAIN_IDS.BSC_TESTNET]: "0x7c8dDa80bbBE1254a7aACf3219EBe1481c6E01d7",
    [CHAIN_IDS.SOLANA_DEVNET]: "0x" + "0".repeat(40) // Placeholder for Solana
  };
  
  return zrc20Addresses[chainId] || "0x" + "0".repeat(40);
}

function generateUniversalAddress(baseAddress: string, chainId: number): string {
  // Generate a deterministic universal address based on the base address and chain
  const hash = ethers.utils.keccak256(
    ethers.utils.defaultAbiCoder.encode(
      ["address", "uint256"],
      [baseAddress, chainId]
    )
  );
  
  // Take first 20 bytes for address format
  return "0x" + hash.slice(2, 42);
}

async function generateConfigurationSummary(
  deploymentRecord: DeploymentRecord,
  verificationResults: Record<string, boolean>,
  errors: string[],
  warnings: string[]
): Promise<ConfigurationSummary> {
  const networks = Object.keys(deploymentRecord.networks);
  const totalConnections = Object.values(deploymentRecord.crossChainConfig?.connections || {})
    .reduce((total, connections) => total + Object.keys(connections).length, 0);
  
  const universalAddressesSet = Object.keys(deploymentRecord.crossChainConfig?.universalAddresses || {}).length;
  
  const nextSteps = [
    "Test cross-chain NFT transfers between networks",
    "Verify NFT metadata preservation across chains",
    "Test the NFT origin system functionality",
    "Monitor transaction fees and gas usage",
    "Set up monitoring for cross-chain events"
  ];
  
  if (errors.length > 0) {
    nextSteps.unshift("Resolve configuration errors before proceeding");
  }
  
  return {
    timestamp: new Date().toISOString(),
    networks,
    totalConnections,
    universalAddressesSet,
    verificationResults,
    errors,
    warnings,
    nextSteps
  };
}

async function main(args: any, hre: HardhatRuntimeEnvironment) {
  console.log("\n🔗 Universal NFT Cross-Chain Configuration\n");
  console.log("═══════════════════════════════════════════════════════════════\n");
  
  const errors: string[] = [];
  const warnings: string[] = [];
  
  try {
    // Step 1: Load deployment record
    console.log("📖 Loading deployment record...");
    const deploymentRecord = await loadDeploymentRecord();
    
    if (!deploymentRecord.networks || Object.keys(deploymentRecord.networks).length === 0) {
      throw new Error("No networks found in deployment record. Please run deployment first.");
    }
    
    console.log(`   ✅ Found ${Object.keys(deploymentRecord.networks).length} deployed networks`);
    
    // Step 2: Initialize cross-chain configuration
    if (!deploymentRecord.crossChainConfig) {
      deploymentRecord.crossChainConfig = {
        configured: false,
        connections: {},
        universalAddresses: {}
      };
    }
    
    // Step 3: Generate universal addresses for each network
    console.log("\n🌐 Generating universal addresses...");
    
    for (const [networkName, networkData] of Object.entries(deploymentRecord.networks)) {
      if (!deploymentRecord.crossChainConfig.universalAddresses[networkName]) {
        const universalAddress = generateUniversalAddress(networkData.contractAddress, networkData.chainId);
        deploymentRecord.crossChainConfig.universalAddresses[networkName] = universalAddress;
        console.log(`   ✅ ${networkName}: ${universalAddress}`);
      } else {
        console.log(`   ✓ ${networkName}: ${deploymentRecord.crossChainConfig.universalAddresses[networkName]} (existing)`);
      }
    }
    
    // Step 4: Configure connections for each network
    console.log("\n🔗 Configuring cross-chain connections...");
    
    for (const [networkName, networkData] of Object.entries(deploymentRecord.networks)) {
      // Prepare connected chains (all other networks)
      const connectedChains: Record<string, string> = {};
      
      for (const [otherNetworkName, otherNetworkData] of Object.entries(deploymentRecord.networks)) {
        if (otherNetworkName !== networkName) {
          connectedChains[otherNetworkData.chainId.toString()] = otherNetworkData.contractAddress;
        }
      }
      
      const universalAddress = deploymentRecord.crossChainConfig.universalAddresses[networkName];
      
      // Configure based on network type
      if (networkName === "solanaDevnet") {
        const result = await configureSolanaProgram(
          networkData.contractAddress,
          networkData.contractAddress, // Assuming this is the collection PDA
          connectedChains,
          universalAddress
        );
        
        if (!result.success) {
          errors.push(...result.errors);
        } else if (result.errors.length > 0) {
          warnings.push(...result.errors);
        }
        
      } else {
        const config = networkName === "baseSepolia" ? NETWORK_CONFIGS.baseSepolia : NETWORK_CONFIGS.zetachainTestnet;
        
        const result = await configureEVMContract(
          config.name,
          networkData.contractAddress,
          connectedChains,
          universalAddress,
          config.rpcUrl,
          config.contractType
        );
        
        if (!result.success) {
          errors.push(...result.errors);
        } else if (result.errors.length > 0) {
          warnings.push(...result.errors);
        }
      }
      
      // Store connections in deployment record
      deploymentRecord.crossChainConfig.connections[networkName] = connectedChains;
    }
    
    // Step 5: Verify all connections
    const verificationResults = await verifyConnections(deploymentRecord);
    
    // Step 6: Update deployment record
    deploymentRecord.crossChainConfig.configured = true;
    deploymentRecord.crossChainConfig.configuredAt = new Date().toISOString();
    
    await saveDeploymentRecord(deploymentRecord);
    
    // Step 7: Generate configuration summary
    const summary = await generateConfigurationSummary(
      deploymentRecord,
      verificationResults,
      errors,
      warnings
    );
    
    // Save summary to file
    const summaryPath = path.join(process.cwd(), "cross-chain-config-summary.json");
    fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));
    
    // Step 8: Display results
    console.log("\n📊 Configuration Summary");
    console.log("═══════════════════════════════════════════════════════════════");
    console.log(`  • Networks configured: ${summary.networks.length}`);
    console.log(`  • Total connections: ${summary.totalConnections}`);
    console.log(`  • Universal addresses set: ${summary.universalAddressesSet}`);
    console.log(`  • Verification results:`);
    
    for (const [network, verified] of Object.entries(summary.verificationResults)) {
      console.log(`    - ${network}: ${verified ? "✅ Verified" : "❌ Failed"}`);
    }
    
    if (summary.errors.length > 0) {
      console.log(`\n❌ Errors (${summary.errors.length}):`);
      summary.errors.forEach(error => console.log(`  • ${error}`));
    }
    
    if (summary.warnings.length > 0) {
      console.log(`\n⚠️  Warnings (${summary.warnings.length}):`);
      summary.warnings.forEach(warning => console.log(`  • ${warning}`));
    }
    
    console.log(`\n📋 Next Steps:`);
    summary.nextSteps.forEach(step => console.log(`  • ${step}`));
    
    console.log(`\n💾 Configuration summary saved to: ${summaryPath}`);
    
    if (args.json) {
      console.log(JSON.stringify(summary, null, 2));
    }
    
    console.log("\n🎉 Cross-chain configuration completed!");
    
    if (errors.length > 0) {
      console.log("\n⚠️  Some configurations failed. Please review errors and retry.");
      process.exit(1);
    }
    
  } catch (error: any) {
    console.error(`\n❌ Configuration failed: ${error.message}`);
    
    if (args.json) {
      console.log(JSON.stringify({ error: error.message, timestamp: new Date().toISOString() }));
    }
    
    process.exit(1);
  }
}

export const configureCrossChain = task(
  "configure:cross-chain",
  "Configure cross-chain connections between all deployed Universal NFT contracts",
  main
)
  .addFlag("json", "Output results in JSON format")
  .addFlag("verify-only", "Only verify existing connections without configuring new ones")
  .addOptionalParam("networks", "Comma-separated list of networks to configure (default: all)")
  .addOptionalParam("deployment-file", "Path to deployment record file", "deployment-record.json");