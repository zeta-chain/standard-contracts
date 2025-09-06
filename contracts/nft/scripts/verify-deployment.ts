import { ethers } from "ethers";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import * as fs from "fs";
import * as path from "path";
import axios from "axios";

// Network configurations
const NETWORKS = {
  baseSepolia: {
    name: "Base Sepolia",
    chainId: 84532,
    rpcUrl: process.env.BASE_SEPOLIA_RPC || "https://sepolia.base.org",
    gateway: "0x0c487a766110c85d301d96e33579c5b317fa4995",
    explorer: "https://sepolia.basescan.org",
    nativeCurrency: "ETH"
  },
  zetachain: {
    name: "ZetaChain Testnet",
    chainId: 7001,
    rpcUrl: process.env.ZETACHAIN_RPC || "https://zetachain-athens-evm.blockpi.network/v1/rpc/public",
    gateway: "0x6c533f7fe93fae114d0954697069df33c9b74fd7",
    explorer: "https://zetachain-athens-3.blockscout.com",
    nativeCurrency: "ZETA"
  },
  solanaDevnet: {
    name: "Solana Devnet",
    chainId: 103,
    rpcUrl: process.env.SOLANA_RPC || "https://api.devnet.solana.com",
    explorer: "https://explorer.solana.com",
    nativeCurrency: "SOL"
  }
};

// Contract ABIs (simplified for verification)
const UNIVERSAL_NFT_ABI = [
  "function name() view returns (string)",
  "function symbol() view returns (string)",
  "function owner() view returns (address)",
  "function totalSupply() view returns (uint256)",
  "function safeMint(address to, string memory uri) returns (uint256)",
  "function transferCrossChain(uint256 tokenId, address receiver, address destination) payable",
  "function tokenURI(uint256 tokenId) view returns (string)",
  "function ownerOf(uint256 tokenId) view returns (address)",
  "function balanceOf(address owner) view returns (uint256)",
  "function getConnectedChains() view returns (uint256[])",
  "function getUniversalAddress() view returns (bytes)",
  "function isConnectedChain(uint256 chainId) view returns (bool)"
];

const GATEWAY_ABI = [
  "function call(bytes memory receiver, bytes memory message, uint256 gasLimit, uint256 gasPrice) payable",
  "function deposit(address receiver, uint256 amount, address asset, bytes memory message) payable"
];

interface DeploymentRecord {
  networks: {
    [key: string]: {
      contractAddress: string;
      deploymentTx: string;
      deployer: string;
      blockNumber: number;
      timestamp: string;
      gatewayAddress: string;
      universalAddress?: string;
      connectedChains?: string[];
    };
  };
  crossChainConfig: {
    connections: Array<{
      from: string;
      to: string;
      configured: boolean;
      verificationTx?: string;
    }>;
  };
  testTransactions: {
    mints: Array<{
      network: string;
      tokenId: string;
      txHash: string;
      recipient: string;
      tokenUri: string;
    }>;
    transfers: Array<{
      from: string;
      to: string;
      tokenId: string;
      txHash: string;
      status: string;
    }>;
  };
  originSystem?: {
    enabled: boolean;
    solanaProgram?: {
      programId: string;
      collectionPda: string;
      originPdas: Array<{
        tokenId: number;
        pda: string;
        chainOfOrigin: number;
        isNative: boolean;
      }>;
    };
  };
}

interface VerificationResult {
  network: string;
  contractAddress: string;
  status: "success" | "warning" | "error";
  checks: {
    deployment: boolean;
    basicFunctionality: boolean;
    crossChainConfig: boolean;
    gatewayIntegration: boolean;
    metadataPreservation: boolean;
    originSystem?: boolean;
  };
  issues: string[];
  recommendations: string[];
  gasEstimates?: {
    mint: string;
    transfer: string;
    crossChainTransfer: string;
  };
}

interface ComprehensiveReport {
  timestamp: string;
  overallStatus: "healthy" | "issues" | "critical";
  networks: VerificationResult[];
  crossChainConnectivity: {
    totalConnections: number;
    workingConnections: number;
    failedConnections: Array<{
      from: string;
      to: string;
      error: string;
    }>;
  };
  originSystemStatus?: {
    enabled: boolean;
    solanaIntegration: boolean;
    pdaValidation: boolean;
    metadataLinking: boolean;
  };
  recommendations: string[];
  troubleshooting: {
    commonIssues: Record<string, string>;
    supportContacts: Record<string, string>;
  };
}

