import * as fs from "fs";
import * as path from "path";
import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Program, Wallet, BN } from "@coral-xyz/anchor";
import { 
  UniversalNftClient, 
  Network, 
  UniversalNftUtils,
  UniversalNftError,
  TransactionError
} from "../sdk/client";

// Deployment configuration
interface DeploymentConfig {
  network: Network;
  endpoint: string;
  programId?: PublicKey;
  collectionName: string;
  collectionSymbol: string;
  collectionUri: string;
  tssAddress: string;
  supportedChains: {
    ethereum: { chainId: number; contractAddress: string; rpcUrl: string };
    bsc: { chainId: number; contractAddress: string; rpcUrl: string };
    base: { chainId: number; contractAddress: string; rpcUrl: string };
  };
}

// Deployment result interface
interface DeploymentResult {
  success: boolean;
  programId: string;
  collection: {
    address: string;
    name: string;
    symbol: string;
    uri: string;
    authority: string;
  };
  connectedContracts: Array<{
    chainId: number;
    chainName: string;
    contractAddress: string;
    connectedPda: string;
  }>;
  testResults: {
    mintTest: {
      success: boolean;
      nftMint?: string;
      tokenAccount?: string;
      signature?: string;
      error?: string;
    };
    transferTest: {
      success: boolean;
      signature?: string;
      destinationChain?: number;
      error?: string;
    };
  };
  deploymentTime: string;
  networkInfo: {
    endpoint: string;
    commitment: string;
    slot: number;
  };
}

// Devnet configuration
const DEVNET_CONFIG: DeploymentConfig = {
  network: Network.DEVNET,
  endpoint: "https://api.devnet.solana.com",
  collectionName: "Universal NFT Devnet",
  collectionSymbol: "UNFT-DEV",
  collectionUri: "https://raw.githubusercontent.com/zetachain/universal-nft/main/metadata/collection.json",
  tssAddress: "0x70e967acFcC17c3941E87562161406d41676FD83", // ZetaChain devnet TSS
  supportedChains: {
    ethereum: {
      chainId: 11155111, // Sepolia
      contractAddress: "0x13A0c5930C028511Dc02665E7285134B6d11A5f4", // Example Universal NFT contract
      rpcUrl: "https://sepolia.infura.io/v3/YOUR_INFURA_KEY"
    },
    bsc: {
      chainId: 97, // BSC Testnet
      contractAddress: "0x13A0c5930C028511Dc02665E7285134B6d11A5f4", // Example Universal NFT contract
      rpcUrl: "https://data-seed-prebsc-1-s1.binance.org:8545"
    },
    base: {
      chainId: 84532, // Base Sepolia
      contractAddress: "0x13A0c5930C028511Dc02665E7285134B6d11A5f4", // Example Universal NFT contract
      rpcUrl: "https://sepolia.base.org"
    }
  }
};

class DevnetDeployer {
  private client: UniversalNftClient | null = null;
  private authority: Keypair;
  private config: DeploymentConfig;
  private deploymentResult: Partial<DeploymentResult> = {};

  constructor(config: DeploymentConfig = DEVNET_CONFIG) {
    this.config = config;
    this.authority = this.loadOrCreateAuthority();
  }

  /**
   * Load authority keypair from file or create new one
   */
  private loadOrCreateAuthority(): Keypair {
    const authorityPath = path.join(__dirname, "../.keys/devnet-authority.json");
    
    try {
      if (fs.existsSync(authorityPath)) {
        const authorityData = JSON.parse(fs.readFileSync(authorityPath, "utf-8"));
        console.log("📋 Loaded existing authority keypair");
        return Keypair.fromSecretKey(new Uint8Array(authorityData));
      }
    } catch (error) {
      console.log("⚠️  Failed to load existing authority, creating new one");
    }

    // Create new authority
    const authority = Keypair.generate();
    
    // Ensure .keys directory exists
    const keysDir = path.dirname(authorityPath);
    if (!fs.existsSync(keysDir)) {
      fs.mkdirSync(keysDir, { recursive: true });
    }

    // Save authority keypair
    fs.writeFileSync(
      authorityPath,
      JSON.stringify(Array.from(authority.secretKey)),
      "utf-8"
    );

    console.log("🔑 Created new authority keypair");
    return authority;
  }

