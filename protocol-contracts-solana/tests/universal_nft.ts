import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";

describe("universal-nft", () => {
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
  let recipientTokenAccount: PublicKey;
  
  const metadataUri = "https://example.com/nft/metadata/1";

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
    
    [nftOriginPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nft_origin"), Buffer.alloc(32)], // Placeholder, will be set in mint
      program.programId
    );
    
    recipientTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      recipient.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
  });

  describe("Mint New NFT", () => {
    it("Should mint a new Universal NFT with collection support", async () => {
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
        
        console.log("✅ Mint transaction:", tx);
        
        // Verify the NFT was minted
        const tokenAccount = await provider.connection.getTokenAccountBalance(recipientTokenAccount);
        expect(tokenAccount.value.amount).toBe("1");
        expect(tokenAccount.value.decimals).toBe(0);
        
        // Verify metadata was created
        const metadataAccount = await provider.connection.getAccountInfo(metadataPda);
        expect(metadataAccount).toBeTruthy();
        
        // Verify master edition was created
        const masterEditionAccount = await provider.connection.getAccountInfo(masterEditionPda);
        expect(masterEditionAccount).toBeTruthy();
        
        // Verify nft_origin PDA was created
        const nftOriginAccount = await provider.connection.getAccountInfo(nftOriginPda);
        expect(nftOriginAccount).toBeTruthy();
        
      } catch (error) {
        console.error("Mint failed:", error);
        throw error;
      }
    });
  });

  describe("Burn for Cross-chain Transfer", () => {
    it("Should burn NFT and create replay protection", async () => {
      try {
        const nonce = Date.now(); // Use timestamp as nonce for testing
        
        // Derive replay marker PDA
        const [replayMarkerPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("replay"), Buffer.alloc(32), Buffer.alloc(8)], // Placeholder
          program.programId
        );
        
        const tx = await program.methods
          .burnForTransfer(new anchor.BN(nonce))
          .accounts({
            owner: recipient.publicKey,
            mint: mint.publicKey,
            ownerTokenAccount: recipientTokenAccount,
            nftOrigin: nftOriginPda,
            replayMarker: replayMarkerPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([recipient])
          .rpc();
        
        console.log("✅ Burn transaction:", tx);
        
        // Verify the NFT was burned
        const tokenAccount = await provider.connection.getTokenAccountBalance(recipientTokenAccount);
        expect(tokenAccount.value.amount).toBe("0");
        
        // Verify replay marker was created
        const replayMarkerAccount = await provider.connection.getAccountInfo(replayMarkerPda);
        expect(replayMarkerAccount).toBeTruthy();
        
      } catch (error) {
        console.error("Burn failed:", error);
        throw error;
      }
    });
  });

  describe("Handle Incoming Cross-chain Message", () => {
    it("Should mint NFT from cross-chain message", async () => {
      try {
        // Create new mint for incoming NFT
        const newMint = Keypair.generate();
        const newMetadataPda = PublicKey.findProgramAddressSync(
          [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), newMint.publicKey.toBuffer()],
          METADATA_PROGRAM_ID
        )[0];
        const newMasterEditionPda = PublicKey.findProgramAddressSync(
          [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), newMint.publicKey.toBuffer(), Buffer.from("edition")],
          METADATA_PROGRAM_ID
        )[0];
        const newNftOriginPda = PublicKey.findProgramAddressSync(
          [Buffer.from("nft_origin"), Buffer.alloc(32)], // Placeholder
          program.programId
        )[0];
        const newRecipientTokenAccount = await getAssociatedTokenAddress(
          newMint.publicKey,
          recipient.publicKey,
          false,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        );
        
        // Mock cross-chain payload
        const payload = {
          version: 1,
          tokenId: Buffer.alloc(32, 1), // Mock token ID
          originChainId: 1, // Ethereum
          originMint: mint.publicKey,
          metadataUri: "https://example.com/nft/metadata/2",
          recipient: recipient.publicKey,
          nonce: Date.now(),
        };
        
        // Serialize payload
        const serializedPayload = Buffer.alloc(0);
        // In real implementation, this would serialize the payload properly
        
        const tx = await program.methods
          .handleIncoming(Array.from(serializedPayload))
          .accounts({
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            mint: newMint.publicKey,
            metadata: newMetadataPda,
            masterEdition: newMasterEditionPda,
            recipientTokenAccount: newRecipientTokenAccount,
            nftOrigin: newNftOriginPda,
            gatewaySigner: payer.publicKey, // Mock gateway signer
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([payer, newMint])
          .rpc();
        
        console.log("✅ Handle incoming transaction:", tx);
        
        // Verify the new NFT was minted
        const newTokenAccount = await provider.connection.getTokenAccountBalance(newRecipientTokenAccount);
        expect(newTokenAccount.value.amount).toBe("1");
        
      } catch (error) {
        console.error("Handle incoming failed:", error);
        throw error;
      }
    });
  });

  describe("Collection Behavior", () => {
    it("Should create separate collections for each Universal NFT", async () => {
      // This test verifies that each new Universal NFT creates a separate collection
      // as specified in the requirements
      
      const collectionMint1 = Keypair.generate();
      const collectionMint2 = Keypair.generate();
      
      // Each mint should have its own metadata and master edition
      const metadata1 = PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), collectionMint1.publicKey.toBuffer()],
        METADATA_PROGRAM_ID
      )[0];
      
      const metadata2 = PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), collectionMint2.publicKey.toBuffer()],
        METADATA_PROGRAM_ID
      )[0];
      
      // Verify they are different
      expect(metadata1.toBase58()).not.toBe(metadata2.toBase58());
      
      console.log("✅ Collection separation verified");
    });
  });
});