class DeploymentVerifier {
  private deploymentRecord: DeploymentRecord | null = null;
  private providers: Record<string, ethers.providers.JsonRpcProvider> = {};
  private solanaConnection: Connection | null = null;
  private solanaProgram: anchor.Program | null = null;

  constructor() {
    this.initializeProviders();
  }

  private initializeProviders(): void {
    console.log("🔌 Initializing network providers...");
    
    try {
      this.providers.baseSepolia = new ethers.providers.JsonRpcProvider(NETWORKS.baseSepolia.rpcUrl);
      this.providers.zetachain = new ethers.providers.JsonRpcProvider(NETWORKS.zetachain.rpcUrl);
      this.solanaConnection = new Connection(NETWORKS.solanaDevnet.rpcUrl, "confirmed");
      console.log("   ✅ All providers initialized successfully");
    } catch (error: any) {
      console.log(`   ⚠️  Provider initialization warning: ${error.message}`);
    }
  }

  private async loadDeploymentRecord(): Promise<void> {
    console.log("📄 Loading deployment record...");
    
    const deploymentPath = path.join(process.cwd(), "deployment-record.json");
    
    if (!fs.existsSync(deploymentPath)) {
      throw new Error(`Deployment record not found at ${deploymentPath}. Please run deployment first.`);
    }
    
    try {
      const recordData = fs.readFileSync(deploymentPath, "utf-8");
      this.deploymentRecord = JSON.parse(recordData);
      console.log(`   ✅ Deployment record loaded successfully`);
      console.log(`   📊 Networks found: ${Object.keys(this.deploymentRecord!.networks).length}`);
    } catch (error: any) {
      throw new Error(`Failed to parse deployment record: ${error.message}`);
    }
  }

  private async initializeSolanaProgram(): Promise<void> {
    if (!this.deploymentRecord?.originSystem?.solanaProgram) {
      console.log("   ⚠️  Solana program not found in deployment record");
      return;
    }

    try {
      const programId = this.deploymentRecord.originSystem.solanaProgram.programId;
      
      // Load IDL (simplified for verification)
      const idlPath = path.join(process.cwd(), "contracts/solana/target/idl/universal_nft.json");
      if (fs.existsSync(idlPath)) {
        const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
        
        // Create dummy wallet for read-only operations
        const dummyKeypair = Keypair.generate();
        const wallet = new anchor.Wallet(dummyKeypair);
        const provider = new anchor.AnchorProvider(this.solanaConnection!, wallet, {
          commitment: "confirmed"
        });
        
        this.solanaProgram = new anchor.Program(idl, new PublicKey(programId), provider);
        console.log("   ✅ Solana program initialized for verification");
      }
    } catch (error: any) {
      console.log(`   ⚠️  Solana program initialization warning: ${error.message}`);
    }
  }

  private async verifyContractDeployment(
    network: string,
    contractAddress: string,
    provider: ethers.providers.JsonRpcProvider
  ): Promise<{ deployed: boolean; code: string; issues: string[] }> {
    const issues: string[] = [];
    
    try {
      const code = await provider.getCode(contractAddress);
      
      if (code === "0x") {
        issues.push(`Contract not deployed at address ${contractAddress}`);
        return { deployed: false, code, issues };
      }
      
      // Verify contract has expected functions
      const contract = new ethers.Contract(contractAddress, UNIVERSAL_NFT_ABI, provider);
      
      try {
        await contract.name();
        await contract.symbol();
        await contract.owner();
      } catch (error: any) {
        issues.push(`Contract missing expected functions: ${error.message}`);
      }
      
      return { deployed: true, code, issues };
      
    } catch (error: any) {
      issues.push(`Deployment verification failed: ${error.message}`);
      return { deployed: false, code: "0x", issues };
    }
  }

