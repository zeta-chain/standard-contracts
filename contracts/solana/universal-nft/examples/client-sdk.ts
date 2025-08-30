import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNftProgram } from "../target/types/universal_nft_program";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Connection,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { getEvmAddressArray } from "../utils/address";

/**
 * Universal NFT Client SDK
 * 
 * This SDK provides a convenient interface for interacting with the Universal NFT Program
 * Handles all the complexity of account derivation and transaction construction
 */
export class UniversalNftClient {
  private program: Program<UniversalNftProgram>;
  private provider: anchor.AnchorProvider;
  public programConfigPda: PublicKey;
  
  constructor(
    program: Program<UniversalNftProgram>,
    provider: anchor.AnchorProvider
  ) {
    this.program = program;
    this.provider = provider;
    
    // Derive program config PDA
    [this.programConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_program")],
      program.programId
    );
  }

  /**
   * Initialize the Universal NFT Program
   */
  async initializeProgram(
    authority: Keypair,
    gatewayProgramId: PublicKey,
    collectionName: string,
    collectionSymbol: string,
    collectionUri: string
  ): Promise<{ signature: string; collectionMint: PublicKey }> {
    const collectionMint = Keypair.generate();
    
    const [collectionMetadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const [collectionMasterEdition] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const collectionTokenAccount = await getAssociatedTokenAddress(
      collectionMint.publicKey,
      authority.publicKey
    );

    const signature = await this.program.methods
      .initializeProgram(
        gatewayProgramId,
        collectionName,
        collectionSymbol,
        collectionUri
      )
      .accounts({
        programConfig: this.programConfigPda,
        collectionMint: collectionMint.publicKey,
        collectionMetadata,
        collectionMasterEdition,
        collectionTokenAccount,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority, collectionMint])
      .rpc();

    return { signature, collectionMint: collectionMint.publicKey };
  }

  /**
   * Mint a new NFT
   */
  async mintNft(
    owner: Keypair,
    authority: Keypair,
    metadata: {
      name: string;
      symbol: string;
      uri: string;
      creators?: any[];
    }
  ): Promise<{ signature: string; mint: PublicKey; tokenAccount: PublicKey }> {
    const nftMint = Keypair.generate();
    
    const [nftStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_state"), nftMint.publicKey.toBuffer()],
      this.program.programId
    );

    const [nftMetadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        nftMint.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const nftTokenAccount = await getAssociatedTokenAddress(
      nftMint.publicKey,
      owner.publicKey
    );

    const signature = await this.program.methods
      .mintNft(metadata.name, metadata.symbol, metadata.uri, metadata.creators || null)
      .accounts({
        programConfig: this.programConfigPda,
        nftState: nftStatePda,
        nftMint: nftMint.publicKey,
        nftMetadata,
        nftTokenAccount,
        owner: owner.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([owner, authority, nftMint])
      .rpc();

    return {
      signature,
      mint: nftMint.publicKey,
      tokenAccount: nftTokenAccount,
    };
  }

  /**
   * Burn NFT for cross-chain transfer
   */
  async burnForCrossChain(
    owner: Keypair,
    nftMint: PublicKey,
    destinationChainId: number,
    destinationAddress: string
  ): Promise<string> {
    const [nftStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_state"), nftMint.toBuffer()],
      this.program.programId
    );

    const nftTokenAccount = await getAssociatedTokenAddress(
      nftMint,
      owner.publicKey
    );

    // Convert and validate EVM address with checksum
    const addressBytes = getEvmAddressArray(destinationAddress);

    const signature = await this.program.methods
      .burnForCrossChain(
        new anchor.BN(destinationChainId),
        addressBytes
      )
      .accounts({
        programConfig: this.programConfigPda,
        nftState: nftStatePda,
        nftMint,
        nftTokenAccount,
        owner: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    return signature;
  }

  /**
   * Get NFT state information
   */
  async getNftState(nftMint: PublicKey) {
    const [nftStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_state"), nftMint.toBuffer()],
      this.program.programId
    );

    try {
      return await this.program.account.nftState.fetch(nftStatePda);
    } catch (error) {
      throw new Error(`NFT state not found for mint: ${nftMint.toString()}`);
    }
  }

  /**
   * Get program configuration
   */
  async getProgramConfig() {
    return await this.program.account.programConfig.fetch(this.programConfigPda);
  }

  /**
   * Update gateway configuration (admin only)
   */
  async updateGatewayConfig(
    authority: Keypair,
    newGatewayProgramId?: PublicKey,
    newTssAddress?: Uint8Array
  ): Promise<string> {
    const signature = await this.program.methods
      .updateGatewayConfig(
        newGatewayProgramId || null,
        newTssAddress ? Array.from(newTssAddress) : null
      )
      .accounts({
        programConfig: this.programConfigPda,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    return signature;
  }

  /**
   * Helper: Get all NFTs owned by a wallet
   */
  async getNftsByOwner(ownerPublicKey: PublicKey): Promise<any[]> {
    // This would require indexing or scanning the blockchain
    // In practice, you'd use a service like Helius, QuickNode, or the RPC getParsedTokenAccountsByOwner
    const accounts = await this.provider.connection.getParsedTokenAccountsByOwner(
      ownerPublicKey,
      { programId: TOKEN_PROGRAM_ID }
    );

    const nfts = [];
    for (const account of accounts.value) {
      // Use the parsed info instead of slicing raw bytes
      const info: any = account.account.data.parsed.info;
      const amount = info.tokenAmount;
      if (amount.decimals === 0 && amount.uiAmount === 1) {
        // This might be an NFT, check if it has our NFT state
        const mintPubkey = new PublicKey(info.mint);
        try {
          const nftState = await this.getNftState(mintPubkey);
          nfts.push({
            mint: mintPubkey.toString(),
            tokenAccount: account.pubkey.toString(),
            state: nftState,
          });
        } catch {
          // Not our NFT, skip
        }
      }
    }

    return nfts;
  }

  /**
   * Helper: Format cross-chain history for display
   */
  formatCrossChainHistory(history: any[]): string {
    return history.map((transfer, index) => {
      const type = transfer.transferType.outbound ? "Outbound" : "Inbound";
      const chainId = transfer.destinationChainId.toString();
      const address = Buffer.from(transfer.destinationAddress).toString('hex');
      const timestamp = new Date(transfer.transferTimestamp * 1000).toISOString();
      
      return `${index + 1}. ${type} to Chain ${chainId} (${address}) at ${timestamp}`;
    }).join('\n');
  }
}

/**
 * Example usage of the Universal NFT Client SDK
 */
export async function exampleUsage() {
  // Setup
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNftProgram as Program<UniversalNftProgram>;
  const client = new UniversalNftClient(program, provider);

  // Generate test keypairs
  const authority = Keypair.generate();
  const user = Keypair.generate();
  
  // Fund accounts (in devnet/localnet)
  await provider.connection.requestAirdrop(authority.publicKey, 2e9);
  await provider.connection.requestAirdrop(user.publicKey, 2e9);
  await new Promise(resolve => setTimeout(resolve, 1000));

  try {
    console.log("🚀 Universal NFT SDK Example");
    console.log("Program ID:", program.programId.toString());
    console.log("Authority:", authority.publicKey.toString());
    console.log("User:", user.publicKey.toString());

    // 1. Initialize program
    console.log("\n1️⃣ Initializing program...");
    const gatewayProgramId = Keypair.generate().publicKey; // Placeholder
    const { signature: initSig, collectionMint } = await client.initializeProgram(
      authority,
      gatewayProgramId,
      "SDK Example Collection",
      "SDKEX",
      "https://example.com/collection.json"
    );
    console.log("✅ Program initialized:", initSig);
    console.log("📦 Collection mint:", collectionMint.toString());

    // 2. Mint an NFT
    console.log("\n2️⃣ Minting NFT...");
    const { signature: mintSig, mint: nftMint } = await client.mintNft(
      user,
      authority,
      {
        name: "SDK Example NFT",
        symbol: "SDKNFT",
        uri: "https://example.com/nft.json",
      }
    );
    console.log("✅ NFT minted:", mintSig);
    console.log("🎨 NFT mint:", nftMint.toString());

    // 3. Get NFT state
    console.log("\n3️⃣ Fetching NFT state...");
    const nftState = await client.getNftState(nftMint);
    console.log("📊 Token ID:", nftState.tokenId.toString());
    console.log("👤 Original owner:", nftState.originalOwner.toString());
    console.log("🔗 Chain origin:", nftState.chainOrigin.toString());
    console.log("🔒 Cross-chain locked:", nftState.isCrossChainLocked);

    // 4. Burn for cross-chain transfer
    console.log("\n4️⃣ Burning NFT for cross-chain transfer...");
    const burnSig = await client.burnForCrossChain(
      user,
      nftMint,
      1, // Ethereum
      "0x742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42"
    );
    console.log("✅ NFT burned for cross-chain:", burnSig);

    // 5. Check updated state
    console.log("\n5️⃣ Checking updated NFT state...");
    const updatedState = await client.getNftState(nftMint);
    console.log("🔒 Cross-chain locked:", updatedState.isCrossChainLocked);
    console.log("📜 Cross-chain history:");
    console.log(client.formatCrossChainHistory(updatedState.crossChainHistory));

    // 6. Get program statistics
    console.log("\n6️⃣ Program statistics...");
    const config = await client.getProgramConfig();
    console.log("📈 Total NFTs minted:", config.totalNftsMinted.toString());
    console.log("🌐 Total cross-chain transfers:", config.totalCrossChainTransfers.toString());
    console.log("🔢 Current nonce:", config.nonce.toString());

    console.log("\n🎉 SDK example completed successfully!");

  } catch (error) {
    console.error("❌ SDK example failed:", error);
    throw error;
  }
}

// Export types for convenience
export type { UniversalNftProgram };