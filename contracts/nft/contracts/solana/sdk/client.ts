import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN, EventParser } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY,
  Transaction,
  Connection,
  Commitment,
  ConfirmOptions,
  TransactionSignature,
  SendOptions,
  RpcResponseAndContext,
  SignatureResult,
  AccountInfo,
  GetProgramAccountsFilter,
  MemcmpFilter
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  MINT_SIZE,
  getAccount,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError
} from "@solana/spl-token";
import { WalletAdapter } from "@solana/wallet-adapter-base";

// Constants
export const PROGRAM_ID = new PublicKey("6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke");
export const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
export const ZETACHAIN_GATEWAY_PROGRAM_ID = new PublicKey("GatewayAddress111111111111111111111111111");

// Network configurations
export enum Network {
  DEVNET = "devnet",
  TESTNET = "testnet",
  MAINNET = "mainnet-beta",
  LOCALNET = "localnet"
}

export const NETWORK_ENDPOINTS = {
  [Network.DEVNET]: "https://api.devnet.solana.com",
  [Network.TESTNET]: "https://api.testnet.solana.com",
  [Network.MAINNET]: "https://api.mainnet-beta.solana.com",
  [Network.LOCALNET]: "http://localhost:8899"
};

// Error classes
export class UniversalNftError extends Error {
  constructor(message: string, public code?: string) {
    super(message);
    this.name = "UniversalNftError";
  }
}

export class TransactionError extends UniversalNftError {
  constructor(message: string, public signature?: string) {
    super(message, "TRANSACTION_ERROR");
  }
}

export class AccountNotFoundError extends UniversalNftError {
  constructor(account: string) {
    super(`Account not found: ${account}`, "ACCOUNT_NOT_FOUND");
  }
}

export class InvalidParameterError extends UniversalNftError {
  constructor(parameter: string, value: any) {
    super(`Invalid parameter ${parameter}: ${value}`, "INVALID_PARAMETER");
  }
}

// Type definitions
export interface CollectionData {
  authority: PublicKey;
  name: string;
  symbol: string;
  uri: string;
  tssAddress: number[];
  universalAddress?: PublicKey;
  totalSupply: BN;
  bump: number;
}

export interface NftOriginData {
  tokenId: BN;
  originChain: number[];
  originContract: number[];
  isReturning: boolean;
  bump: number;
}

export interface ConnectedData {
  chainId: number[];
  contractAddress: number[];
  bump: number;
}

export interface MintResult {
  mint: PublicKey;
  nftOrigin: PublicKey;
  tokenAccount: PublicKey;
  metadata: PublicKey;
  signature: string;
}

export interface TransferResult {
  signature: string;
  tokenId: BN;
  destinationChain: number;
  recipient: number[];
}

export interface CrossChainMessage {
  tokenId: BN;
  name: string;
  symbol: string;
  uri: string;
  recipient: PublicKey;
}

export interface EventSubscription {
  id: string;
  unsubscribe: () => void;
}

export interface ClientConfig {
  network?: Network;
  endpoint?: string;
  commitment?: Commitment;
  confirmOptions?: ConfirmOptions;
  programId?: PublicKey;
  skipPreflight?: boolean;
}

export interface WalletConfig {
  adapter?: WalletAdapter;
  keypair?: Keypair;
}

// Event types
export interface NftMintedEvent {
  collection: PublicKey;
  mint: PublicKey;
  recipient: PublicKey;
  tokenId: BN;
  name: string;
  symbol: string;
  uri: string;
}

export interface CrossChainTransferEvent {
  collection: PublicKey;
  mint: PublicKey;
  sender: PublicKey;
  destinationChain: number;
  recipient: number[];
  tokenId: BN;
}

export interface CrossChainReceiveEvent {
  collection: PublicKey;
  mint: PublicKey;
  recipient: PublicKey;
  sourceChain: number;
  sender: number[];
  tokenId: BN;
  isReturning: boolean;
}

