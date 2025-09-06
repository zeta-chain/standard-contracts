import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY,
  Transaction
} from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  MINT_SIZE
} from "@solana/spl-token";

// Program ID from deployment
const PROGRAM_ID = new PublicKey("6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke");
const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export class UniversalNftClient {
  constructor(
    public program: Program<UniversalNft>,
    public provider: anchor.AnchorProvider
  ) {}

  /**
   * Initialize a new Universal NFT collection
   */
  async initializeCollection(
    name: string,
    symbol: string,
    uri: string,
    tssAddress: number[]
  ): Promise<{ collection: PublicKey; signature: string }> {
    const authority = this.provider.wallet.publicKey;
    
    // Derive collection PDA
    const [collection] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.toBuffer(),
        Buffer.from(name)
      ],
      this.program.programId
    );

    const signature = await this.program.methods
      .initializeCollection(name, symbol, uri, tssAddress)
      .accounts({
        authority,
        collection,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    return { collection, signature };
  }

  /**
   * Mint a new NFT with NFT Origin tracking
   */
  async mintNft(
    collection: PublicKey,
    name: string,
    symbol: string,
    uri: string,
    recipient?: PublicKey
  ): Promise<{ 
    mint: PublicKey; 
    nftOrigin: PublicKey; 
    tokenAccount: PublicKey;
    signature: string 
  }> {
    const authority = this.provider.wallet.publicKey;
    const mintKeypair = Keypair.generate();
    const recipientKey = recipient || authority;

    // Generate token ID for NFT Origin PDA
    const tokenId = this.generateTokenId(mintKeypair.publicKey, authority);
    
    // Derive NFT Origin PDA
    const [nftOrigin] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        Buffer.from(tokenId.toString('le'), 'hex').slice(0, 8)
      ],
      this.program.programId
    );

    // Derive associated token account
    const tokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      recipientKey
    );

    // Derive metadata PDA
    const [metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

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
      .rpc();

    return {
      mint: mintKeypair.publicKey,
      nftOrigin,
      tokenAccount,
      signature
    };
  }

  /**
   * Transfer NFT cross-chain
   */
  async transferCrossChain(
    collection: PublicKey,
    nftMint: PublicKey,
    destinationChainId: number,
    recipient: number[]
  ): Promise<string> {
    const authority = this.provider.wallet.publicKey;
    
    // Get associated token account
    const tokenAccount = await getAssociatedTokenAddress(nftMint, authority);
    
    // Generate token ID and derive NFT Origin PDA
    const tokenId = this.generateTokenId(nftMint, authority);
    const [nftOrigin] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        Buffer.from(tokenId.toString('le'), 'hex').slice(0, 8)
      ],
      this.program.programId
    );

    // Derive gateway PDA (placeholder - would be actual gateway)
    const [gateway] = PublicKey.findProgramAddressSync(
      [Buffer.from("gateway")],
      new PublicKey("GatewayAddress111111111111111111111111111") // Placeholder
    );

    const signature = await this.program.methods
      .transferCrossChain(new anchor.BN(destinationChainId), recipient)
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
      .rpc();

    return signature;
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
  ): Promise<string> {
    const authority = this.provider.wallet.publicKey;
    
    // Parse message to get token ID and derive NFT Origin PDA
    const tokenId = this.parseTokenIdFromMessage(message);
    const [nftOrigin] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        Buffer.from(tokenId.toString('le'), 'hex').slice(0, 8)
      ],
      this.program.programId
    );

    // Create new mint for incoming NFT
    const mintKeypair = Keypair.generate();
    const tokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      authority
    );

    // Derive metadata PDA
    const [metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    // Gateway PDA (placeholder)
    const [gateway] = PublicKey.findProgramAddressSync(
      [Buffer.from("gateway")],
      new PublicKey("GatewayAddress111111111111111111111111111")
    );

    const signature = await this.program.methods
      .onCall(sender, new anchor.BN(sourceChainId), message, new anchor.BN(nonce))
      .accounts({
        collection,
        recipient: authority,
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
      .rpc();

    return signature;
  }

  /**
   * Set universal contract address
   */
  async setUniversal(
    collection: PublicKey,
    universalAddress: PublicKey
  ): Promise<string> {
    const authority = this.provider.wallet.publicKey;

    const signature = await this.program.methods
      .setUniversal(universalAddress)
      .accounts({
        authority,
        collection,
      })
      .rpc();

    return signature;
  }

  /**
   * Set connected contract for a specific chain
   */
  async setConnected(
    collection: PublicKey,
    chainId: number[],
    contractAddress: number[]
  ): Promise<string> {
    const authority = this.provider.wallet.publicKey;
    
    // Derive connected PDA
    const [connected] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("connected"),
        collection.toBuffer(),
        Buffer.from(chainId)
      ],
      this.program.programId
    );

    const signature = await this.program.methods
      .setConnected(chainId, contractAddress)
      .accounts({
        authority,
        collection,
        connected,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return signature;
  }

  /**
   * Get NFT Origin data
   */
  async getNftOrigin(nftOrigin: PublicKey) {
    return await this.program.account.nftOrigin.fetch(nftOrigin);
  }

  /**
   * Get Collection data
   */
  async getCollection(collection: PublicKey) {
    return await this.program.account.collection.fetch(collection);
  }

  /**
   * Get Connected contract data
   */
  async getConnected(connected: PublicKey) {
    return await this.program.account.connected.fetch(connected);
  }

  /**
   * Helper: Generate deterministic token ID
   */
  private generateTokenId(mint: PublicKey, authority: PublicKey): bigint {
    // This should match the on-chain generation logic
    const hash = anchor.utils.sha256.hash(
      Buffer.concat([mint.toBuffer(), authority.toBuffer()])
    );
    return BigInt('0x' + hash.slice(0, 16));
  }

  /**
   * Helper: Parse token ID from cross-chain message
   */
  private parseTokenIdFromMessage(message: number[]): bigint {
    // Parse the first 8 bytes as token ID (little-endian)
    const tokenIdBytes = message.slice(0, 8);
    let tokenId = 0n;
    for (let i = 0; i < 8; i++) {
      tokenId |= BigInt(tokenIdBytes[i]) << BigInt(i * 8);
    }
    return tokenId;
  }

  /**
   * Helper: Derive collection PDA
   */
  static deriveCollectionPda(
    authority: PublicKey,
    name: string,
    programId: PublicKey = PROGRAM_ID
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.toBuffer(),
        Buffer.from(name)
      ],
      programId
    );
  }

  /**
   * Helper: Derive NFT Origin PDA
   */
  static deriveNftOriginPda(
    tokenId: bigint,
    programId: PublicKey = PROGRAM_ID
  ): [PublicKey, number] {
    const tokenIdBuffer = Buffer.alloc(8);
    tokenIdBuffer.writeBigUInt64LE(tokenId);
    
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_origin"),
        tokenIdBuffer
      ],
      programId
    );
  }

  /**
   * Helper: Derive Connected PDA
   */
  static deriveConnectedPda(
    collection: PublicKey,
    chainId: number[],
    programId: PublicKey = PROGRAM_ID
  ): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("connected"),
        collection.toBuffer(),
        Buffer.from(chainId)
      ],
      programId
    );
  }
}

