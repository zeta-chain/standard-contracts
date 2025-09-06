import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { UniversalNftClient } from "./client-integration";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";

describe("Universal NFT - Core Scenarios", () => {
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;
  const client = new UniversalNftClient(program, provider);

  let collection: PublicKey;
  let mint: PublicKey;
  let nftOrigin: PublicKey;

  describe("NFT Origin System Tests", () => {
    it("Should initialize collection with proper configuration", async () => {
      const result = await client.initializeCollection(
        "Test Collection",
        "TEST",
        "https://test.com/collection.json",
        Array.from(Buffer.alloc(20, 1)) // Mock TSS address
      );

      collection = result.collection;
      expect(result.signature).to.be.a('string');

      // Verify collection data
      const collectionData = await client.getCollection(collection);
      expect(collectionData.name).to.equal("Test Collection");
      expect(collectionData.symbol).to.equal("TEST");
      expect(collectionData.nextTokenId.toNumber()).to.equal(1);
    });

    it("Should mint NFT with proper Origin PDA creation", async () => {
      const result = await client.mintNft(
        collection,
        "Test NFT #1",
        "TEST",
        "https://test.com/nft1.json"
      );

      mint = result.mint;
      nftOrigin = result.nftOrigin;

      // Verify NFT Origin data
      const originData = await client.getNftOrigin(nftOrigin);
      expect(originData.originalMint.toString()).to.equal(mint.toString());
      expect(originData.collection.toString()).to.equal(collection.toString());
      expect(originData.chainOfOrigin.toNumber()).to.equal(103); // Solana devnet
      expect(originData.metadataUri).to.equal("https://test.com/nft1.json");

      // Verify collection stats updated
      const collectionData = await client.getCollection(collection);
      expect(collectionData.nextTokenId.toNumber()).to.equal(2);
      expect(collectionData.totalMinted.toNumber()).to.equal(1);
    });

    it("Should handle cross-chain transfer with proper Origin tracking", async () => {
      const destinationChain = 1; // Ethereum
      const recipient = Array.from(Buffer.from("0x1234567890123456789012345678901234567890", 'hex'));

      const signature = await client.transferCrossChain(
        collection,
        mint,
        destinationChain,
        recipient
      );

      expect(signature).to.be.a('string');

      // Verify Origin data is preserved
      const originData = await client.getNftOrigin(nftOrigin);
      expect(originData.originalMint.toString()).to.equal(mint.toString());
      expect(originData.chainOfOrigin.toNumber()).to.equal(103); // Still Solana
    });

    it("Should set connected contracts correctly", async () => {
      const chainId = [1]; // Ethereum
      const contractAddress = Array.from(Buffer.from("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd", 'hex'));

      const signature = await client.setConnected(
        collection,
        chainId,
        contractAddress
      );

      expect(signature).to.be.a('string');

      // Derive and verify connected PDA
      const [connectedPda] = UniversalNftClient.deriveConnectedPda(
        collection,
        chainId
      );

      const connectedData = await client.getConnected(connectedPda);
      expect(connectedData.collection.toString()).to.equal(collection.toString());
      expect(connectedData.chainId).to.deep.equal(chainId);
      expect(connectedData.contractAddress).to.deep.equal(contractAddress);
    });
  });

  describe("Cross-Chain Message Handling", () => {
    it("Should handle incoming NFT from another chain (Scenario A: New NFT)", async () => {
      // Simulate message from Ethereum
      const sender = Array.from(Buffer.from("0x9876543210987654321098765432109876543210", 'hex'));
      const sourceChainId = 1; // Ethereum
      const tokenId = 12345n;
      const uri = "https://ethereum.com/nft12345.json";
      
      // Create mock cross-chain message
      const message = createCrossChainMessage(tokenId, uri, provider.wallet.publicKey);
      const nonce = 1;

      const signature = await client.onCall(
        collection,
        sender,
        sourceChainId,
        message,
        nonce
      );

      expect(signature).to.be.a('string');

      // Verify new Origin PDA was created
      const [newOriginPda] = UniversalNftClient.deriveNftOriginPda(tokenId);
      const originData = await client.getNftOrigin(newOriginPda);
      
      expect(originData.chainOfOrigin.toNumber()).to.equal(sourceChainId);
      expect(originData.metadataUri).to.equal(uri);
    });

    it("Should handle returning NFT to Solana (Scenario B: Existing Origin)", async () => {
      // This would test the case where an NFT originally from Solana
      // is returning from another chain
      const sender = Array.from(Buffer.from("0x1111111111111111111111111111111111111111", 'hex'));
      const sourceChainId = 56; // BSC
      
      // Use the token ID from our original mint
      const originalOriginData = await client.getNftOrigin(nftOrigin);
      const tokenId = originalOriginData.tokenId;
      
      const message = createCrossChainMessage(
        tokenId, 
        "https://bsc.com/returned-nft.json", 
        provider.wallet.publicKey
      );
      const nonce = 2;

      const signature = await client.onCall(
        collection,
        sender,
        sourceChainId,
        message,
        nonce
      );

      expect(signature).to.be.a('string');

      // Verify the original Origin PDA still exists and is correct
      const updatedOriginData = await client.getNftOrigin(nftOrigin);
      expect(updatedOriginData.originalMint.toString()).to.equal(mint.toString());
      expect(updatedOriginData.chainOfOrigin.toNumber()).to.equal(103); // Still Solana
    });
  });

  describe("Edge Cases and Error Handling", () => {
    it("Should prevent duplicate collection names", async () => {
      try {
        await client.initializeCollection(
          "Test Collection", // Same name as before
          "TEST2",
          "https://test.com/collection2.json",
          Array.from(Buffer.alloc(20, 2))
        );
        expect.fail("Should have thrown an error for duplicate collection name");
      } catch (error) {
        expect(error.message).to.include("already in use");
      }
    });

    it("Should handle invalid cross-chain messages gracefully", async () => {
      const sender = Array.from(Buffer.alloc(20, 3));
      const sourceChainId = 999; // Invalid chain
      const invalidMessage = [1, 2, 3]; // Too short
      const nonce = 3;

      try {
        await client.onCall(
          collection,
          sender,
          sourceChainId,
          invalidMessage,
          nonce
        );
        expect.fail("Should have thrown an error for invalid message");
      } catch (error) {
        expect(error.message).to.include("Invalid");
      }
    });

    it("Should prevent replay attacks with nonce validation", async () => {
      const sender = Array.from(Buffer.alloc(20, 4));
      const sourceChainId = 1;
      const message = createCrossChainMessage(
        999n, 
        "https://test.com/replay.json", 
        provider.wallet.publicKey
      );
      const oldNonce = 1; // Reusing old nonce

      try {
        await client.onCall(
          collection,
          sender,
          sourceChainId,
          message,
          oldNonce
        );
        expect.fail("Should have thrown an error for replay attack");
      } catch (error) {
        expect(error.message).to.include("nonce");
      }
    });
  });

  describe("PDA Derivation Validation", () => {
    it("Should derive consistent PDAs for same inputs", async () => {
      const authority = provider.wallet.publicKey;
      const name = "Consistent Collection";
      
      // Derive PDA multiple times
      const [pda1] = UniversalNftClient.deriveCollectionPda(authority, name);
      const [pda2] = UniversalNftClient.deriveCollectionPda(authority, name);
      
      expect(pda1.toString()).to.equal(pda2.toString());
    });

    it("Should derive different PDAs for different token IDs", async () => {
      const [pda1] = UniversalNftClient.deriveNftOriginPda(123n);
      const [pda2] = UniversalNftClient.deriveNftOriginPda(456n);
      
      expect(pda1.toString()).to.not.equal(pda2.toString());
    });

    it("Should derive different PDAs for different chains", async () => {
      const [pda1] = UniversalNftClient.deriveConnectedPda(collection, [1]);
      const [pda2] = UniversalNftClient.deriveConnectedPda(collection, [56]);
      
      expect(pda1.toString()).to.not.equal(pda2.toString());
    });
  });
});

// Helper function to create mock cross-chain messages
function createCrossChainMessage(
  tokenId: bigint, 
  uri: string, 
  recipient: PublicKey
): number[] {
  const message: number[] = [];
  
  // Token ID (8 bytes, little-endian)
  const tokenIdBuffer = Buffer.alloc(8);
  tokenIdBuffer.writeBigUInt64LE(tokenId);
  message.push(...Array.from(tokenIdBuffer));
  
  // URI length (4 bytes)
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

// Performance and gas estimation tests
describe("Performance Tests", () => {
  it("Should estimate compute units for mint operation", async () => {
    // This would require simulation to get compute unit usage
    console.log("Mint operation compute estimation would go here");
  });

  it("Should handle large metadata URIs efficiently", async () => {
    const largeUri = "https://example.com/" + "x".repeat(200) + ".json";
    
    // Test with large URI
    const result = await client.mintNft(
      collection,
      "Large Metadata NFT",
      "LARGE",
      largeUri
    );
    
    expect(result.signature).to.be.a('string');
  });
});