// Main SDK Client
export class UniversalNftClient {
  public readonly program: Program<UniversalNft>;
  public readonly provider: AnchorProvider;
  public readonly connection: Connection;
  public readonly config: Required<ClientConfig>;
  private eventParser: EventParser;
  private subscriptions: Map<string, EventSubscription> = new Map();

  constructor(
    program: Program<UniversalNft>,
    provider: AnchorProvider,
    config: ClientConfig = {}
  ) {
    this.program = program;
    this.provider = provider;
    this.connection = provider.connection;
    this.config = {
      network: config.network || Network.DEVNET,
      endpoint: config.endpoint || NETWORK_ENDPOINTS[config.network || Network.DEVNET],
      commitment: config.commitment || "confirmed",
      confirmOptions: config.confirmOptions || { commitment: "confirmed" },
      programId: config.programId || PROGRAM_ID,
      skipPreflight: config.skipPreflight || false
    };
    this.eventParser = new EventParser(this.program.programId, this.program.coder);
  }

  /**
   * Create a new UniversalNftClient instance
   */
  static async create(
    config: ClientConfig = {},
    walletConfig: WalletConfig = {}
  ): Promise<UniversalNftClient> {
    const endpoint = config.endpoint || NETWORK_ENDPOINTS[config.network || Network.DEVNET];
    const connection = new Connection(endpoint, config.commitment || "confirmed");
    
    let wallet: any;
    if (walletConfig.adapter) {
      wallet = walletConfig.adapter;
    } else if (walletConfig.keypair) {
      wallet = {
        publicKey: walletConfig.keypair.publicKey,
        signTransaction: async (tx: Transaction) => {
          tx.sign(walletConfig.keypair!);
          return tx;
        },
        signAllTransactions: async (txs: Transaction[]) => {
          txs.forEach(tx => tx.sign(walletConfig.keypair!));
          return txs;
        }
      };
    } else {
      throw new InvalidParameterError("wallet", "Either adapter or keypair must be provided");
    }

    const provider = new AnchorProvider(
      connection,
      wallet,
      config.confirmOptions || { commitment: "confirmed" }
    );

    // Load IDL and create program
    const idl = await Program.fetchIdl(config.programId || PROGRAM_ID, provider);
    if (!idl) {
      throw new UniversalNftError("Failed to fetch program IDL");
    }

    const program = new Program(idl as any, config.programId || PROGRAM_ID, provider);
    
    return new UniversalNftClient(program, provider, config);
  }

