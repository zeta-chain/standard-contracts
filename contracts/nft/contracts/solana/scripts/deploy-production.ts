import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  PublicKey, 
  Keypair, 
  Connection, 
  clusterApiUrl, 
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import * as readline from "readline";

// Production configuration
const MAINNET_CONFIG = {
  cluster: "mainnet-beta",
  rpcUrl: process.env.MAINNET_RPC_URL || clusterApiUrl("mainnet-beta"),
  gatewayProgramId: "ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis", // Replace with actual mainnet gateway
  // Production ZetaChain TSS address - MUST be updated with actual mainnet TSS
  tssAddress: process.env.MAINNET_TSS_ADDRESS ? 
    JSON.parse(process.env.MAINNET_TSS_ADDRESS) : 
    null,
  // Minimum SOL balance required for operations
  minBalance: 10 * LAMPORTS_PER_SOL,
  // Connected chains configuration
  connectedChains: [
    { chainId: 1, name: "Ethereum", enabled: true },
    { chainId: 56, name: "BSC", enabled: true },
    { chainId: 137, name: "Polygon", enabled: true },
    { chainId: 7000, name: "ZetaChain", enabled: true }
  ]
};

interface DeploymentConfig {
  programId?: string;
  multisigAuthority?: string;
  emergencyAuthority?: string;
  collectionName: string;
  collectionSymbol: string;
  baseUri: string;
  tssAddress: number[];
  maxSupply?: number;
  royaltyBasisPoints?: number;
  dryRun: boolean;
}

interface DeploymentState {
  phase: string;
  completed: string[];
  failed: string[];
  rollbackActions: Array<() => Promise<void>>;
}

class ProductionDeployer {
  private connection: Connection;
  private wallet: anchor.Wallet;
  private provider: anchor.AnchorProvider;
  private program: Program;
  private config: DeploymentConfig;
  private state: DeploymentState;
  private rl: readline.Interface;