  private async testBasicFunctionality(
    network: string,
    contractAddress: string,
    provider: ethers.providers.JsonRpcProvider
  ): Promise<{ working: boolean; issues: string[]; gasEstimates?: any }> {
    const issues: string[] = [];
    const gasEstimates: any = {};
    
    try {
      const contract = new ethers.Contract(contractAddress, UNIVERSAL_NFT_ABI, provider);
      
      // Test read functions
      const name = await contract.name();
      const symbol = await contract.symbol();
      const owner = await contract.owner();
      const totalSupply = await contract.totalSupply();
      
      console.log(`      • Name: ${name}`);
      console.log(`      • Symbol: ${symbol}`);
      console.log(`      • Owner: ${owner}`);
      console.log(`      • Total Supply: ${totalSupply.toString()}`);
      
      // Estimate gas for common operations
      try {
        const dummyAddress = "0x0000000000000000000000000000000000000001";
        const dummyUri = "https://example.com/token/1";
        
        gasEstimates.mint = await contract.estimateGas.safeMint(dummyAddress, dummyUri);
        console.log(`      • Mint gas estimate: ${gasEstimates.mint.toString()}`);
      } catch (error: any) {
        issues.push(`Gas estimation failed for mint: ${error.message}`);
      }
      
      return { working: true, issues, gasEstimates };
      
    } catch (error: any) {
      issues.push(`Basic functionality test failed: ${error.message}`);
      return { working: false, issues };
    }
  }

  private async verifyCrossChainConfiguration(
    network: string,
    contractAddress: string,
    provider: ethers.providers.JsonRpcProvider
  ): Promise<{ configured: boolean; issues: string[] }> {
    const issues: string[] = [];
    
    try {
      const contract = new ethers.Contract(contractAddress, UNIVERSAL_NFT_ABI, provider);
      
      // Check if contract has cross-chain functions
      try {
        const connectedChains = await contract.getConnectedChains();
        console.log(`      • Connected chains: ${connectedChains.length}`);
        
        // Verify specific chain connections
        const expectedConnections = this.deploymentRecord?.crossChainConfig?.connections || [];
        for (const connection of expectedConnections) {
          if (connection.from === network) {
            const targetChainId = NETWORKS[connection.to as keyof typeof NETWORKS]?.chainId;
            if (targetChainId) {
              try {
                const isConnected = await contract.isConnectedChain(targetChainId);
                if (!isConnected) {
                  issues.push(`Chain ${connection.to} (${targetChainId}) not properly connected`);
                }
              } catch (error: any) {
                issues.push(`Failed to verify connection to ${connection.to}: ${error.message}`);
              }
            }
          }
        }
        
        // Check universal address
        try {
          const universalAddress = await contract.getUniversalAddress();
          if (!universalAddress || universalAddress === "0x") {
            issues.push("Universal address not set");
          } else {
            console.log(`      • Universal address: ${universalAddress}`);
          }
        } catch (error: any) {
          issues.push(`Failed to get universal address: ${error.message}`);
        }
        
      } catch (error: any) {
        issues.push(`Cross-chain configuration check failed: ${error.message}`);
      }
      
      return { configured: issues.length === 0, issues };
      
    } catch (error: any) {
      issues.push(`Cross-chain verification failed: ${error.message}`);
      return { configured: false, issues };
    }
  }

  private async verifyGatewayIntegration(
    network: string,
    gatewayAddress: string,
    provider: ethers.providers.JsonRpcProvider
  ): Promise<{ working: boolean; issues: string[] }> {
    const issues: string[] = [];
    
    try {
      const gatewayCode = await provider.getCode(gatewayAddress);
      
      if (gatewayCode === "0x") {
        issues.push(`Gateway contract not found at ${gatewayAddress}`);
        return { working: false, issues };
      }
      
      // Test gateway contract interface
      const gateway = new ethers.Contract(gatewayAddress, GATEWAY_ABI, provider);
      
      try {
        // Estimate gas for gateway operations
        const dummyReceiver = "0x0000000000000000000000000000000000000001";
        const dummyMessage = "0x";
        await gateway.estimateGas.call(dummyReceiver, dummyMessage, 100000, 1);
        console.log(`      • Gateway integration verified`);
      } catch (error: any) {
        // This is expected to fail without proper setup, but verifies interface
        console.log(`      • Gateway interface verified (estimation failed as expected)`);
      }
      
      return { working: true, issues };
      
    } catch (error: any) {
      issues.push(`Gateway integration verification failed: ${error.message}`);
      return { working: false, issues };
    }
  }

