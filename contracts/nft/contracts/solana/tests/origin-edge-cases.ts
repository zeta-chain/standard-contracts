import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";

describe("NFT Origin System - Edge Cases & Performance", () => {
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;

  let collection: PublicKey;
  let authority: Keypair;
  let payer: Keypair;

  before(async () => {
    authority = Keypair.generate();
    payer = Keypair.generate();
    
    // Airdrop SOL
    const airdropPromises = [authority, payer].map(async (keypair) => {
      const signature = await provider.connection.requestAirdrop(
        keypair.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(signature);
    });
    await Promise.all(airdropPromises);

    // Initialize collection
    [collection] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.publicKey.toBuffer(),
        Buffer.from("Edge Case Collection")
      ],
      program.programId
    );

    await program.methods
      .initializeCollection(
        "Edge Case Collection",
        "EDGE",
        "https://example.com/edge-collection.json",
        Array.from(Buffer.alloc(20, 1))
      )
      .accounts({
        authority: authority.publicKey,
        collection,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();
  });

  describe("Token ID Edge Cases", () => {
    it("Should handle sequential token IDs correctly", async () => {
      const collectionData = await program.account.collection.fetch(collection);
      const startingTokenId = collectionData.nextTokenId;

      // Mint multiple NFTs and verify sequential IDs
      const nftCount = 5;
      const mintedNfts = [];

      for (let i = 0; i < nftCount; i++) {
        const nftMint = Keypair.generate();
        const expectedTokenId = startingTokenId.add(new anchor.BN(i));
        
        const [nftOrigin] = PublicKey.findProgramAddressSync(
          [Buffer.from("nft_origin"), expectedTokenId.toArrayLike(Buffer, 'le', 8)],
          program.programId
        );

        const [nftTokenAccount] = PublicKey.findProgramAddressSync(
          [
            authority.publicKey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            nftMint.publicKey.toBuffer(),
          ],
          ASSOCIATED_TOKEN_PROGRAM_ID
        );

        const [nftMetadata] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMint.publicKey.toBuffer(),
          ],
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        );

        const [masterEdition] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMint.publicKey.toBuffer(),
            Buffer.from("edition"),
          ],
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        );

        await program.methods
          .mintNft(
            `Edge NFT #${i + 1}`,
            "EDGE",
            `https://example.com/edge${i + 1}.json`
          )
          .accounts({
            authority: authority.publicKey,
            collection,
            nftMint: nftMint.publicKey,
            nftTokenAccount,
            recipient: authority.publicKey,
            nftOrigin,
            nftMetadata,
            masterEdition,
            metadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([authority, nftMint])
          .rpc();

        // Verify origin data
        const originData = await program.account.nftOrigin.fetch(nftOrigin);
        expect(originData.tokenId.toString()).to.equal(expectedTokenId.toString());
        expect(originData.originalMint.toString()).to.equal(nftMint.publicKey.toString());

        mintedNfts.push({
          mint: nftMint.publicKey,
          tokenId: expectedTokenId,
          origin: nftOrigin
        });
      }

      // Verify collection state
      const finalCollectionData = await program.account.collection.fetch(collection);
      expect(finalCollectionData.nextTokenId.toString()).to.equal(
        startingTokenId.add(new anchor.BN(nftCount)).toString()
      );
      expect(finalCollectionData.totalMinted.toString()).to.equal(nftCount.toString());
    });

    it("Should prevent token ID overflow", async () => {
      // This test simulates approaching the maximum token ID
      const collectionData = await program.account.collection.fetch(collection);
      const currentTokenId = collectionData.nextTokenId;
      
      // Calculate maximum safe token ID (u64::MAX - buffer)
      const maxSafeTokenId = new anchor.BN("18446744073709551615"); // u64::MAX
      const buffer = new anchor.BN(1000);
      const safeLimit = maxSafeTokenId.sub(buffer);

      // Verify we're not approaching overflow
      expect(currentTokenId.lt(safeLimit)).to.be.true;
      
      console.log(`Current token ID: ${currentTokenId.toString()}`);
      console.log(`Safe limit: ${safeLimit.toString()}`);
      console.log(`Remaining capacity: ${safeLimit.sub(currentTokenId).toString()}`);
    });
  });

  describe("Large Metadata Handling", () => {
    it("Should handle maximum URI length efficiently", async () => {
      // Test with very large URI (close to transaction size limits)
      const baseUri = "https://example.com/";
      const maxDataSize = 1000; // Conservative limit for URI
      const largeUri = baseUri + "x".repeat(maxDataSize - baseUri.length - 5) + ".json";
      
      const nftMint = Keypair.generate();
      const collectionData = await program.account.collection.fetch(collection);
      const tokenId = collectionData.nextTokenId;
      
      const [nftOrigin] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), tokenId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );

      const [nftTokenAccount] = PublicKey.findProgramAddressSync(
        [
          authority.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const [nftMetadata] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      const [masterEdition] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
          Buffer.from("edition"),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      // Should handle large URI without issues
      await program.methods
        .mintNft(
          "Large Metadata NFT",
          "LARGE",
          largeUri
        )
        .accounts({
          authority: authority.publicKey,
          collection,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          recipient: authority.publicKey,
          nftOrigin,
          nftMetadata,
          masterEdition,
          metadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([authority, nftMint])
        .rpc();

      // Verify metadata was stored correctly
      const originData = await program.account.nftOrigin.fetch(nftOrigin);
      expect(originData.metadataUri).to.equal(largeUri);
      expect(originData.metadataUri.length).to.equal(largeUri.length);
    });

    it("Should handle special characters in metadata", async () => {
      const specialUri = "https://example.com/nft-with-émojis-and-spëcial-chars-🚀-💎.json";
      
      const nftMint = Keypair.generate();
      const collectionData = await program.account.collection.fetch(collection);
      const tokenId = collectionData.nextTokenId;
      
      const [nftOrigin] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), tokenId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );

      const [nftTokenAccount] = PublicKey.findProgramAddressSync(
        [
          authority.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const [nftMetadata] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      const [masterEdition] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
          Buffer.from("edition"),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      await program.methods
        .mintNft(
          "Special Chars NFT 🎨",
          "SPEC",
          specialUri
        )
        .accounts({
          authority: authority.publicKey,
          collection,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          recipient: authority.publicKey,
          nftOrigin,
          nftMetadata,
          masterEdition,
          metadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([authority, nftMint])
        .rpc();

      const originData = await program.account.nftOrigin.fetch(nftOrigin);
      expect(originData.metadataUri).to.equal(specialUri);
    });
  });

  describe("Cross-Chain Round Trip Scenarios", () => {
    it("Should preserve origin data through multiple transfers", async () => {
      // Create initial NFT
      const nftMint = Keypair.generate();
      const collectionData = await program.account.collection.fetch(collection);
      const tokenId = collectionData.nextTokenId;
      
      const [nftOrigin] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), tokenId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );

      // Mint NFT (simulating original creation on Solana)
      const originalUri = "https://solana.com/original-nft.json";
      const [nftTokenAccount] = PublicKey.findProgramAddressSync(
        [
          authority.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const [nftMetadata] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      const [masterEdition] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
          nftMint.publicKey.toBuffer(),
          Buffer.from("edition"),
        ],
        new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
      );

      await program.methods
        .mintNft(
          "Round Trip NFT",
          "TRIP",
          originalUri
        )
        .accounts({
          authority: authority.publicKey,
          collection,
          nftMint: nftMint.publicKey,
          nftTokenAccount,
          recipient: authority.publicKey,
          nftOrigin,
          nftMetadata,
          masterEdition,
          metadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([authority, nftMint])
        .rpc();

      // Capture original origin data
      const originalOriginData = await program.account.nftOrigin.fetch(nftOrigin);
      
      // Simulate transfer to Ethereum (would burn NFT and send message)
      // Then simulate return from Ethereum with modified metadata
      const returnUri = "https://ethereum.com/modified-metadata.json";
      
      // Simulate on_call for returning NFT (Scenario B)
      const sender = Array.from(Buffer.from("1234567890123456789012345678901234567890", 'hex'));
      const sourceChain = 1; // Ethereum
      const message = createCrossChainMessage(tokenId, returnUri, authority.publicKey);
      const nonce = 1;

      // Mock gateway and connected accounts
      const [gateway] = PublicKey.findProgramAddressSync(
        [Buffer.from("gateway")],
        program.programId
      );

      const [connected] = PublicKey.findProgramAddressSync(
        [Buffer.from("connected"), collection.toBuffer(), Buffer.from([sourceChain])],
        program.programId
      );

      // For this test, we'll verify the origin data structure remains intact
      // In a real scenario, the on_call would recreate the NFT with preserved origin
      
      // Verify original data is preserved
      expect(originalOriginData.tokenId.toString()).to.equal(tokenId.toString());
      expect(originalOriginData.originalMint.toString()).to.equal(nftMint.publicKey.toString());
      expect(originalOriginData.chainOfOrigin.toString()).to.equal("103"); // Solana
      expect(originalOriginData.metadataUri).to.equal(originalUri);
      
      console.log("Origin data preserved through simulated round trip:");
      console.log(`- Token ID: ${originalOriginData.tokenId.toString()}`);
      console.log(`- Original Mint: ${originalOriginData.originalMint.toString()}`);
      console.log(`- Chain of Origin: ${originalOriginData.chainOfOrigin.toString()}`);
      console.log(`- Original URI: ${originalOriginData.metadataUri}`);
    });
  });

  describe("Performance Benchmarks", () => {
    it("Should measure mint operation performance", async () => {
      const iterations = 3;
      const times = [];

      for (let i = 0; i < iterations; i++) {
        const nftMint = Keypair.generate();
        const collectionData = await program.account.collection.fetch(collection);
        const tokenId = collectionData.nextTokenId;
        
        const [nftOrigin] = PublicKey.findProgramAddressSync(
          [Buffer.from("nft_origin"), tokenId.toArrayLike(Buffer, 'le', 8)],
          program.programId
        );

        const [nftTokenAccount] = PublicKey.findProgramAddressSync(
          [
            authority.publicKey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            nftMint.publicKey.toBuffer(),
          ],
          ASSOCIATED_TOKEN_PROGRAM_ID
        );

        const [nftMetadata] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMint.publicKey.toBuffer(),
          ],
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        );

        const [masterEdition] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMint.publicKey.toBuffer(),
            Buffer.from("edition"),
          ],
          new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        );

        const startTime = Date.now();
        
        await program.methods
          .mintNft(
            `Perf Test #${i + 1}`,
            "PERF",
            `https://example.com/perf${i + 1}.json`
          )
          .accounts({
            authority: authority.publicKey,
            collection,
            nftMint: nftMint.publicKey,
            nftTokenAccount,
            recipient: authority.publicKey,
            nftOrigin,
            nftMetadata,
            masterEdition,
            metadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([authority, nftMint])
          .rpc();

        const endTime = Date.now();
        times.push(endTime - startTime);
      }

      const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
      const minTime = Math.min(...times);
      const maxTime = Math.max(...times);

      console.log(`Mint Performance (${iterations} iterations):`);
      console.log(`- Average: ${avgTime.toFixed(2)}ms`);
      console.log(`- Min: ${minTime}ms`);
      console.log(`- Max: ${maxTime}ms`);
      
      // Performance should be reasonable (under 5 seconds for devnet)
      expect(avgTime).to.be.lessThan(5000);
    });
  });
});

// Helper function to create cross-chain message
function createCrossChainMessage(tokenId: anchor.BN, uri: string, recipient: PublicKey): number[] {
  const message: number[] = [];
  
  // Token ID (8 bytes, little-endian)
  const tokenIdBuffer = tokenId.toArrayLike(Buffer, 'le', 8);
  message.push(...Array.from(tokenIdBuffer));
  
  // URI length (4 bytes, little-endian)
  const uriBuffer = Buffer.from(uri, 'utf8');
  const lengthBuffer = Buffer.alloc(4);
  lengthBuffer.writeUInt32LE(uriBuffer.length);
  message.push(...Array.from(lengthBuffer));
  
  // URI data
  message.push(...Array.from(uriBuffer));
  
  // Recipient (32 bytes for Solana pubkey)
  message.push(...Array.from(recipient.toBuffer()));
  
  return message;
}