  // Collection Management
  /**
   * Initialize a new Universal NFT collection
   */
  async initializeCollection(
    name: string,
    symbol: string,
    uri: string,
    tssAddress: number[]
  ): Promise<{ collection: PublicKey; signature: string }> {
    if (!name || name.length > 32) {
      throw new InvalidParameterError("name", name);
    }
    if (!symbol || symbol.length > 10) {
      throw new InvalidParameterError("symbol", symbol);
    }
    if (!uri) {
      throw new InvalidParameterError("uri", uri);
    }
    if (!tssAddress || tssAddress.length !== 20) {
      throw new InvalidParameterError("tssAddress", tssAddress);
    }

    const authority = this.provider.wallet.publicKey;
    
    // Derive collection PDA
    const [collection] = this.deriveCollectionPda(authority, name);

    try {
      const signature = await this.program.methods
        .initializeCollection(name, symbol, uri, tssAddress)
        .accounts({
          authority,
          collection,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc(this.config.confirmOptions);

      return { collection, signature };
    } catch (error) {
      throw new TransactionError(`Failed to initialize collection: ${error}`);
    }
  }

  /**
   * Get collection data
   */
  async getCollection(collection: PublicKey): Promise<CollectionData> {
    try {
      return await this.program.account.collection.fetch(collection);
    } catch (error) {
      throw new AccountNotFoundError(collection.toString());
    }
  }

  /**
   * Get all collections for an authority
   */
  async getCollectionsByAuthority(authority: PublicKey): Promise<Array<{ pubkey: PublicKey; account: CollectionData }>> {
    const filters: GetProgramAccountsFilter[] = [
      {
        memcmp: {
          offset: 8, // Skip discriminator
          bytes: authority.toBase58(),
        } as MemcmpFilter,
      },
    ];

    try {
      const accounts = await this.program.account.collection.all(filters);
      return accounts;
    } catch (error) {
      throw new UniversalNftError(`Failed to fetch collections: ${error}`);
    }
  }

  // NFT Minting
  /**
   * Mint a new NFT with NFT Origin tracking
   */
  async mintNft(
    collection: PublicKey,
    name: string,
    symbol: string,
    uri: string,
    recipient?: PublicKey
  ): Promise<MintResult> {
    if (!name || name.length > 32) {
      throw new InvalidParameterError("name", name);
    }
    if (!symbol || symbol.length > 10) {
      throw new InvalidParameterError("symbol", symbol);
    }
    if (!uri) {
      throw new InvalidParameterError("uri", uri);
    }

    const authority = this.provider.wallet.publicKey;
    const mintKeypair = Keypair.generate();
    const recipientKey = recipient || authority;

    // Generate token ID for NFT Origin PDA
    const tokenId = this.generateTokenId(mintKeypair.publicKey, authority);
    
    // Derive PDAs
    const [nftOrigin] = this.deriveNftOriginPda(tokenId);
    const tokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      recipientKey
    );
    const [metadata] = this.deriveMetadataPda(mintKeypair.publicKey);

    try {
      const signature = await this.program.methods
        .mintNft(name, symbol, uri)
        .accounts({
          authority,
          collection,
          nftMint: mintKeypair.publicKey,
          nftTokenAccount: tokenAccount,
          recipient: recipientKey,
          nftOrigin,
          nftMetadata: metadata,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([mintKeypair])
        .rpc(this.config.confirmOptions);

      return {
        mint: mintKeypair.publicKey,
        nftOrigin,
        tokenAccount,
        metadata,
        signature
      };
    } catch (error) {
      throw new TransactionError(`Failed to mint NFT: ${error}`);
    }
  }

  // Cross-chain Operations
  /**
   * Transfer NFT cross-chain
   */
  async transferCrossChain(
    collection: PublicKey,
    nftMint: PublicKey,
    destinationChainId: number,
    recipient: number[]
  ): Promise<TransferResult> {
    if (destinationChainId <= 0) {
      throw new InvalidParameterError("destinationChainId", destinationChainId);
    }
    if (!recipient || recipient.length !== 20) {
      throw new InvalidParameterError("recipient", recipient);
    }

    const authority = this.provider.wallet.publicKey;
    
    // Get associated token account
    const tokenAccount = await getAssociatedTokenAddress(nftMint, authority);
    
    // Verify token ownership
    try {
      const tokenAccountInfo = await getAccount(this.connection, tokenAccount);
      if (tokenAccountInfo.amount === 0n) {
        throw new UniversalNftError("Token account has no tokens");
      }
    } catch (error) {
      if (error instanceof TokenAccountNotFoundError) {
        throw new UniversalNftError("Token account not found");
      }
      throw error;
    }
    
    // Generate token ID and derive NFT Origin PDA
    const tokenId = this.generateTokenId(nftMint, authority);
    const [nftOrigin] = this.deriveNftOriginPda(tokenId);

    // Derive gateway PDA
    const [gateway] = this.deriveGatewayPda();

    try {
      const signature = await this.program.methods
        .transferCrossChain(new BN(destinationChainId), recipient)
        .accounts({
          sender: authority,
          collection,
          nftMint,
          nftTokenAccount: tokenAccount,
          nftOrigin,
          gateway,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc(this.config.confirmOptions);

      return {
        signature,
        tokenId,
        destinationChain: destinationChainId,
        recipient
      };
    } catch (error) {
      throw new TransactionError(`Failed to transfer cross-chain: ${error}`);
    }
  }

  /**
   * Handle incoming cross-chain NFT transfer
   */
  async onCall(
    collection: PublicKey,
    sender: number[],
    sourceChainId: number,
    message: number[],
    nonce: number
  ): Promise<{ mint: PublicKey; signature: string; isReturning: boolean }> {
    if (!sender || sender.length !== 20) {
      throw new InvalidParameterError("sender", sender);
    }
    if (sourceChainId <= 0) {
      throw new InvalidParameterError("sourceChainId", sourceChainId);
    }
    if (!message || message.length === 0) {
      throw new InvalidParameterError("message", message);
    }

    const authority = this.provider.wallet.publicKey;
    
    // Parse message to get token ID and metadata
    const parsedMessage = this.parseMessage(message);
    const [nftOrigin] = this.deriveNftOriginPda(parsedMessage.tokenId);

    // Check if NFT Origin exists to determine if this is a returning NFT
    let isReturning = false;
    try {
      await this.getNftOrigin(nftOrigin);
      isReturning = true;
    } catch (error) {
      // NFT Origin doesn't exist, this is a new NFT
      isReturning = false;
    }

    // Create new mint for incoming NFT
    const mintKeypair = Keypair.generate();
    const tokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      parsedMessage.recipient
    );

    // Derive metadata PDA
    const [metadata] = this.deriveMetadataPda(mintKeypair.publicKey);
    const [gateway] = this.deriveGatewayPda();

    try {
      const signature = await this.program.methods
        .onCall(sender, new BN(sourceChainId), message, new BN(nonce))
        .accounts({
          collection,
          recipient: parsedMessage.recipient,
          nftMint: mintKeypair.publicKey,
          nftTokenAccount: tokenAccount,
          nftOrigin,
          nftMetadata: metadata,
          gateway,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .signers([mintKeypair])
        .rpc(this.config.confirmOptions);

      return {
        mint: mintKeypair.publicKey,
        signature,
        isReturning
      };
    } catch (error) {
      throw new TransactionError(`Failed to handle cross-chain call: ${error}`);
    }
  }

  // Configuration Management
  /**
   * Set universal contract address
   */
  async setUniversal(
    collection: PublicKey,
    universalAddress: PublicKey
  ): Promise<string> {
    const authority = this.provider.wallet.publicKey;

    try {
      const signature = await this.program.methods
        .setUniversal(universalAddress)
        .accounts({
          authority,
          collection,
        })
        .rpc(this.config.confirmOptions);

      return signature;
    } catch (error) {
      throw new TransactionError(`Failed to set universal address: ${error}`);
    }
  }

  /**
   * Set connected contract for a specific chain
   */
  async setConnected(
    collection: PublicKey,
    chainId: number[],
    contractAddress: number[]
  ): Promise<string> {
    if (!chainId || chainId.length === 0) {
      throw new InvalidParameterError("chainId", chainId);
    }
    if (!contractAddress || contractAddress.length !== 20) {
      throw new InvalidParameterError("contractAddress", contractAddress);
    }

    const authority = this.provider.wallet.publicKey;
    
    // Derive connected PDA
    const [connected] = this.deriveConnectedPda(collection, chainId);

    try {
      const signature = await this.program.methods
        .setConnected(chainId, contractAddress)
        .accounts({
          authority,
          collection,
          connected,
          systemProgram: SystemProgram.programId,
        })
        .rpc(this.config.confirmOptions);

      return signature;
    } catch (error) {
      throw new TransactionError(`Failed to set connected contract: ${error}`);
    }
  }

  // Data Queries
  /**
   * Get NFT Origin data
   */
  async getNftOrigin(nftOrigin: PublicKey): Promise<NftOriginData> {
    try {
      return await this.program.account.nftOrigin.fetch(nftOrigin);
    } catch (error) {
      throw new AccountNotFoundError(nftOrigin.toString());
    }
  }

  /**
   * Get Connected contract data
   */
  async getConnected(connected: PublicKey): Promise<ConnectedData> {
    try {
      return await this.program.account.connected.fetch(connected);
    } catch (error) {
      throw new AccountNotFoundError(connected.toString());
    }
  }

  /**
   * Get all NFTs for a collection
   */
  async getNftsByCollection(collection: PublicKey): Promise<Array<{ pubkey: PublicKey; account: NftOriginData }>> {
    // This would require indexing or filtering by collection
    // For now, we'll get all NFT Origins and filter client-side
    try {
      const accounts = await this.program.account.nftOrigin.all();
      return accounts;
    } catch (error) {
      throw new UniversalNftError(`Failed to fetch NFTs: ${error}`);
    }
  }

  /**
   * Get NFT by token ID
   */
  async getNftByTokenId(tokenId: BN): Promise<{ pubkey: PublicKey; account: NftOriginData }> {
    const [nftOrigin] = this.deriveNftOriginPda(tokenId);
    const account = await this.getNftOrigin(nftOrigin);
    return { pubkey: nftOrigin, account };
  }

  // Event Monitoring
  /**
   * Subscribe to NFT minted events
   */
  async onNftMinted(
    callback: (event: NftMintedEvent) => void,
    collection?: PublicKey
  ): Promise<EventSubscription> {
    const id = `nft_minted_${Date.now()}_${Math.random()}`;
    
    const listener = this.program.addEventListener("NftMinted", (event, slot) => {
      if (!collection || event.collection.equals(collection)) {
        callback(event as NftMintedEvent);
      }
    });

    const subscription: EventSubscription = {
      id,
      unsubscribe: () => {
        this.program.removeEventListener(listener);
        this.subscriptions.delete(id);
      }
    };

    this.subscriptions.set(id, subscription);
    return subscription;
  }

  /**
   * Subscribe to cross-chain transfer events
   */
  async onCrossChainTransfer(
    callback: (event: CrossChainTransferEvent) => void,
    collection?: PublicKey
  ): Promise<EventSubscription> {
    const id = `cross_chain_transfer_${Date.now()}_${Math.random()}`;
    
    const listener = this.program.addEventListener("CrossChainTransfer", (event, slot) => {
      if (!collection || event.collection.equals(collection)) {
        callback(event as CrossChainTransferEvent);
      }
    });

    const subscription: EventSubscription = {
      id,
      unsubscribe: () => {
        this.program.removeEventListener(listener);
        this.subscriptions.delete(id);
      }
    };

    this.subscriptions.set(id, subscription);
    return subscription;
  }

  /**
   * Subscribe to cross-chain receive events
   */
  async onCrossChainReceive(
    callback: (event: CrossChainReceiveEvent) => void,
    collection?: PublicKey
  ): Promise<EventSubscription> {
    const id = `cross_chain_receive_${Date.now()}_${Math.random()}`;
    
    const listener = this.program.addEventListener("CrossChainReceive", (event, slot) => {
      if (!collection || event.collection.equals(collection)) {
        callback(event as CrossChainReceiveEvent);
      }
    });

    const subscription: EventSubscription = {
      id,
      unsubscribe: () => {
        this.program.removeEventListener(listener);
        this.subscriptions.delete(id);
      }
    };

    this.subscriptions.set(id, subscription);
    return subscription;
  }

  /**
   * Unsubscribe from all events
   */
  unsubscribeAll(): void {
    this.subscriptions.forEach(subscription => subscription.unsubscribe());
    this.subscriptions.clear();
  }

  // Transaction Utilities
  /**
   * Build transaction without sending
   */
  async buildTransaction(
    instructions: anchor.web3.TransactionInstruction[],
    signers: Keypair[] = []
  ): Promise<Transaction> {
    const transaction = new Transaction();
    transaction.add(...instructions);
    
    const { blockhash } = await this.connection.getLatestBlockhash(this.config.commitment);
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = this.provider.wallet.publicKey;

    if (signers.length > 0) {
      transaction.sign(...signers);
    }

    return transaction;
  }

  /**
   * Send and confirm transaction with retry logic
   */
  async sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Keypair[] = [],
    options: SendOptions = {}
  ): Promise<string> {
    const maxRetries = 3;
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        if (signers.length > 0) {
          transaction.sign(...signers);
        }

        const signature = await this.provider.sendAndConfirm(
          transaction,
          signers,
          { ...this.config.confirmOptions, ...options }
        );

        return signature;
      } catch (error) {
        lastError = error as Error;
        
        if (attempt === maxRetries) {
          throw new TransactionError(`Transaction failed after ${maxRetries} attempts: ${lastError.message}`);
        }

        // Wait before retry
        await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
        
        // Update blockhash for retry
        const { blockhash } = await this.connection.getLatestBlockhash(this.config.commitment);
        transaction.recentBlockhash = blockhash;
      }
    }

    throw new TransactionError(`Transaction failed: ${lastError?.message}`);
  }

  // Helper Methods
  /**
   * Generate deterministic token ID
   */
  generateTokenId(mint: PublicKey, authority: PublicKey): BN {
    const hash = anchor.utils.sha256.hash(
      Buffer.concat([mint.toBuffer(), authority.toBuffer()])
    );
    const tokenIdBytes = hash.slice(0, 8);
    return new BN(tokenIdBytes, 'le');
  }

  /**
   * Parse cross-chain message
   */
  parseMessage(message: number[]): CrossChainMessage {
    if (message.length < 8) {
      throw new InvalidParameterError("message", "Message too short");
    }

    // Parse token ID (first 8 bytes, little-endian)
    const tokenIdBytes = message.slice(0, 8);
    let tokenId = new BN(0);
    for (let i = 0; i < 8; i++) {
      tokenId = tokenId.or(new BN(tokenIdBytes[i]).shln(i * 8));
    }

    // Parse remaining message (simplified - would need proper ABI/Borsh decoding)
    const remainingBytes = message.slice(8);
    
    // For now, return a basic structure
    return {
      tokenId,
      name: "Cross-chain NFT",
      symbol: "XNFT",
      uri: "https://example.com/metadata.json",
      recipient: this.provider.wallet.publicKey
    };
  }

  /**
   * Derive collection PDA
   */
  deriveCollectionPda(authority: PublicKey, name: string): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.toBuffer(),
        Buffer.from(name)
      ],
      this.program.programId
    );
  }