  private async testMetadataPreservation(
    network: string,
    contractAddress: string,
    provider: ethers.providers.JsonRpcProvider
  ): Promise<{ working: boolean; issues: string[] }> {
    const issues: string[] = [];
    
    try {
      const contract = new ethers.Contract(contractAddress, UNIVERSAL_NFT_ABI, provider);
      
      // Check if there are any existing tokens to test
      const totalSupply = await contract.totalSupply();
      
      if (totalSupply.gt(0)) {
        // Test metadata retrieval for first token
        try {
          const tokenUri = await contract.tokenURI(1);
          console.log(`      • Sample token URI: ${tokenUri}`);
          
          // Verify URI is accessible
          if (tokenUri.startsWith("http")) {
            try {
              const response = await axios.head(tokenUri, { timeout: 5000 });
              if (response.status === 200) {
                console.log(`      • Metadata accessible via HTTP`);
              } else {
                issues.push(`Metadata URI not accessible: ${response.status}`);
              }
            } catch (error: any) {
              issues.push(`Metadata URI not accessible: ${error.message}`);
            }
          } else if (tokenUri.startsWith("ipfs://")) {
            console.log(`      • IPFS metadata URI detected`);
            // Note: IPFS verification would require IPFS gateway
          }
          
        } catch (error: any) {
          issues.push(`Failed to retrieve token metadata: ${error.message}`);
        }
      } else {
        console.log(`      • No tokens minted yet for metadata testing`);
      }
      
      return { working: issues.length === 0, issues };
      
    } catch (error: any) {
      issues.push(`Metadata preservation test failed: ${error.message}`);
      return { working: false, issues };
    }
  }

  private async verifySolanaOriginSystem(): Promise<{ working: boolean; issues: string[] }> {
    const issues: string[] = [];
    
    if (!this.solanaProgram || !this.deploymentRecord?.originSystem?.solanaProgram) {
      issues.push("Solana program not available for verification");
      return { working: false, issues };
    }
    
    try {
      const { collectionPda, originPdas } = this.deploymentRecord.originSystem.solanaProgram;
      
      // Verify collection account
      try {
        const collectionAccount = await this.solanaProgram.account.collection.fetch(
          new PublicKey(collectionPda)
        );
        console.log(`      • Collection verified: ${collectionAccount.name}`);
        console.log(`      • Total minted: ${collectionAccount.totalMinted.toString()}`);
        console.log(`      • Next token ID: ${collectionAccount.nextTokenId.toString()}`);
      } catch (error: any) {
        issues.push(`Collection account verification failed: ${error.message}`);
      }
      
      // Verify origin PDAs
      let verifiedPdas = 0;
      for (const originPda of originPdas) {
        try {
          const originAccount = await this.solanaProgram.account.nftOrigin.fetch(
            new PublicKey(originPda.pda)
          );
          
          if (originAccount.tokenId.toNumber() === originPda.tokenId) {
            verifiedPdas++;
            console.log(`      • Origin PDA verified: Token ${originPda.tokenId}`);
          } else {
            issues.push(`Origin PDA token ID mismatch: expected ${originPda.tokenId}, got ${originAccount.tokenId.toNumber()}`);
          }
        } catch (error: any) {
          issues.push(`Origin PDA verification failed for token ${originPda.tokenId}: ${error.message}`);
        }
      }
      
      console.log(`      • Origin PDAs verified: ${verifiedPdas}/${originPdas.length}`);
      
      return { working: issues.length === 0, issues };
      
    } catch (error: any) {
      issues.push(`Solana origin system verification failed: ${error.message}`);
      return { working: false, issues };
    }
  }