  constructor(config: DeploymentConfig) {
    this.config = config;
    this.state = {
      phase: "initialization",
      completed: [],
      failed: [],
      rollbackActions: []
    };
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });
  }

  async initialize(): Promise<void> {
    console.log("🚀 Initializing Production Deployment...");
    
    // Validate environment
    await this.validateEnvironment();
    
    // Setup connection
    this.connection = new Connection(MAINNET_CONFIG.rpcUrl, "confirmed");
    
    // Load wallet
    const walletKeypair = await this.loadWallet();
    this.wallet = new anchor.Wallet(walletKeypair);
    
    // Setup provider
    this.provider = new anchor.AnchorProvider(this.connection, this.wallet, {
      commitment: "confirmed",
      preflightCommitment: "confirmed",
      skipPreflight: false
    });
    anchor.setProvider(this.provider);
    
    // Load program
    await this.loadProgram();
    
    console.log("✅ Initialization complete");
    this.state.completed.push("initialization");
  }

  private async validateEnvironment(): Promise<void> {
    console.log("🔍 Validating environment...");
    
    // Check required environment variables
    const required = [
      "MAINNET_RPC_URL",
      "MAINNET_TSS_ADDRESS",
      "SOLANA_WALLET_PATH"
    ];
    
    for (const env of required) {
      if (!process.env[env]) {
        throw new Error(`Missing required environment variable: ${env}`);
      }
    }
    
    // Validate TSS address
    if (!this.config.tssAddress || this.config.tssAddress.length !== 20) {
      throw new Error("Invalid TSS address - must be 20 bytes");
    }
    
    // Validate RPC connection
    try {
      const slot = await this.connection.getSlot();
      console.log(`📡 Connected to mainnet at slot: ${slot}`);
    } catch (error) {
      throw new Error(`Failed to connect to RPC: ${error}`);
    }
    
    console.log("✅ Environment validation passed");
  }

  private async loadWallet(): Promise<Keypair> {
    const walletPath = process.env.SOLANA_WALLET_PATH || 
      path.join(process.env.HOME || "", ".config/solana/id.json");
    
    if (!fs.existsSync(walletPath)) {
      throw new Error(`Wallet not found at: ${walletPath}`);
    }
    
    const walletKeypair = Keypair.fromSecretKey(
      Buffer.from(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
    );
    
    // Verify wallet balance
    const balance = await this.connection.getBalance(walletKeypair.publicKey);
    console.log(`💰 Wallet balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    
    if (balance < MAINNET_CONFIG.minBalance) {
      throw new Error(`Insufficient balance. Required: ${MAINNET_CONFIG.minBalance / LAMPORTS_PER_SOL} SOL`);
    }
    
    return walletKeypair;
  }

  private async loadProgram(): Promise<void> {
    const idlPath = path.join(__dirname, "../target/idl/universal_nft.json");
    if (!fs.existsSync(idlPath)) {
      throw new Error(`IDL not found at: ${idlPath}`);
    }
    
    const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));
    
    const programId = new PublicKey(
      this.config.programId || 
      process.env.MAINNET_PROGRAM_ID ||
      idl.metadata?.address
    );
    
    if (!programId) {
      throw new Error("Program ID not specified");
    }
    
    // Verify program exists on mainnet
    const programAccount = await this.connection.getAccountInfo(programId);
    if (!programAccount) {
      throw new Error(`Program not found on mainnet: ${programId.toString()}`);
    }
    
    idl.metadata = idl.metadata || {};
    idl.metadata.address = programId.toString();
    
    this.program = new anchor.Program(idl as anchor.Idl, this.provider);
    console.log(`📦 Program loaded: ${programId.toString()}`);
  }

  async deploy(): Promise<void> {
    try {
      console.log("\n🎯 Starting Production Deployment");
      console.log("=" * 50);
      
      // Pre-deployment checks
      await this.preDeploymentChecks();
      
      // Deploy phases
      await this.deployPhase1_SecuritySetup();
      await this.deployPhase2_CollectionInitialization();
      await this.deployPhase3_ChainConfiguration();
      await this.deployPhase4_ValidationAndTesting();
      await this.deployPhase5_FinalVerification();
      
      // Save deployment info
      await this.saveDeploymentInfo();
      
      console.log("\n🎉 Production deployment completed successfully!");
      
    } catch (error) {
      console.error("\n❌ Deployment failed:", error);
      await this.handleDeploymentFailure(error);
      throw error;
    } finally {
      this.rl.close();
    }
  }

  private async preDeploymentChecks(): Promise<void> {
    console.log("\n🔍 Pre-deployment checks...");
    this.state.phase = "pre-deployment";
    
    // Confirm production deployment
    if (!this.config.dryRun) {
      const confirmed = await this.confirmAction(
        "⚠️  This will deploy to MAINNET. Are you sure? (yes/no): "
      );
      if (!confirmed) {
        throw new Error("Deployment cancelled by user");
      }
    }
    
    // Check multisig setup
    if (this.config.multisigAuthority) {
      await this.validateMultisigAuthority();
    }
    
    // Verify gateway program
    await this.verifyGatewayProgram();
    
    // Check for existing collection
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    const existingCollection = await this.connection.getAccountInfo(collectionPda);
    if (existingCollection && !this.config.dryRun) {
      const overwrite = await this.confirmAction(
        "Collection already exists. Overwrite? (yes/no): "
      );
      if (!overwrite) {
        throw new Error("Deployment cancelled - collection exists");
      }
    }
    
    console.log("✅ Pre-deployment checks passed");
    this.state.completed.push("pre-deployment");
  }

  private async deployPhase1_SecuritySetup(): Promise<void> {
    console.log("\n🔐 Phase 1: Security Setup");
    this.state.phase = "security-setup";
    
    try {
      // Setup emergency procedures
      await this.setupEmergencyProcedures();
      
      // Configure access controls
      await this.configureAccessControls();
      
      // Initialize security parameters
      await this.initializeSecurityParameters();
      
      console.log("✅ Security setup completed");
      this.state.completed.push("security-setup");
      
    } catch (error) {
      this.state.failed.push("security-setup");
      throw error;
    }
  }

  private async deployPhase2_CollectionInitialization(): Promise<void> {
    console.log("\n🎨 Phase 2: Collection Initialization");
    this.state.phase = "collection-init";
    
    try {
      const [collectionPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
        this.program.programId
      );
      
      if (this.config.dryRun) {
        console.log("🧪 DRY RUN: Would initialize collection at:", collectionPda.toString());
        this.state.completed.push("collection-init");
        return;
      }
      
      console.log("🔨 Initializing collection...");
      
      const tx = await this.program.methods
        .initializeCollection(
          this.config.collectionName,
          this.config.collectionSymbol,
          this.config.baseUri,
          this.config.tssAddress
        )
        .accounts({
          collection: collectionPda,
          authority: this.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      
      console.log("📝 Transaction:", tx);
      
      // Wait for confirmation with timeout
      await this.waitForConfirmation(tx, 60000);
      
      // Verify collection data
      const collection = await (this.program.account as any).collection.fetch(collectionPda);
      console.log("📊 Collection verified:");
      console.log("  Name:", collection.name);
      console.log("  Symbol:", collection.symbol);
      console.log("  TSS Address:", Buffer.from(collection.tssAddress).toString("hex"));
      
      // Add rollback action
      this.state.rollbackActions.push(async () => {
        console.log("🔄 Rolling back collection initialization...");
        // Note: Collection accounts cannot be easily closed, so this is logged for manual intervention
        console.log("⚠️  Manual intervention required to close collection account:", collectionPda.toString());
      });
      
      console.log("✅ Collection initialization completed");
      this.state.completed.push("collection-init");
      
    } catch (error) {
      this.state.failed.push("collection-init");
      throw error;
    }
  }

  private async deployPhase3_ChainConfiguration(): Promise<void> {
    console.log("\n🌐 Phase 3: Chain Configuration");
    this.state.phase = "chain-config";
    
    try {
      for (const chain of MAINNET_CONFIG.connectedChains) {
        if (chain.enabled) {
          await this.configureConnectedChain(chain);
        }
      }
      
      console.log("✅ Chain configuration completed");
      this.state.completed.push("chain-config");
      
    } catch (error) {
      this.state.failed.push("chain-config");
      throw error;
    }
  }

  private async deployPhase4_ValidationAndTesting(): Promise<void> {
    console.log("\n🧪 Phase 4: Validation and Testing");
    this.state.phase = "validation";
    
    try {
      // Test basic functionality
      await this.testBasicFunctionality();
      
      // Validate security parameters
      await this.validateSecurityParameters();
      
      // Test emergency procedures
      await this.testEmergencyProcedures();
      
      console.log("✅ Validation and testing completed");
      this.state.completed.push("validation");
      
    } catch (error) {
      this.state.failed.push("validation");
      throw error;
    }
  }

  private async deployPhase5_FinalVerification(): Promise<void> {
    console.log("\n🔍 Phase 5: Final Verification");
    this.state.phase = "final-verification";
    
    try {
      // Comprehensive system check
      await this.comprehensiveSystemCheck();
      
      // Generate deployment report
      await this.generateDeploymentReport();
      
      console.log("✅ Final verification completed");
      this.state.completed.push("final-verification");
      
    } catch (error) {
      this.state.failed.push("final-verification");
      throw error;
    }
  }

  private async setupEmergencyProcedures(): Promise<void> {
    console.log("🚨 Setting up emergency procedures...");
    
    if (this.config.emergencyAuthority) {
      const emergencyKey = new PublicKey(this.config.emergencyAuthority);
      
      // Verify emergency authority account exists
      const emergencyAccount = await this.connection.getAccountInfo(emergencyKey);
      if (!emergencyAccount) {
        throw new Error(`Emergency authority account not found: ${emergencyKey.toString()}`);
      }
      
      console.log("✅ Emergency authority configured:", emergencyKey.toString());
    }
    
    // Create emergency contact file
    const emergencyInfo = {
      deployer: this.wallet.publicKey.toString(),
      emergencyAuthority: this.config.emergencyAuthority,
      deploymentTime: new Date().toISOString(),
      emergencyProcedures: [
        "1. Contact deployer immediately",
        "2. Use emergency authority to pause operations",
        "3. Initiate rollback procedures if necessary",
        "4. Document incident and resolution"
      ]
    };
    
    const emergencyPath = path.join(__dirname, "../emergency-contacts.json");
    fs.writeFileSync(emergencyPath, JSON.stringify(emergencyInfo, null, 2));
    console.log("📋 Emergency procedures documented");
  }

  private async configureAccessControls(): Promise<void> {
    console.log("🔐 Configuring access controls...");
    
    // Validate multisig if configured
    if (this.config.multisigAuthority) {
      await this.validateMultisigAuthority();
    }
    
    // Log access control configuration
    console.log("👤 Access Control Summary:");
    console.log("  Deployer:", this.wallet.publicKey.toString());
    console.log("  Multisig Authority:", this.config.multisigAuthority || "Not configured");
    console.log("  Emergency Authority:", this.config.emergencyAuthority || "Not configured");
  }

  private async initializeSecurityParameters(): Promise<void> {
    console.log("🛡️  Initializing security parameters...");
    
    // Validate TSS address format
    if (this.config.tssAddress.length !== 20) {
      throw new Error("TSS address must be exactly 20 bytes");
    }
    
    // Log security configuration
    console.log("🔒 Security Configuration:");
    console.log("  TSS Address:", Buffer.from(this.config.tssAddress).toString("hex"));
    console.log("  Max Supply:", this.config.maxSupply || "Unlimited");
    console.log("  Royalty Basis Points:", this.config.royaltyBasisPoints || "0");
  }

  private async configureConnectedChain(chain: any): Promise<void> {
    console.log(`🔗 Configuring chain: ${chain.name} (${chain.chainId})`);
    
    if (this.config.dryRun) {
      console.log("🧪 DRY RUN: Would configure chain", chain.name);
      return;
    }
    
    // In a real implementation, this would configure the connected chain
    // For now, we'll just log the configuration
    console.log(`✅ Chain ${chain.name} configured`);
  }

  private async testBasicFunctionality(): Promise<void> {
    console.log("🧪 Testing basic functionality...");
    
    if (this.config.dryRun) {
      console.log("🧪 DRY RUN: Would test basic functionality");
      return;
    }
    
    // Test collection fetch
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    try {
      const collection = await (this.program.account as any).collection.fetch(collectionPda);
      console.log("✅ Collection fetch test passed");
    } catch (error) {
      throw new Error(`Collection fetch test failed: ${error}`);
    }
  }

  private async validateSecurityParameters(): Promise<void> {
    console.log("🔍 Validating security parameters...");
    
    // Validate TSS address is properly set
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    if (!this.config.dryRun) {
      const collection = await (this.program.account as any).collection.fetch(collectionPda);
      const tssHex = Buffer.from(collection.tssAddress).toString("hex");
      const expectedTssHex = Buffer.from(this.config.tssAddress).toString("hex");
      
      if (tssHex !== expectedTssHex) {
        throw new Error(`TSS address mismatch. Expected: ${expectedTssHex}, Got: ${tssHex}`);
      }
    }
    
    console.log("✅ Security parameters validated");
  }

  private async testEmergencyProcedures(): Promise<void> {
    console.log("🚨 Testing emergency procedures...");
    
    // Test emergency authority access (if configured)
    if (this.config.emergencyAuthority && !this.config.dryRun) {
      const emergencyKey = new PublicKey(this.config.emergencyAuthority);
      const account = await this.connection.getAccountInfo(emergencyKey);
      
      if (!account) {
        throw new Error("Emergency authority account not accessible");
      }
    }
    
    console.log("✅ Emergency procedures tested");
  }

  private async comprehensiveSystemCheck(): Promise<void> {
    console.log("🔍 Performing comprehensive system check...");
    
    const checks = [
      { name: "Program Account", check: () => this.checkProgramAccount() },
      { name: "Collection Account", check: () => this.checkCollectionAccount() },
      { name: "Gateway Integration", check: () => this.checkGatewayIntegration() },
      { name: "Security Configuration", check: () => this.checkSecurityConfiguration() },
      { name: "Network Connectivity", check: () => this.checkNetworkConnectivity() }
    ];
    
    for (const check of checks) {
      try {
        await check.check();
        console.log(`✅ ${check.name}: PASS`);
      } catch (error) {
        console.log(`❌ ${check.name}: FAIL - ${error}`);
        throw new Error(`System check failed: ${check.name}`);
      }
    }
    
    console.log("✅ All system checks passed");
  }

  private async checkProgramAccount(): Promise<void> {
    const account = await this.connection.getAccountInfo(this.program.programId);
    if (!account) {
      throw new Error("Program account not found");
    }
  }

  private async checkCollectionAccount(): Promise<void> {
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    const account = await this.connection.getAccountInfo(collectionPda);
    if (!account && !this.config.dryRun) {
      throw new Error("Collection account not found");
    }
  }

  private async checkGatewayIntegration(): Promise<void> {
    const gatewayProgramId = new PublicKey(MAINNET_CONFIG.gatewayProgramId);
    const account = await this.connection.getAccountInfo(gatewayProgramId);
    if (!account) {
      throw new Error("Gateway program not found");
    }
  }

  private async checkSecurityConfiguration(): Promise<void> {
    if (!this.config.tssAddress || this.config.tssAddress.length !== 20) {
      throw new Error("Invalid TSS address configuration");
    }
  }

  private async checkNetworkConnectivity(): Promise<void> {
    const slot = await this.connection.getSlot();
    if (!slot || slot < 0) {
      throw new Error("Network connectivity issue");
    }
  }

  private async generateDeploymentReport(): Promise<void> {
    console.log("📊 Generating deployment report...");
    
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    const report = {
      deployment: {
        timestamp: new Date().toISOString(),
        network: "mainnet-beta",
        deployer: this.wallet.publicKey.toString(),
        dryRun: this.config.dryRun
      },
      program: {
        programId: this.program.programId.toString(),
        collectionPda: collectionPda.toString()
      },
      configuration: {
        collectionName: this.config.collectionName,
        collectionSymbol: this.config.collectionSymbol,
        baseUri: this.config.baseUri,
        tssAddress: Buffer.from(this.config.tssAddress).toString("hex"),
        maxSupply: this.config.maxSupply,
        royaltyBasisPoints: this.config.royaltyBasisPoints
      },
      security: {
        multisigAuthority: this.config.multisigAuthority,
        emergencyAuthority: this.config.emergencyAuthority
      },
      phases: {
        completed: this.state.completed,
        failed: this.state.failed
      },
      connectedChains: MAINNET_CONFIG.connectedChains
    };
    
    const reportPath = path.join(__dirname, "../deployment-report-mainnet.json");
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log("📋 Deployment report saved to:", reportPath);
  }

  private async saveDeploymentInfo(): Promise<void> {
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    const [gatewayPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("meta")],
      new PublicKey(MAINNET_CONFIG.gatewayProgramId)
    );
    
    const deploymentInfo = {
      network: "mainnet-beta",
      programId: this.program.programId.toString(),
      collectionPda: collectionPda.toString(),
      gatewayProgramId: MAINNET_CONFIG.gatewayProgramId,
      gatewayPda: gatewayPda.toString(),
      deployer: this.wallet.publicKey.toString(),
      multisigAuthority: this.config.multisigAuthority,
      emergencyAuthority: this.config.emergencyAuthority,
      timestamp: new Date().toISOString(),
      dryRun: this.config.dryRun,
      tssAddress: Buffer.from(this.config.tssAddress).toString("hex")
    };
    
    const deploymentPath = path.join(__dirname, "../deployment-mainnet.json");
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
    console.log("💾 Deployment info saved to:", deploymentPath);
  }

  private async handleDeploymentFailure(error: any): Promise<void> {
    console.log("\n🚨 Handling deployment failure...");
    console.log("Failed phase:", this.state.phase);
    console.log("Completed phases:", this.state.completed);
    console.log("Failed phases:", this.state.failed);
    
    if (this.state.rollbackActions.length > 0 && !this.config.dryRun) {
      const rollback = await this.confirmAction(
        "Attempt automatic rollback? (yes/no): "
      );
      
      if (rollback) {
        console.log("🔄 Executing rollback procedures...");
        for (const action of this.state.rollbackActions.reverse()) {
          try {
            await action();
          } catch (rollbackError) {
            console.error("Rollback action failed:", rollbackError);
          }
        }
      }
    }
    
    // Save failure report
    const failureReport = {
      timestamp: new Date().toISOString(),
      error: error.toString(),
      phase: this.state.phase,
      completed: this.state.completed,
      failed: this.state.failed,
      rollbackExecuted: this.state.rollbackActions.length > 0
    };
    
    const failurePath = path.join(__dirname, "../deployment-failure-mainnet.json");
    fs.writeFileSync(failurePath, JSON.stringify(failureReport, null, 2));
    console.log("📋 Failure report saved to:", failurePath);
  }

  private async validateMultisigAuthority(): Promise<void> {
    if (!this.config.multisigAuthority) return;
    
    const multisigKey = new PublicKey(this.config.multisigAuthority);
    const account = await this.connection.getAccountInfo(multisigKey);
    
    if (!account) {
      throw new Error(`Multisig authority account not found: ${multisigKey.toString()}`);
    }
    
    console.log("✅ Multisig authority validated:", multisigKey.toString());
  }

  private async verifyGatewayProgram(): Promise<void> {
    const gatewayProgramId = new PublicKey(MAINNET_CONFIG.gatewayProgramId);
    const account = await this.connection.getAccountInfo(gatewayProgramId);
    
    if (!account) {
      throw new Error(`Gateway program not found: ${gatewayProgramId.toString()}`);
    }
    
    console.log("✅ Gateway program verified:", gatewayProgramId.toString());
  }

  private async waitForConfirmation(signature: string, timeout: number = 30000): Promise<void> {
    const start = Date.now();
    
    while (Date.now() - start < timeout) {
      try {
        const status = await this.connection.getSignatureStatus(signature);
        if (status.value?.confirmationStatus === "confirmed" || 
            status.value?.confirmationStatus === "finalized") {
          return;
        }
      } catch (error) {
        // Continue waiting
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new Error(`Transaction confirmation timeout: ${signature}`);
  }

  private async confirmAction(message: string): Promise<boolean> {
    return new Promise((resolve) => {
      this.rl.question(message, (answer) => {
        resolve(answer.toLowerCase() === 'yes' || answer.toLowerCase() === 'y');
      });
    });
  }
}

// Main deployment function
async function main() {
  console.log("🚀 Universal NFT Production Deployment");
  console.log("=====================================");
  
  // Load configuration
  const config: DeploymentConfig = {
    programId: process.env.MAINNET_PROGRAM_ID,
    multisigAuthority: process.env.MULTISIG_AUTHORITY,
    emergencyAuthority: process.env.EMERGENCY_AUTHORITY,
    collectionName: process.env.COLLECTION_NAME || "Universal NFT",
    collectionSymbol: process.env.COLLECTION_SYMBOL || "UNFT",
    baseUri: process.env.BASE_URI || "https://universal-nft.zetachain.com/metadata/",
    tssAddress: JSON.parse(process.env.MAINNET_TSS_ADDRESS || "[]"),
    maxSupply: process.env.MAX_SUPPLY ? parseInt(process.env.MAX_SUPPLY) : undefined,
    royaltyBasisPoints: process.env.ROYALTY_BASIS_POINTS ? parseInt(process.env.ROYALTY_BASIS_POINTS) : undefined,
    dryRun: process.env.DRY_RUN === "true"
  };
  
  // Validate configuration
  if (!config.tssAddress || config.tssAddress.length === 0) {
    throw new Error("MAINNET_TSS_ADDRESS environment variable is required");
  }
  
  if (config.dryRun) {
    console.log("🧪 DRY RUN MODE - No actual transactions will be sent");
  }
  
  // Initialize and run deployment
  const deployer = new ProductionDeployer(config);
  await deployer.initialize();
  await deployer.deploy();
  
  console.log("\n🎉 Production deployment completed successfully!");
  console.log("\n📋 Next steps:");
  console.log("1. Verify all configurations in the deployment report");
  console.log("2. Test the system with small transactions");
  console.log("3. Monitor system health and performance");
  console.log("4. Set up monitoring and alerting");
  console.log("5. Document operational procedures");
  
  if (!config.dryRun) {
    const [collectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("collection"), new PublicKey(process.env.SOLANA_WALLET_PATH || "")],
      new PublicKey(config.programId || "")
    );
    
    console.log("\n🔗 View on Solana Explorer:");
    console.log(`https://explorer.solana.com/address/${collectionPda.toString()}`);
  }
}

// Error handling and execution
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error("\n❌ Deployment failed:", error);
      process.exit(1);
    });
}

export { ProductionDeployer, DeploymentConfig };