  /**
   * Initialize the Universal NFT client
   */
  private async initializeClient(): Promise<void> {
    console.log("🚀 Initializing Universal NFT client...");

    try {
      this.client = await UniversalNftClient.create(
        {
          network: this.config.network,
          endpoint: this.config.endpoint,
          commitment: "confirmed",
          confirmOptions: { commitment: "confirmed" }
        },
        {
          keypair: this.authority
        }
      );

      console.log(`   ✅ Client initialized for ${this.config.network}`);
      console.log(`   📡 Endpoint: ${this.config.endpoint}`);
      console.log(`   🔑 Authority: ${this.authority.publicKey.toBase58()}`);
      console.log(`   📋 Program ID: ${this.client.program.programId.toBase58()}`);

    } catch (error) {
      throw new UniversalNftError(`Failed to initialize client: ${error}`);
    }
  }

  /**
   * Check and fund authority account
   */
  private async ensureFunding(): Promise<void> {
    console.log("💰 Checking authority funding...");

    if (!this.client) {
      throw new UniversalNftError("Client not initialized");
    }

    const balance = await this.client.connection.getBalance(this.authority.publicKey);
    const requiredBalance = 0.5 * LAMPORTS_PER_SOL; // 0.5 SOL minimum

    console.log(`   💳 Current balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    if (balance < requiredBalance) {
      console.log("   💸 Requesting airdrop...");
      
      try {
        const airdropSignature = await this.client.connection.requestAirdrop(
          this.authority.publicKey,
          2 * LAMPORTS_PER_SOL
        );

        await this.client.waitForConfirmation(airdropSignature);
        
        const newBalance = await this.client.connection.getBalance(this.authority.publicKey);
        console.log(`   ✅ Airdrop successful! New balance: ${newBalance / LAMPORTS_PER_SOL} SOL`);
      } catch (error) {
        throw new UniversalNftError(`Airdrop failed: ${error}`);
      }
    } else {
      console.log("   ✅ Authority has sufficient funding");
    }
  }

  /**
   * Deploy and initialize the collection
   */
  private async deployCollection(): Promise<{ collection: PublicKey; signature: string }> {
    console.log("🏗️  Deploying Universal NFT collection...");

    if (!this.client) {
      throw new UniversalNftError("Client not initialized");
    }

    // Convert TSS address to bytes
    const tssAddressBytes = UniversalNftUtils.ethAddressToBytes(this.config.tssAddress);

    try {
      const result = await this.client.initializeCollection(
        this.config.collectionName,
        this.config.collectionSymbol,
        this.config.collectionUri,
        tssAddressBytes
      );

      console.log(`   ✅ Collection deployed successfully!`);
      console.log(`   📄 Collection PDA: ${result.collection.toBase58()}`);
      console.log(`   📝 Transaction: ${result.signature}`);
      console.log(`   🏷️  Name: ${this.config.collectionName}`);
      console.log(`   🔤 Symbol: ${this.config.collectionSymbol}`);
      console.log(`   🔗 URI: ${this.config.collectionUri}`);
      console.log(`   🔐 TSS Address: ${this.config.tssAddress}`);

      // Verify collection was created correctly
      const collectionData = await this.client.getCollection(result.collection);
      console.log(`   ✅ Collection verification passed`);
      console.log(`   📊 Next Token ID: ${collectionData.totalSupply.toNumber()}`);

      return result;

    } catch (error) {
      throw new TransactionError(`Collection deployment failed: ${error}`);
    }
  }

  /**
   * Set up connected contracts for supported chains
   */
  private async setupConnectedContracts(collection: PublicKey): Promise<Array<{
    chainId: number;
    chainName: string;
    contractAddress: string;
    connectedPda: string;
  }>> {
    console.log("🌐 Setting up connected contracts...");

    if (!this.client) {
      throw new UniversalNftError("Client not initialized");
    }

    const connectedContracts = [];
    const chains = [
      { name: "ethereum", config: this.config.supportedChains.ethereum },
      { name: "bsc", config: this.config.supportedChains.bsc },
      { name: "base", config: this.config.supportedChains.base }
    ];

    for (const chain of chains) {
      try {
        console.log(`   🔗 Setting up ${chain.name} connection...`);

        const chainIdBytes = UniversalNftUtils.chainIdToBytes(chain.config.chainId);
        const contractAddressBytes = UniversalNftUtils.ethAddressToBytes(chain.config.contractAddress);

        const signature = await this.client.setConnected(
          collection,
          chainIdBytes,
          contractAddressBytes
        );

        // Derive connected PDA for verification
        const [connectedPda] = this.client.deriveConnectedPda(collection, chainIdBytes);

        // Verify connected contract was set
        const connectedData = await this.client.getConnected(connectedPda);
        
        console.log(`   ✅ ${chain.name} connected successfully`);
        console.log(`      Chain ID: ${chain.config.chainId}`);
        console.log(`      Contract: ${chain.config.contractAddress}`);
        console.log(`      Connected PDA: ${connectedPda.toBase58()}`);
        console.log(`      Transaction: ${signature}`);

        connectedContracts.push({
          chainId: chain.config.chainId,
          chainName: chain.name,
          contractAddress: chain.config.contractAddress,
          connectedPda: connectedPda.toBase58()
        });

      } catch (error) {
        console.error(`   ❌ Failed to connect ${chain.name}: ${error}`);
        throw new TransactionError(`Failed to setup ${chain.name} connection: ${error}`);
      }
    }

    console.log(`   ✅ All connected contracts configured (${connectedContracts.length})`);
    return connectedContracts;
  }

  /**
   * Test basic functionality - mint NFT
   */
  private async testMintNft(collection: PublicKey): Promise<{
    success: boolean;
    nftMint?: string;
    tokenAccount?: string;
    signature?: string;
    error?: string;
  }> {
    console.log("🎨 Testing NFT minting functionality...");

    if (!this.client) {
      throw new UniversalNftError("Client not initialized");
    }

    try {
      const testNftName = "Test Universal NFT #1";
      const testNftSymbol = "TUNFT1";
      const testNftUri = "https://raw.githubusercontent.com/zetachain/universal-nft/main/metadata/test-nft-1.json";

      const mintResult = await this.client.mintNft(
        collection,
        testNftName,
        testNftSymbol,
        testNftUri,
        this.authority.publicKey
      );

      console.log(`   ✅ NFT minted successfully!`);
      console.log(`   🎨 NFT Mint: ${mintResult.mint.toBase58()}`);
      console.log(`   🎯 Token Account: ${mintResult.tokenAccount.toBase58()}`);
      console.log(`   🔗 Origin PDA: ${mintResult.nftOrigin.toBase58()}`);
      console.log(`   📄 Metadata: ${mintResult.metadata.toBase58()}`);
      console.log(`   📝 Transaction: ${mintResult.signature}`);

      // Verify NFT was minted correctly
      const tokenBalance = await this.client.connection.getTokenAccountBalance(mintResult.tokenAccount);
      if (tokenBalance.value.amount !== "1") {
        throw new Error("NFT token balance is not 1");
      }

      // Verify origin data
      const originData = await this.client.getNftOrigin(mintResult.nftOrigin);
      console.log(`   ✅ Origin verification passed`);
      console.log(`   🆔 Token ID: ${originData.tokenId.toNumber()}`);

      return {
        success: true,
        nftMint: mintResult.mint.toBase58(),
        tokenAccount: mintResult.tokenAccount.toBase58(),
        signature: mintResult.signature
      };

    } catch (error) {
      console.error(`   ❌ NFT minting test failed: ${error}`);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  /**
   * Test cross-chain transfer functionality
   */
  private async testCrossChainTransfer(
    collection: PublicKey,
    nftMint: string,
    tokenAccount: string
  ): Promise<{
    success: boolean;
    signature?: string;
    destinationChain?: number;
    error?: string;
  }> {
    console.log("🌉 Testing cross-chain transfer functionality...");

    if (!this.client) {
      throw new UniversalNftError("Client not initialized");
    }

    try {
      const destinationChainId = this.config.supportedChains.ethereum.chainId;
      const recipientAddress = "0x742d35Cc6634C0532925a3b8D0C9C0E3C5d5c8eE"; // Test recipient
      const recipientBytes = UniversalNftUtils.ethAddressToBytes(recipientAddress);

      const transferResult = await this.client.transferCrossChain(
        collection,
        new PublicKey(nftMint),
        destinationChainId,
        recipientBytes
      );

      console.log(`   ✅ Cross-chain transfer initiated successfully!`);
      console.log(`   🎯 Destination: Ethereum Sepolia (${destinationChainId})`);
      console.log(`   📧 Recipient: ${recipientAddress}`);
      console.log(`   🆔 Token ID: ${transferResult.tokenId.toNumber()}`);
      console.log(`   📝 Transaction: ${transferResult.signature}`);

      // Verify NFT was burned locally
      const finalBalance = await this.client.connection.getTokenAccountBalance(
        new PublicKey(tokenAccount)
      );
      if (finalBalance.value.amount !== "0") {
        throw new Error("NFT was not burned after transfer");
      }

      console.log(`   ✅ NFT burned locally - transfer verified`);

      return {
        success: true,
        signature: transferResult.signature,
        destinationChain: destinationChainId
      };

    } catch (error) {
      console.error(`   ❌ Cross-chain transfer test failed: ${error}`);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  /**
   * Save deployment configuration and results
   */
  private async saveDeploymentResults(result: DeploymentResult): Promise<void> {
    console.log("💾 Saving deployment results...");

    const outputDir = path.join(__dirname, "../deployments");
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
    const outputFile = path.join(outputDir, `devnet-deployment-${timestamp}.json`);
    const latestFile = path.join(outputDir, "devnet-latest.json");

    try {
      // Save timestamped deployment
      fs.writeFileSync(outputFile, JSON.stringify(result, null, 2), "utf-8");
      
      // Save as latest deployment
      fs.writeFileSync(latestFile, JSON.stringify(result, null, 2), "utf-8");

      console.log(`   ✅ Deployment results saved:`);
      console.log(`   📄 Timestamped: ${outputFile}`);
      console.log(`   📄 Latest: ${latestFile}`);

      // Also save a simple config file for integration
      const integrationConfig = {
        network: result.networkInfo.endpoint,
        programId: result.programId,
        collection: result.collection.address,
        authority: result.collection.authority,
        connectedChains: result.connectedContracts.reduce((acc, chain) => {
          acc[chain.chainName] = {
            chainId: chain.chainId,
            contractAddress: chain.contractAddress
          };
          return acc;
        }, {} as Record<string, any>)
      };

      const configFile = path.join(outputDir, "devnet-config.json");
      fs.writeFileSync(configFile, JSON.stringify(integrationConfig, null, 2), "utf-8");
      console.log(`   📄 Integration config: ${configFile}`);

    } catch (error) {
      console.error(`   ⚠️  Failed to save deployment results: ${error}`);
    }
  }

  /**
   * Rollback deployment in case of failure
   */
  private async rollbackDeployment(error: Error): Promise<void> {
    console.log("🔄 Attempting deployment rollback...");

    try {
      // Log the error for debugging
      const errorLog = {
        timestamp: new Date().toISOString(),
        error: error.message,
        stack: error.stack,
        deploymentState: this.deploymentResult
      };

      const errorDir = path.join(__dirname, "../deployments/errors");
      if (!fs.existsSync(errorDir)) {
        fs.mkdirSync(errorDir, { recursive: true });
      }

      const errorFile = path.join(errorDir, `devnet-error-${Date.now()}.json`);
      fs.writeFileSync(errorFile, JSON.stringify(errorLog, null, 2), "utf-8");

      console.log(`   📄 Error logged: ${errorFile}`);
      console.log(`   ⚠️  Manual cleanup may be required for any created accounts`);

    } catch (rollbackError) {
      console.error(`   ❌ Rollback failed: ${rollbackError}`);
    }
  }

  /**
   * Main deployment function
   */
  async deploy(): Promise<DeploymentResult> {
    const startTime = Date.now();
    console.log("🚀 Starting Universal NFT Devnet Deployment");
    console.log("=" .repeat(60));

    try {
      // Step 1: Initialize client
      await this.initializeClient();
      
      // Step 2: Ensure funding
      await this.ensureFunding();

      // Step 3: Deploy collection
      const { collection, signature: collectionSignature } = await this.deployCollection();

      // Step 4: Setup connected contracts
      const connectedContracts = await this.setupConnectedContracts(collection);

      // Step 5: Test minting
      const mintTest = await this.testMintNft(collection);

      // Step 6: Test cross-chain transfer (if mint succeeded)
      let transferTest = { success: false, error: "Mint test failed" };
      if (mintTest.success && mintTest.nftMint && mintTest.tokenAccount) {
        transferTest = await this.testCrossChainTransfer(
          collection,
          mintTest.nftMint,
          mintTest.tokenAccount
        );
      }

      // Get network info
      const slot = await this.client!.getCurrentSlot();

      // Compile results
      const result: DeploymentResult = {
        success: true,
        programId: this.client!.program.programId.toBase58(),
        collection: {
          address: collection.toBase58(),
          name: this.config.collectionName,
          symbol: this.config.collectionSymbol,
          uri: this.config.collectionUri,
          authority: this.authority.publicKey.toBase58()
        },
        connectedContracts,
        testResults: {
          mintTest,
          transferTest
        },
        deploymentTime: new Date().toISOString(),
        networkInfo: {
          endpoint: this.config.endpoint,
          commitment: "confirmed",
          slot
        }
      };

      // Save results
      await this.saveDeploymentResults(result);

      // Success summary
      const duration = Date.now() - startTime;
      console.log("\n🎉 Deployment Completed Successfully!");
      console.log("=" .repeat(60));
      console.log(`⏱️  Total time: ${duration}ms`);
      console.log(`📋 Program ID: ${result.programId}`);
      console.log(`🏷️  Collection: ${result.collection.address}`);
      console.log(`🔑 Authority: ${result.collection.authority}`);
      console.log(`🌐 Connected chains: ${connectedContracts.length}`);
      console.log(`✅ Mint test: ${mintTest.success ? "PASSED" : "FAILED"}`);
      console.log(`✅ Transfer test: ${transferTest.success ? "PASSED" : "FAILED"}`);
      console.log("\n📋 Next Steps:");
      console.log("1. Update your frontend/backend with the new addresses");
      console.log("2. Test cross-chain integration with ZetaChain gateway");
      console.log("3. Run end-to-end tests with real cross-chain messages");
      console.log("4. Monitor the deployment using the health check system");

      return result;

    } catch (error) {
      console.error("\n❌ Deployment Failed!");
      console.error("=" .repeat(60));
      console.error(`Error: ${error instanceof Error ? error.message : String(error)}`);

      await this.rollbackDeployment(error instanceof Error ? error : new Error(String(error)));

      const result: DeploymentResult = {
        success: false,
        programId: this.client?.program.programId.toBase58() || "unknown",
        collection: {
          address: "failed",
          name: this.config.collectionName,
          symbol: this.config.collectionSymbol,
          uri: this.config.collectionUri,
          authority: this.authority.publicKey.toBase58()
        },
        connectedContracts: [],
        testResults: {
          mintTest: { success: false, error: "Deployment failed" },
          transferTest: { success: false, error: "Deployment failed" }
        },
        deploymentTime: new Date().toISOString(),
        networkInfo: {
          endpoint: this.config.endpoint,
          commitment: "confirmed",
          slot: 0
        }
      };

      throw error;
    } finally {
      // Cleanup
      if (this.client) {
        this.client.dispose();
      }
    }
  }
}

/**
 * Main deployment script execution
 */
async function main() {
  try {
    // Parse command line arguments
    const args = process.argv.slice(2);
    const customConfig = args.find(arg => arg.startsWith("--config="));
    
    let config = DEVNET_CONFIG;
    if (customConfig) {
      const configPath = customConfig.split("=")[1];
      if (fs.existsSync(configPath)) {
        const customConfigData = JSON.parse(fs.readFileSync(configPath, "utf-8"));
        config = { ...DEVNET_CONFIG, ...customConfigData };
        console.log(`📋 Using custom config: ${configPath}`);
      }
    }

    // Create deployer and run deployment
    const deployer = new DevnetDeployer(config);
    const result = await deployer.deploy();

    // Exit with success
    process.exit(0);

  } catch (error) {
    console.error("\n💥 Deployment script failed:");
    console.error(error instanceof Error ? error.message : String(error));
    
    if (error instanceof Error && error.stack) {
      console.error("\nStack trace:");
      console.error(error.stack);
    }

    process.exit(1);
  }
}

// Export for programmatic use
export { DevnetDeployer, DEVNET_CONFIG };
export type { DeploymentConfig, DeploymentResult };

// Run if called directly
if (require.main === module) {
  main();
}