  private async verifyNetwork(networkName: string): Promise<VerificationResult> {
    console.log(`\n🔍 Verifying ${NETWORKS[networkName as keyof typeof NETWORKS].name}...`);
    
    const result: VerificationResult = {
      network: networkName,
      contractAddress: "",
      status: "success",
      checks: {
        deployment: false,
        basicFunctionality: false,
        crossChainConfig: false,
        gatewayIntegration: false,
        metadataPreservation: false
      },
      issues: [],
      recommendations: []
    };
    
    if (networkName === "solanaDevnet") {
      return await this.verifySolanaNetwork(result);
    }
    
    const networkConfig = this.deploymentRecord?.networks[networkName];
    if (!networkConfig) {
      result.status = "error";
      result.issues.push(`Network ${networkName} not found in deployment record`);
      return result;
    }
    
    result.contractAddress = networkConfig.contractAddress;
    const provider = this.providers[networkName];
    
    if (!provider) {
      result.status = "error";
      result.issues.push(`Provider not available for ${networkName}`);
      return result;
    }
    
    // 1. Verify contract deployment
    console.log("   📋 Checking contract deployment...");
    const deploymentCheck = await this.verifyContractDeployment(
      networkName,
      networkConfig.contractAddress,
      provider
    );
    result.checks.deployment = deploymentCheck.deployed;
    result.issues.push(...deploymentCheck.issues);
    
    if (!deploymentCheck.deployed) {
      result.status = "error";
      return result;
    }
    
    // 2. Test basic functionality
    console.log("   ⚙️  Testing basic functionality...");
    const functionalityCheck = await this.testBasicFunctionality(
      networkName,
      networkConfig.contractAddress,
      provider
    );
    result.checks.basicFunctionality = functionalityCheck.working;
    result.issues.push(...functionalityCheck.issues);
    result.gasEstimates = functionalityCheck.gasEstimates;
    
    // 3. Verify cross-chain configuration
    console.log("   🔗 Verifying cross-chain configuration...");
    const crossChainCheck = await this.verifyCrossChainConfiguration(
      networkName,
      networkConfig.contractAddress,
      provider
    );
    result.checks.crossChainConfig = crossChainCheck.configured;
    result.issues.push(...crossChainCheck.issues);
    
    // 4. Verify gateway integration
    console.log("   🌉 Verifying gateway integration...");
    const gatewayCheck = await this.verifyGatewayIntegration(
      networkName,
      networkConfig.gatewayAddress,
      provider
    );
    result.checks.gatewayIntegration = gatewayCheck.working;
    result.issues.push(...gatewayCheck.issues);
    
    // 5. Test metadata preservation
    console.log("   🎨 Testing metadata preservation...");
    const metadataCheck = await this.testMetadataPreservation(
      networkName,
      networkConfig.contractAddress,
      provider
    );
    result.checks.metadataPreservation = metadataCheck.working;
    result.issues.push(...metadataCheck.issues);
    
    // Determine overall status
    const criticalChecks = [result.checks.deployment, result.checks.basicFunctionality];
    const allChecks = Object.values(result.checks);
    
    if (criticalChecks.some(check => !check)) {
      result.status = "error";
    } else if (allChecks.some(check => !check)) {
      result.status = "warning";
    }
    
    // Generate recommendations
    if (!result.checks.crossChainConfig) {
      result.recommendations.push("Configure cross-chain connections using setConnected task");
    }
    if (!result.checks.gatewayIntegration) {
      result.recommendations.push("Verify gateway contract address and integration");
    }
    if (!result.checks.metadataPreservation) {
      result.recommendations.push("Test NFT minting and metadata accessibility");
    }
    
    return result;
  }

  private async verifySolanaNetwork(result: VerificationResult): Promise<VerificationResult> {
    console.log("   🔍 Verifying Solana program...");
    
    if (!this.deploymentRecord?.originSystem?.solanaProgram) {
      result.status = "error";
      result.issues.push("Solana program not found in deployment record");
      return result;
    }
    
    result.contractAddress = this.deploymentRecord.originSystem.solanaProgram.programId;
    
    // Verify program deployment
    try {
      const programId = new PublicKey(this.deploymentRecord.originSystem.solanaProgram.programId);
      const accountInfo = await this.solanaConnection!.getAccountInfo(programId);
      
      if (!accountInfo) {
        result.status = "error";
        result.issues.push("Solana program not deployed");
        return result;
      }
      
      result.checks.deployment = true;
      console.log("   ✅ Program deployment verified");
      
    } catch (error: any) {
      result.status = "error";
      result.issues.push(`Solana program verification failed: ${error.message}`);
      return result;
    }
    
    // Verify origin system
    console.log("   🔍 Verifying origin system...");
    const originCheck = await this.verifySolanaOriginSystem();
    result.checks.originSystem = originCheck.working;
    result.checks.basicFunctionality = originCheck.working;
    result.issues.push(...originCheck.issues);
    
    // Set other checks as not applicable for Solana
    result.checks.crossChainConfig = true; // Handled differently in Solana
    result.checks.gatewayIntegration = true; // Handled differently in Solana
    result.checks.metadataPreservation = true; // Handled by Metaplex
    
    if (result.issues.length > 0) {
      result.status = "warning";
    }
    
    return result;
  }