  /**
   * Derive NFT Origin PDA
   */
  deriveNftOriginPda(tokenId: BN): [PublicKey, number] {
    const tokenIdBuffer = Buffer.alloc(8);
    tokenId.toArrayLike(Buffer, 'le', 8).copy(tokenIdBuffer);
    
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        tokenIdBuffer
      ],
      this.program.programId
    );
  }

  /**
   * Derive Connected PDA
   */
  deriveConnectedPda(collection: PublicKey, chainId: number[]): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("connected"),
        collection.toBuffer(),
        Buffer.from(chainId)
      ],
      this.program.programId
    );
  }

  /**
   * Derive Metadata PDA
   */
  deriveMetadataPda(mint: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
  }

  /**
   * Derive Gateway PDA
   */
  deriveGatewayPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("gateway")],
      ZETACHAIN_GATEWAY_PROGRAM_ID
    );
  }

  /**
   * Get account info with error handling
   */
  async getAccountInfo(pubkey: PublicKey): Promise<AccountInfo<Buffer> | null> {
    try {
      return await this.connection.getAccountInfo(pubkey, this.config.commitment);
    } catch (error) {
      throw new UniversalNftError(`Failed to get account info: ${error}`);
    }
  }

  /**
   * Check if account exists
   */
  async accountExists(pubkey: PublicKey): Promise<boolean> {
    const accountInfo = await this.getAccountInfo(pubkey);
    return accountInfo !== null;
  }

  /**
   * Get current slot
   */
  async getCurrentSlot(): Promise<number> {
    return await this.connection.getSlot(this.config.commitment);
  }

  /**
   * Get transaction status
   */
  async getTransactionStatus(signature: string): Promise<RpcResponseAndContext<SignatureResult | null>> {
    return await this.connection.getSignatureStatus(signature, {
      searchTransactionHistory: true
    });
  }

  /**
   * Wait for transaction confirmation
   */
  async waitForConfirmation(
    signature: string,
    timeout: number = 60000
  ): Promise<RpcResponseAndContext<SignatureResult | null>> {
    const start = Date.now();
    
    while (Date.now() - start < timeout) {
      const status = await this.getTransactionStatus(signature);
      
      if (status.value?.confirmationStatus === this.config.commitment) {
        return status;
      }
      
      if (status.value?.err) {
        throw new TransactionError(`Transaction failed: ${JSON.stringify(status.value.err)}`, signature);
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new TransactionError(`Transaction confirmation timeout`, signature);
  }

  /**
   * Cleanup resources
   */
  dispose(): void {
    this.unsubscribeAll();
  }
}

// Utility functions
export class UniversalNftUtils {
  /**
   * Convert Ethereum address to bytes array
   */
  static ethAddressToBytes(address: string): number[] {
    if (!address.startsWith('0x') || address.length !== 42) {
      throw new InvalidParameterError("address", address);
    }
    return Array.from(Buffer.from(address.slice(2), 'hex'));
  }

  /**
   * Convert bytes array to Ethereum address
   */
  static bytesToEthAddress(bytes: number[]): string {
    if (bytes.length !== 20) {
      throw new InvalidParameterError("bytes", bytes);
    }
    return '0x' + Buffer.from(bytes).toString('hex');
  }

  /**
   * Convert chain ID to bytes array
   */
  static chainIdToBytes(chainId: number): number[] {
    const buffer = Buffer.alloc(4);
    buffer.writeUInt32BE(chainId, 0);
    return Array.from(buffer);
  }

  /**
   * Convert bytes array to chain ID
   */
  static bytesToChainId(bytes: number[]): number {
    if (bytes.length !== 4) {
      throw new InvalidParameterError("bytes", bytes);
    }
    return Buffer.from(bytes).readUInt32BE(0);
  }

  /**
   * Validate Solana public key
   */
  static isValidPublicKey(key: string): boolean {
    try {
      new PublicKey(key);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Validate Ethereum address
   */
  static isValidEthAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  /**
   * Format token amount for display
   */
  static formatTokenAmount(amount: BN, decimals: number = 0): string {
    if (decimals === 0) {
      return amount.toString();
    }
    
    const divisor = new BN(10).pow(new BN(decimals));
    const quotient = amount.div(divisor);
    const remainder = amount.mod(divisor);
    
    if (remainder.isZero()) {
      return quotient.toString();
    }
    
    const remainderStr = remainder.toString().padStart(decimals, '0');
    return `${quotient.toString()}.${remainderStr}`;
  }

  /**
   * Parse token amount from string
   */
  static parseTokenAmount(amount: string, decimals: number = 0): BN {
    if (decimals === 0) {
      return new BN(amount);
    }
    
    const parts = amount.split('.');
    const wholePart = new BN(parts[0] || '0');
    const fractionalPart = parts[1] || '';
    
    const fractionalBN = new BN(fractionalPart.padEnd(decimals, '0').slice(0, decimals));
    const multiplier = new BN(10).pow(new BN(decimals));
    
    return wholePart.mul(multiplier).add(fractionalBN);
  }
}

// Export everything
export default UniversalNftClient;