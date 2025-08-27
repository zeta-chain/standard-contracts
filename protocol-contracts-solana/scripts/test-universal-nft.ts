import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";

describe("Universal NFT Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;
  
  // Test accounts
  const payer = Keypair.generate();
  const recipient = Keypair.generate();
  const mint = Keypair.generate();
  
  // PDAs
  let metadataPda: PublicKey;
  let masterEditionPda: PublicKey;
  let nftOriginPda: PublicKey;
  let gatewayConfigPda: PublicKey;
  let replayMarkerPda: PublicKey;
  let recipientTokenAccount: PublicKey;

  beforeAll(async () => {
    // Airdrop SOL to payer
    const signature = await provider.connection.requestAirdrop(payer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature);

    // Derive PDAs
    [metadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      METADATA_PROGRAM_ID
    );

    [masterEditionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer(), Buffer.from("edition")],
      METADATA_PROGRAM_ID
    );

    // Generate a test token ID
    const tokenId = new Uint8Array(32);
    crypto.getRandomValues(tokenId);

    [nftOriginPda] = PublicKey.findProgramAddressSync(
      [tokenId, Buffer.from("nft_origin")],
      program.programId
    );

    [gatewayConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("gateway_config")],
      program.programId
    );

    [replayMarkerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("replay"), tokenId, new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [recipientTokenAccount] = PublicKey.findProgramAddressSync(
      [recipient.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
  });

  describe("Mint New NFT", () => {
    it("Should mint a new NFT successfully", async () => {
      const metadataUri = "https://example.com/metadata.json";

      try {
        const tx = await program.methods
          .mintNewNft(metadataUri)
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        console.log("Mint transaction signature:", tx);
        
        // Verify the transaction was successful
        const txInfo = await provider.connection.getTransaction(tx, {
          commitment: "confirmed",
        });
        
        expect(txInfo?.meta?.err).toBeNull();
        
      } catch (error) {
        console.error("Mint error:", error);
        throw error;
      }
    });

    it("Should fail with invalid metadata URI length", async () => {
      const longMetadataUri = "a".repeat(201); // Exceeds MAX_URI_LEN

      try {
        await program.methods
          .mintNewNft(longMetadataUri)
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        fail("Should have thrown an error");
      } catch (error) {
        expect(error.toString()).toContain("MetadataTooLong");
      }
    });
  });

  describe("Cross-Chain Transfer", () => {
    it("Should handle incoming cross-chain message", async () => {
      // Create test payload
      const payload = {
        tokenId: new Uint8Array(32),
        originChainId: new anchor.BN(1),
        originMint: new PublicKey("11111111111111111111111111111111"),
        recipient: recipient.publicKey,
        metadataUri: "https://example.com/cross-chain-nft.json",
        nonce: new anchor.BN(1),
      };

      // Serialize payload
      const serializedPayload = Buffer.alloc(0);
      // Add serialization logic here

      try {
        const tx = await program.methods
          .handleIncoming(serializedPayload)
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            gatewayConfig: gatewayConfigPda,
            gatewayProgram: new PublicKey("11111111111111111111111111111111"), // Placeholder
            replayMarker: replayMarkerPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        console.log("Cross-chain transfer transaction signature:", tx);
        
        // Verify the transaction was successful
        const txInfo = await provider.connection.getTransaction(tx, {
          commitment: "confirmed",
        });
        
        expect(txInfo?.meta?.err).toBeNull();
        
      } catch (error) {
        console.error("Cross-chain transfer error:", error);
        throw error;
      }
    });

    it("Should prevent replay attacks", async () => {
      // This test would attempt to reuse the same nonce
      // and should fail with ReplayAttack error
      
      const payload = {
        tokenId: new Uint8Array(32),
        originChainId: new anchor.BN(1),
        originMint: new PublicKey("11111111111111111111111111111111"),
        recipient: recipient.publicKey,
        metadataUri: "https://example.com/cross-chain-nft.json",
        nonce: new anchor.BN(1), // Same nonce as before
      };

      const serializedPayload = Buffer.alloc(0);
      // Add serialization logic here

      try {
        await program.methods
          .handleIncoming(serializedPayload)
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            gatewayConfig: gatewayConfigPda,
            gatewayProgram: new PublicKey("11111111111111111111111111111111"),
            replayMarker: replayMarkerPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        fail("Should have thrown a replay attack error");
      } catch (error) {
        expect(error.toString()).toContain("ReplayAttack");
      }
    });
  });

  describe("Gateway Integration", () => {
    it("Should validate gateway configuration", async () => {
      // Test with invalid gateway config
      const invalidGatewayConfig = Keypair.generate().publicKey;

      try {
        await program.methods
          .handleIncoming(Buffer.alloc(0))
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            gatewayConfig: invalidGatewayConfig,
            gatewayProgram: new PublicKey("11111111111111111111111111111111"),
            replayMarker: replayMarkerPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        fail("Should have thrown an unauthorized gateway error");
      } catch (error) {
        expect(error.toString()).toContain("UnauthorizedGateway");
      }
    });
  });

  describe("PDA Validation", () => {
    it("Should validate NFT origin PDA", async () => {
      const invalidNftOriginPda = Keypair.generate().publicKey;

      try {
        await program.methods
          .mintNewNft("https://example.com/metadata.json")
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: mint.publicKey,
            metadata: metadataPda,
            masterEdition: masterEditionPda,
            recipientTokenAccount: recipientTokenAccount,
            nftOrigin: invalidNftOriginPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, mint])
          .rpc();

        fail("Should have thrown a PDA mismatch error");
      } catch (error) {
        expect(error.toString()).toContain("NftOriginPdaMismatch");
      }
    });
  });
});

// Utility functions
function createTestTokenId(): Uint8Array {
  const tokenId = new Uint8Array(32);
  crypto.getRandomValues(tokenId);
  return tokenId;
}

function serializeCrossChainPayload(payload: any): Buffer {
  // Implement payload serialization
  // This would match the Rust struct serialization
  return Buffer.alloc(0);
}