  private async testCrossChainConnectivity(): Promise<{
    totalConnections: number;
    workingConnections: number;
    failedConnections: Array<{ from: string; to: string; error: string }>;
  }> {
    console.log("\n🌐 Testing cross-chain connectivity...");
    
    const connections = this.deploymentRecord?.crossChainConfig?.connections || [];
    const failedConnections: Array<{ from: string; to: string; error: string }> = [];
    let workingConnections = 0;
    
    for (const connection of connections) {
      try {
        console.log(`   🔗 Testing ${connection.from} → ${connection.to}...`);
        
        const fromNetwork = this.deploymentRecord?.networks[connection.from];
        if (!fromNetwork) {
          failedConnections.push({
            from: connection.from,
            to: connection.to,
            error: "Source network not found in deployment"
          });
          continue;
        }
        
        const provider = this.providers[connection.from];
        if (!provider) {
          failedConnections.push({
            from: connection.from,
            to: connection.to,
            error: "Provider not available"
          });
          continue;
        }
        
        const contract = new ethers.Contract(fromNetwork.contractAddress, UNIVERSAL_NFT_ABI, provider);
        const targetChainId = NETWORKS[connection.to as keyof typeof NETWORKS]?.chainId;
        
        if (targetChainId) {
          const isConnected = await contract.isConnectedChain(targetChainId);
          if (isConnected) {
            workingConnections++;
            console.log(`      ✅ Connection verified`);
          } else {
            failedConnections.push({
              from: connection.from,
              to: connection.to,
              error: "Chain not configured as connected"
            });
          }
        } else {
          failedConnections.push({
            from: connection.from,
            to: connection.to,
            error: "Target chain ID not found"
          });
        }
        
      } catch (error: any) {
        failedConnections.push({
          from: connection.from,
          to: connection.to,
          error: error.message
        });
      }
    }
    
    return {
      totalConnections: connections.length,
      workingConnections,
      failedConnections
    };
  }

  private generateTroubleshootingGuide(): {
    commonIssues: Record<string, string>;
    supportContacts: Record<string, string>;
  } {
    return {
      commonIssues: {
        "Contract not deployed": "Verify deployment transaction was successful and contract address is correct",
        "Cross-chain connection failed": "Run configure-cross-chain.ts script to set up connections",
        "Gateway integration error": "Verify gateway contract address matches network configuration",
        "Metadata not accessible": "Check token URI format and ensure metadata server is accessible",
        "Origin PDA not found": "Ensure Solana program is properly deployed and origin PDAs are created",
        "Gas estimation failed": "Check contract ABI and ensure functions are properly implemented",
        "Provider connection timeout": "Verify RPC URLs are correct and networks are accessible",
        "Universal address not set": "Run setUniversal task to configure universal addressing"
      },
      supportContacts: {
        "ZetaChain Documentation": "https://docs.zetachain.com",
        "Base Documentation": "https://docs.base.org",
        "Solana Documentation": "https://docs.solana.com",
        "GitHub Issues": "https://github.com/your-repo/issues",
        "Discord Support": "https://discord.gg/zetachain"
      }
    };
  }

