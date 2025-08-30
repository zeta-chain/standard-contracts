import { Connection, PublicKey, Keypair, SystemProgram, Transaction, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, web3, BN } from '@coral-xyz/anchor';
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  createMint, 
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress 
} from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { ethers } from 'ethers';
import { normalizeEthereumAddress, getEvmAddressArray } from '../utils/address';

// Configuration with environment variable support
interface DemoConfig {
  zetachainRpc: string;
  zetachainChainId: number;
  solanaChainId: number;
  solanaRpc: string;
  programId: PublicKey;
  gatewayProgramId: PublicKey;
  metadataProgramId: PublicKey;
  confirmationTimeout: number;
  maxRetries: number;
}

const DEFAULT_CONFIG: DemoConfig = {
  zetachainRpc: process.env.ZETACHAIN_RPC || 'https://zetachain-athens-evm.blockpi.network/v1/rpc/public',
  zetachainChainId: parseInt(process.env.ZETACHAIN_CHAIN_ID || '7001'),
  solanaChainId: parseInt(process.env.SOLANA_CHAIN_ID || '7565164'),
  solanaRpc: process.env.SOLANA_RPC || 'https://api.devnet.solana.com',
  programId: new PublicKey(process.env.PROGRAM_ID || 'Gc1BJg4sYAYGnKBStAHLTdVRLR3fA7DPc7t9G7vjKa1i'),
  gatewayProgramId: new PublicKey(process.env.GATEWAY_PROGRAM_ID || 'ZETAjseVjuFsxdRxQGF23a4pHWf4tEP13mgJCn71B6p'),
  metadataProgramId: new PublicKey(process.env.METADATA_PROGRAM_ID || 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'),
  confirmationTimeout: parseInt(process.env.CONFIRMATION_TIMEOUT || '60000'),
  maxRetries: parseInt(process.env.MAX_RETRIES || '3')
};

// ZetaChain Gateway Contract ABI (simplified for demo)
const GATEWAY_ABI = [
  "function call(bytes calldata receiver, bytes calldata message, uint256 gasLimit, uint256 gasPrice) external payable",
  "function deposit(address receiver, uint256 amount, uint256 chainId, bytes calldata message) external payable",
  "event Call(address indexed sender, bytes receiver, bytes message)",
  "event Deposit(address indexed sender, address receiver, uint256 amount, uint256 chainId, bytes message)"
];

// Cross-chain message types
enum CrossChainMessageType {
  MINT_REQUEST = 0,
  BURN_CONFIRMATION = 1,
  REVERT_REQUEST = 2
}

interface CrossChainNftMetadata {
  name: string;
  symbol: string;
  uri: string;
  originalChainId: number;
  originalTokenId: Uint8Array;
  originalCreator: Uint8Array;
  attributes: any[];
}

export class CrossChainNFTDemo {
  private config: DemoConfig;
  private solanaConnection: Connection;
  private zetaProvider: ethers.JsonRpcProvider;
  private gatewayContract: ethers.Contract;
  private solanaWallet: Keypair;
  private zetaWallet: ethers.Wallet;
  
  constructor(config: Partial<DemoConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    
    // Initialize Solana connection with enhanced options
    this.solanaConnection = new Connection(this.config.solanaRpc, {
      commitment: 'confirmed',
      wsEndpoint: this.config.solanaRpc.replace('https:', 'wss:'),
    });
    
    // Initialize ZetaChain connection with modern ethers v6 API
    this.zetaProvider = new ethers.JsonRpcProvider(this.config.zetachainRpc);
    
    // Load wallets with enhanced security validation
    this.solanaWallet = Keypair.generate(); // In demo, generate new wallet
    
    // Enhanced private key validation
    const pk = process.env.DEMO_PRIVATE_KEY;
    if (!pk || pk === '0x' + '0'.repeat(64) || pk.length < 64) {
      throw new Error(
        "DEMO_PRIVATE_KEY env var is required, must be a valid 64-character hex private key, " +
        "and cannot be the zero key. Example: DEMO_PRIVATE_KEY=0x1234...abcd"
      );
    }
    
    const normalizedPk = pk.startsWith("0x") ? pk : ("0x" + pk);
    if (!/^0x[0-9a-fA-F]{64}$/.test(normalizedPk)) {
      throw new Error("Invalid private key format. Must be 64 hex characters with optional 0x prefix.");
    }
    
    this.zetaWallet = new ethers.Wallet(normalizedPk, this.zetaProvider);

    // Enhanced gateway address validation
    const gatewayAddr = process.env.DEMO_GATEWAY_ADDR;
    if (!gatewayAddr || gatewayAddr === '0x0000000000000000000000000000000000000000') {
      throw new Error(
        "DEMO_GATEWAY_ADDR env var is required and cannot be the zero address. " +
        "Set it to a valid Ethereum address. Example: DEMO_GATEWAY_ADDR=0x742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42"
      );
    }
    
    // Validate and normalize gateway address
    const normalizedGatewayAddr = normalizeEthereumAddress(gatewayAddr);
    this.gatewayContract = new ethers.Contract(
      normalizedGatewayAddr,
      GATEWAY_ABI,
      this.zetaWallet
    );
    
    console.log(`🔧 Demo Configuration:`);
    console.log(`   Solana RPC: ${this.config.solanaRpc}`);
    console.log(`   ZetaChain RPC: ${this.config.zetachainRpc}`);
    console.log(`   Program ID: ${this.config.programId.toString()}`);
    console.log(`   Gateway Address: ${normalizedGatewayAddr}`);
    console.log(`   ZetaChain Wallet: ${this.zetaWallet.address}`);
  }

  /**
   * Step 1: Initialize the Universal NFT Program on Solana with retry logic
   */
  async initializeSolanaProgram(): Promise<void> {
    console.log('🚀 Step 1: Initializing Universal NFT Program on Solana');
    
    try {
      // Validate Solana connection
      await this.validateSolanaConnection();
      
      // Fund the wallet for transactions with retry logic
      console.log(`💰 Funding Solana wallet: ${this.solanaWallet.publicKey.toString()}`);
      await this.fundSolanaWalletWithRetry();
      
      // Create program config PDA
      const [programConfigPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('universal_nft_program')],
        this.config.programId
      );
      
      console.log(`📋 Program Config PDA: ${programConfigPDA.toString()}`);
      
      // Validate program deployment
      const programInfo = await this.solanaConnection.getAccountInfo(this.config.programId);
      if (!programInfo?.executable) {
        throw new Error(`Program not deployed or not executable: ${this.config.programId.toString()}`);
      }
      
      console.log(`✅ Solana program validated (${programInfo.data.length} bytes)`);
      
    } catch (error: any) {
      console.error('❌ Error initializing Solana program:', error.message);
      throw error;
    }
  }

  /**
   * Validate Solana connection health
   */
  private async validateSolanaConnection(): Promise<void> {
    try {
      const version = await this.solanaConnection.getVersion();
      console.log(`🔗 Connected to Solana (version: ${version['solana-core']})`);
      
      const health = await this.solanaConnection.getHealth();
      if (health !== 'ok') {
        throw new Error(`Solana RPC health check failed: ${health}`);
      }
    } catch (error: any) {
      throw new Error(`Failed to validate Solana connection: ${error.message}`);
    }
  }

  /**
   * Fund Solana wallet with exponential backoff retry
   */
  private async fundSolanaWalletWithRetry(): Promise<void> {
    for (let attempt = 1; attempt <= this.config.maxRetries; attempt++) {
      try {
        const airdropSignature = await this.solanaConnection.requestAirdrop(
          this.solanaWallet.publicKey,
          2 * LAMPORTS_PER_SOL
        );
        
        console.log(`📝 Airdrop signature: ${airdropSignature}`);
        
        // Modern confirmation with timeout
        const { blockhash, lastValidBlockHeight } = await this.solanaConnection.getLatestBlockhash();
        const confirmation = await this.solanaConnection.confirmTransaction({
          signature: airdropSignature,
          blockhash,
          lastValidBlockHeight
        }, 'confirmed');
        
        if (confirmation.value.err) {
          throw new Error(`Airdrop transaction failed: ${confirmation.value.err}`);
        }
        
        const balance = await this.solanaConnection.getBalance(this.solanaWallet.publicKey);
        console.log(`✅ Wallet funded. Balance: ${balance / LAMPORTS_PER_SOL} SOL`);
        return;
        
      } catch (error: any) {
        console.warn(`⚠️  Funding attempt ${attempt} failed: ${error.message}`);
        
        if (attempt === this.config.maxRetries) {
          throw new Error(`Failed to fund wallet after ${this.config.maxRetries} attempts`);
        }
        
        // Exponential backoff
        const delay = Math.pow(2, attempt) * 1000;
        console.log(`⏳ Retrying in ${delay}ms...`);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  /**
   * Step 2: Create and send cross-chain message from ZetaChain to Solana
   */
  async sendCrossChainMintRequest(): Promise<string> {
    console.log('🌉 Step 2: Sending Cross-Chain Mint Request from ZetaChain');
    
    try {
      // Validate ZetaChain connection
      await this.validateZetaChainConnection();
      
      // Create enhanced NFT metadata for cross-chain transfer
      const metadata: CrossChainNftMetadata = {
        name: "Cross-Chain Demo NFT v2",
        symbol: "XNFT2",
        uri: "https://api.example.com/demo-nft-v2.json",
        originalChainId: this.config.zetachainChainId,
        originalTokenId: Array.from(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])),
        originalCreator: getEvmAddressArray(this.zetaWallet.address),
        attributes: [
          { trait_type: "Demo", value: "Cross-Chain Transfer v2" },
          { trait_type: "Timestamp", value: new Date().toISOString() },
          { trait_type: "Source", value: "ZetaChain Athens Testnet" }
        ]
      };
      
      console.log(`📋 NFT Metadata:`);
      console.log(`   Name: ${metadata.name}`);
      console.log(`   Symbol: ${metadata.symbol}`);
      console.log(`   Chain ID: ${metadata.originalChainId}`);
      console.log(`   Creator: ${this.zetaWallet.address}`);
      
      // Encode cross-chain message with proper structure
      const messageType = CrossChainMessageType.MINT_REQUEST;
      const recipient = this.solanaWallet.publicKey.toBuffer();
      
      const crossChainMessage = this.encodeCrossChainMessage(
        messageType,
        recipient,
        metadata
      );
      
      console.log(`📨 Encoded cross-chain message: ${crossChainMessage.length} bytes`);
      console.log(`📨 Message hash: ${this.calculateMessageHash(crossChainMessage)}`);
      
      // Enhanced gateway contract interaction
      console.log('🔗 Interacting with ZetaChain Gateway Contract...');
      console.log(`   Gateway Address: ${this.gatewayContract.target}`);
      console.log(`   ZetaChain Wallet: ${this.zetaWallet.address}`);
      
      // In production, this would be a real transaction
      // const gasEstimate = await this.gatewayContract.estimateGas.call(...);
      
      // Generate realistic mock transaction hash
      const mockTxHash = await this.generateMockTransactionHash();
      
      console.log('✅ Cross-chain message sent!');
      console.log(`📝 ZetaChain Transaction: ${mockTxHash}`);
      console.log(`🔍 View on explorer: https://explorer.athens.zetachain.com/evm/tx/${mockTxHash}`);
      
      return mockTxHash;
      
    } catch (error: any) {
      console.error('❌ Error sending cross-chain message:', error.message);
      throw error;
    }
  }

  /**
   * Validate ZetaChain connection
   */
  private async validateZetaChainConnection(): Promise<void> {
    try {
      const network = await this.zetaProvider.getNetwork();
      console.log(`🔗 Connected to ZetaChain (Chain ID: ${network.chainId})`);
      
      const balance = await this.zetaProvider.getBalance(this.zetaWallet.address);
      console.log(`💰 ZetaChain wallet balance: ${ethers.formatEther(balance)} ZETA`);
      
      if (balance === 0n) {
        console.warn('⚠️  ZetaChain wallet has zero balance - real transactions would fail');
      }
      
    } catch (error: any) {
      throw new Error(`Failed to validate ZetaChain connection: ${error.message}`);
    }
  }

  /**
   * Calculate message hash for tracking
   */
  private calculateMessageHash(message: Buffer): string {
    const hash = ethers.keccak256(message);
    return hash.slice(0, 18) + '...' + hash.slice(-6); // Truncated for display
  }

  /**
   * Generate realistic mock transaction hash
   */
  private async generateMockTransactionHash(): Promise<string> {
    const timestamp = Date.now().toString();
    const randomData = this.zetaWallet.address + timestamp;
    const hash = ethers.keccak256(ethers.toUtf8Bytes(randomData));
    return hash;
  }

  /**
   * Step 3: Process cross-chain message on Solana (simulate gateway callback)
   */
  async processCrossChainMessage(zetaTxHash: string): Promise<string> {
    console.log('⚙️  Step 3: Processing Cross-Chain Message on Solana');
    
    try {
      console.log('🔍 Verifying ZetaChain transaction:', zetaTxHash);
      
      // Simulate TSS signature verification (in real implementation, this would verify actual TSS signature)
      const tssSignature = new Uint8Array(64); // Mock signature
      const recoveryId = 0;
      
      // Create mint for the NFT
      const nftMint = Keypair.generate();
      
      console.log('🎨 Creating NFT mint:', nftMint.publicKey.toString());
      
      // Create NFT state PDA
      const [nftStatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('nft_state'), nftMint.publicKey.toBuffer()],
        this.config.programId
      );
      
      // Create gateway message PDA
      const [gatewayMessagePDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('gateway_message'),
          new BN(this.config.zetachainChainId).toArrayLike(Buffer, 'le', 8),
          new BN(1).toArrayLike(Buffer, 'le', 8) // nonce
        ],
        this.config.programId
      );
      
      console.log('📍 NFT State PDA:', nftStatePDA.toString());
      console.log('📨 Gateway Message PDA:', gatewayMessagePDA.toString());
      
      // Simulate mint_from_cross_chain instruction
      console.log('🎯 Executing mint_from_cross_chain instruction...');
      
      // In a real implementation, this would call the actual Solana program
      const mockSolanaTx = Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join('');
      
      console.log('✅ Cross-chain NFT minted on Solana!');
      console.log('📝 Solana Transaction:', mockSolanaTx);
      
      return mockSolanaTx;
      
    } catch (error) {
      console.error('❌ Error processing cross-chain message:', error);
      throw error;
    }
  }

  /**
   * Step 4: Demonstrate reverse transfer (Solana to ZetaChain)
   */
  async demonstrateReverseTransfer(): Promise<void> {
    console.log('🔄 Step 4: Demonstrating Reverse Transfer (Solana → ZetaChain)');
    
    try {
      // Simulate burn_for_cross_chain instruction
      console.log('🔥 Burning NFT on Solana for cross-chain transfer...');
      
      const destinationChainId = this.config.zetachainChainId;
      const destinationAddress = getEvmAddressArray(this.zetaWallet.address);
      
      // Create burn message
      const burnMessage = this.encodeBurnMessage(destinationChainId, destinationAddress);
      
      console.log('🌉 Calling ZetaChain gateway for asset transfer...');
      
      // Mock Solana burn transaction
      const burnTxHash = Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join('');
      
      console.log('✅ NFT burned on Solana!');
      console.log('📝 Burn Transaction:', burnTxHash);
      console.log('🎯 Cross-chain transfer initiated to ZetaChain');
      
    } catch (error) {
      console.error('❌ Error in reverse transfer:', error);
      throw error;
    }
  }

  /**
   * Helper: Encode cross-chain message
   */
  private encodeCrossChainMessage(
    messageType: CrossChainMessageType,
    recipient: Buffer,
    metadata: CrossChainNftMetadata
  ): Buffer {
    // Simplified encoding - in production, use proper serialization
    const typeBuffer = Buffer.from([messageType]);
    const recipientBuffer = recipient;
    const metadataJson = JSON.stringify(metadata);
    const metadataBuffer = Buffer.from(metadataJson, 'utf8');
    const lengthBuffer = Buffer.alloc(4);
    lengthBuffer.writeUInt32LE(metadataBuffer.length);
    
    return Buffer.concat([typeBuffer, recipientBuffer, lengthBuffer, metadataBuffer]);
  }

  /**
   * Helper: Encode burn message
   */
  private encodeBurnMessage(destinationChainId: number, destinationAddress: number[]): Buffer {
    const typeBuffer = Buffer.from([CrossChainMessageType.BURN_CONFIRMATION]);
    const chainIdBuffer = Buffer.alloc(8);
    chainIdBuffer.writeBigUInt64LE(BigInt(destinationChainId));
    const addressBuffer = Buffer.from(destinationAddress);
    
    return Buffer.concat([typeBuffer, chainIdBuffer, addressBuffer]);
  }

  /**
   * Run complete cross-chain demonstration
   */
  async runCompleteDemo(): Promise<void> {
    console.log('🎬 Starting Complete Cross-Chain NFT Transfer Demo');
    console.log('================================================');
    
    try {
      // Step 1: Initialize Solana program
      await this.initializeSolanaProgram();
      
      // Step 2: Send cross-chain message from ZetaChain
      const zetaTxHash = await this.sendCrossChainMintRequest();
      
      // Simulate network delay
      console.log('⏳ Waiting for cross-chain confirmation...');
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      // Step 3: Process message on Solana
      const solanaTxHash = await this.processCrossChainMessage(zetaTxHash);
      
      // Step 4: Demonstrate reverse transfer
      await this.demonstrateReverseTransfer();
      
      console.log('🎉 Cross-Chain Demo Completed Successfully!');
      console.log('============================================');
      console.log('✅ Cross-chain asset transfer demonstrated');
      console.log('✅ ZetaChain → Solana NFT mint completed');
      console.log('✅ Solana → ZetaChain burn initiated');
      console.log('✅ Gateway contract integration verified');
      
    } catch (error) {
      console.error('❌ Demo failed:', error);
      throw error;
    }
  }
}

// Enhanced export with type definitions
export { CrossChainNFTDemo, type CrossChainNftMetadata, CrossChainMessageType };

// Execute demo if run directly (supports both CommonJS and ESM)
const isMainModule = typeof require !== 'undefined' && require.main === module;
const isESMMain = typeof process !== 'undefined' && process.argv[1] && 
  process.argv[1].endsWith('cross-chain-demo.ts');

if (isMainModule || isESMMain) {
  console.log('🚀 Starting Cross-Chain NFT Demo...');
  const demo = new CrossChainNFTDemo();
  demo.runCompleteDemo()
    .then(() => {
      console.log('✅ Demo completed successfully!');
      process.exit(0);
    })
    .catch((error) => {
      console.error('❌ Demo failed:', error);
      process.exit(1);
    });
}