// Usage example
export async function example() {
  // Setup
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;
  const client = new UniversalNftClient(program, provider);

  try {
    // 1. Initialize collection
    console.log("Initializing collection...");
    const { collection } = await client.initializeCollection(
      "Universal Collection",
      "UNI",
      "https://example.com/collection.json",
      Array.from(Buffer.alloc(20)) // TSS address placeholder
    );
    console.log("Collection created:", collection.toString());

    // 2. Mint NFT with Origin tracking
    console.log("Minting NFT...");
    const { mint, nftOrigin } = await client.mintNft(
      collection,
      "Universal NFT #1",
      "UNI",
      "https://example.com/nft1.json"
    );
    console.log("NFT minted:", mint.toString());
    console.log("NFT Origin PDA:", nftOrigin.toString());

    // 3. Get NFT Origin data
    const originData = await client.getNftOrigin(nftOrigin);
    console.log("Origin data:", originData);

    // 4. Set connected contract
    console.log("Setting connected contract...");
    await client.setConnected(
      collection,
      [1], // Ethereum mainnet
      Array.from(Buffer.from("0x1234567890123456789012345678901234567890", 'hex'))
    );

    // 5. Transfer cross-chain (simulation)
    console.log("Initiating cross-chain transfer...");
    await client.transferCrossChain(
      collection,
      mint,
      1, // Ethereum
      Array.from(Buffer.from("0x1234567890123456789012345678901234567890", 'hex'))
    );

    console.log("✅ All operations completed successfully!");

  } catch (error) {
    console.error("❌ Error:", error);
  }
}