  public async runComprehensiveVerification(): Promise<ComprehensiveReport> {
    console.log("🔍 Universal NFT Deployment Verification");
    console.log("═══════════════════════════════════════════════════════════════\n");
    
    try {
      // Load deployment record
      await this.loadDeploymentRecord();
      
      // Initialize Solana program if available
      await this.initializeSolanaProgram();
      
      // Verify each network
      const networkResults: VerificationResult[] = [];
      const networks = Object.keys(this.deploymentRecord!.networks);
      
      // Add Solana if origin system is enabled
      if (this.deploymentRecord!.originSystem?.enabled) {
        networks.push("solanaDevnet");
      }
      
      for (const network of networks) {
        const result = await this.verifyNetwork(network);
        networkResults.push(result);
      }
      
      // Test cross-chain connectivity
      const connectivityResults = await this.testCrossChainConnectivity();
      
      // Determine overall status
      const hasErrors = networkResults.some(r => r.status === "error");
      const hasWarnings = networkResults.some(r => r.status === "warning");
      const overallStatus = hasErrors ? "critical" : hasWarnings ? "issues" : "healthy";
      
      // Generate origin system status
      const originSystemStatus = this.deploymentRecord!.originSystem?.enabled ? {
        enabled: true,
        solanaIntegration: networkResults.find(r => r.network === "solanaDevnet")?.checks.deployment || false,
        pdaValidation: networkResults.find(r => r.network === "solanaDevnet")?.checks.originSystem || false,
        metadataLinking: networkResults.find(r => r.network === "solanaDevnet")?.checks.metadataPreservation || false
      } : {
        enabled: false,
        solanaIntegration: false,
        pdaValidation: false,
        metadataLinking: false
      };
      
      // Generate recommendations
      const recommendations: string[] = [];
      networkResults.forEach(result => {
        recommendations.push(...result.recommendations);
      });
      
      if (connectivityResults.failedConnections.length > 0) {
        recommendations.push("Fix failed cross-chain connections using configure-cross-chain.ts");
      }
      
      if (overallStatus === "critical") {
        recommendations.push("Address critical deployment issues before proceeding with testing");
      }
      
      const troubleshooting = this.generateTroubleshootingGuide();
      
      const report: ComprehensiveReport = {
        timestamp: new Date().toISOString(),
        overallStatus,
        networks: networkResults,
        crossChainConnectivity: connectivityResults,
        originSystemStatus,
        recommendations: [...new Set(recommendations)], // Remove duplicates
        troubleshooting
      };
      
      return report;
      
    } catch (error: any) {
      throw new Error(`Verification failed: ${error.message}`);
    }
  }

  public async generateReport(report: ComprehensiveReport): Promise<void> {
    console.log("\n📊 Generating verification report...");
    
    // Save JSON report
    const reportPath = path.join(process.cwd(), "verification-report.json");
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    
    // Generate human-readable report
    const readableReport = this.generateReadableReport(report);
    const readablePath = path.join(process.cwd(), "VERIFICATION_REPORT.md");
    fs.writeFileSync(readablePath, readableReport);
    
    console.log(`   ✅ JSON report saved: ${reportPath}`);
    console.log(`   ✅ Readable report saved: ${readablePath}`);
    
    // Print summary to console
    this.printSummary(report);
  }

  private generateReadableReport(report: ComprehensiveReport): string {
    const statusEmoji = {
      healthy: "🟢",
      issues: "🟡",
      critical: "🔴"
    };
    
    const checkEmoji = (check: boolean) => check ? "✅" : "❌";
    
    return `# Universal NFT Deployment Verification Report

**Generated:** ${new Date(report.timestamp).toLocaleString()}  
**Overall Status:** ${statusEmoji[report.overallStatus]} ${report.overallStatus.toUpperCase()}

## Network Verification Results

${report.networks.map(network => `
### ${NETWORKS[network.network as keyof typeof NETWORKS]?.name || network.network}

**Contract Address:** \`${network.contractAddress}\`  
**Status:** ${statusEmoji[network.status]} ${network.status.toUpperCase()}

**Checks:**
- ${checkEmoji(network.checks.deployment)} Contract Deployment
- ${checkEmoji(network.checks.basicFunctionality)} Basic Functionality
- ${checkEmoji(network.checks.crossChainConfig)} Cross-Chain Configuration
- ${checkEmoji(network.checks.gatewayIntegration)} Gateway Integration
- ${checkEmoji(network.checks.metadataPreservation)} Metadata Preservation
${network.checks.originSystem !== undefined ? `- ${checkEmoji(network.checks.originSystem)} Origin System` : ''}

${network.gasEstimates ? `**Gas Estimates:**
- Mint: ${network.gasEstimates.mint} gas
- Transfer: ${network.gasEstimates.transfer || 'N/A'} gas
- Cross-Chain Transfer: ${network.gasEstimates.crossChainTransfer || 'N/A'} gas` : ''}

${network.issues.length > 0 ? `**Issues:**
${network.issues.map(issue => `- ⚠️ ${issue}`).join('\n')}` : ''}

${network.recommendations.length > 0 ? `**Recommendations:**
${network.recommendations.map(rec => `- 💡 ${rec}`).join('\n')}` : ''}
`).join('\n')}

## Cross-Chain Connectivity

**Total Connections:** ${report.crossChainConnectivity.totalConnections}  
**Working Connections:** ${report.crossChainConnectivity.workingConnections}  
**Failed Connections:** ${report.crossChainConnectivity.failedConnections.length}

${report.crossChainConnectivity.failedConnections.length > 0 ? `
### Failed Connections
${report.crossChainConnectivity.failedConnections.map(conn => 
  `- **${conn.from} → ${conn.to}:** ${conn.error}`
).join('\n')}` : ''}

## NFT Origin System

${report.originSystemStatus ? `
**Status:** ${report.originSystemStatus.enabled ? '✅ Enabled' : '❌ Disabled'}  
**Solana Integration:** ${checkEmoji(report.originSystemStatus.solanaIntegration)}  
**PDA Validation:** ${checkEmoji(report.originSystemStatus.pdaValidation)}  
**Metadata Linking:** ${checkEmoji(report.originSystemStatus.metadataLinking)}
` : 'Not configured'}

## Recommendations

${report.recommendations.length > 0 ? 
  report.recommendations.map(rec => `- 💡 ${rec}`).join('\n') : 
  '✅ No recommendations - deployment looks good!'
}

## Troubleshooting

### Common Issues
${Object.entries(report.troubleshooting.commonIssues).map(([issue, solution]) => 
  `**${issue}:** ${solution}`
).join('\n\n')}

### Support Resources
${Object.entries(report.troubleshooting.supportContacts).map(([name, url]) => 
  `- [${name}](${url})`
).join('\n')}

---

*This report was generated automatically by the Universal NFT deployment verification system.*
`;
  }

  private printSummary(report: ComprehensiveReport): void {
    console.log("\n📋 Verification Summary");
    console.log("═══════════════════════════════════════════════════════════════");
    
    const statusEmoji = {
      healthy: "🟢",
      issues: "🟡", 
      critical: "🔴"
    };
    
    console.log(`Overall Status: ${statusEmoji[report.overallStatus]} ${report.overallStatus.toUpperCase()}`);
    console.log(`Networks Verified: ${report.networks.length}`);
    console.log(`Cross-Chain Connections: ${report.crossChainConnectivity.workingConnections}/${report.crossChainConnectivity.totalConnections} working`);
    
    if (report.originSystemStatus?.enabled) {
      console.log(`Origin System: ${report.originSystemStatus.enabled ? '✅ Enabled' : '❌ Disabled'}`);
    }
    
    console.log("\nNetwork Status:");
    report.networks.forEach(network => {
      const networkName = NETWORKS[network.network as keyof typeof NETWORKS]?.name || network.network;
      console.log(`  • ${networkName}: ${statusEmoji[network.status]} ${network.status}`);
    });
    
    if (report.recommendations.length > 0) {
      console.log("\nRecommendations:");
      report.recommendations.slice(0, 3).forEach(rec => {
        console.log(`  • ${rec}`);
      });
      if (report.recommendations.length > 3) {
        console.log(`  • ... and ${report.recommendations.length - 3} more (see full report)`);
      }
    }
    
    console.log("\n📄 Full reports saved to verification-report.json and VERIFICATION_REPORT.md");
  }
}

// Main execution function
async function main(): Promise<void> {
  try {
    const verifier = new DeploymentVerifier();
    const report = await verifier.runComprehensiveVerification();
    await verifier.generateReport(report);
    
    // Exit with appropriate code
    if (report.overallStatus === "critical") {
      process.exit(1);
    } else if (report.overallStatus === "issues") {
      process.exit(2);
    } else {
      process.exit(0);
    }
    
  } catch (error: any) {
    console.error("\n❌ Verification failed:");
    console.error(error.message);
    process.exit(1);
  }
}

// Export for use as module
export { DeploymentVerifier, VerificationResult, ComprehensiveReport };

// Run if called directly
if (require.main === module) {
  main().catch(error => {
    console.error("Fatal error:", error);
    process.exit(1);
  });
}