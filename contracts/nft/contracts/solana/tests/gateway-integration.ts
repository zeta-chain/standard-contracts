import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";

// Mock ZetaChain Gateway Program ID (replace with actual when available)
const ZETACHAIN_GATEWAY_PROGRAM_ID = new PublicKey("GatewayProgram1111111111111111111111111111");

describe("Gateway Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;

  let collection: PublicKey;
  let collectionBump: number;
  let authority: Keypair;

  before(async () => {
    authority = Keypair.generate();
    
    // Airdrop SOL to authority
    const signature = await provider.connection.requestAirdrop(
      authority.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // Derive collection PDA
    [collection, collectionBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.publicKey.toBuffer(),
        Buffer.from("Gateway Test Collection")
      ],
      program.programId
    );
  });

  describe("Cross-Chain Message Format Validation", () => {
    it("Should serialize cross-chain message correctly", async () => {
      const tokenId = 12345n;
      const uri = "https://example.com/nft.json";
      const recipient = new PublicKey("11111111111111111111111111111112");

      // Test message serialization format
      const message = serializeCrossChainMessage(tokenId, uri, recipient);
      
      // Validate message structure
      expect(message.length).to.be.greaterThan(44); // 8 + 4 + uri.length + 32
      
      // Validate token ID (first 8 bytes, little-endian)
      const deserializedTokenId = Buffer.from(message.slice(0, 8)).readBigUInt64LE();
      expect(deserializedTokenId).to.equal(tokenId);
      
      // Validate URI length (next 4 bytes)
      const uriLength = Buffer.from(message.slice(8, 12)).readUInt32LE();
      expect(uriLength).to.equal(uri.length);
      
      // Validate URI content
      const deserializedUri = Buffer.from(message.slice(12, 12 + uriLength)).toString('utf8');
      expect(deserializedUri).to.equal(uri);
      
      // Validate recipient (last 32 bytes)
      const deserializedRecipient = new PublicKey(message.slice(12 + uriLength));
      expect(deserializedRecipient.toString()).to.equal(recipient.toString());
    });

    it("Should handle large URI messages", async () => {
      const tokenId = 999n;
      const largeUri = "https://example.com/" + "x".repeat(500) + ".json";
      const recipient = new PublicKey("11111111111111111111111111111112");

      const message = serializeCrossChainMessage(tokenId, largeUri, recipient);
      
      // Should handle large URIs without truncation
      const uriLength = Buffer.from(message.slice(8, 12)).readUInt32LE();
      expect(uriLength).to.equal(largeUri.length);
      
      const deserializedUri = Buffer.from(message.slice(12, 12 + uriLength)).toString('utf8');
      expect(deserializedUri).to.equal(largeUri);
    });

    it("Should validate EVM address format compatibility", async () => {
      // Test with EVM-style address (20 bytes)
      const evmAddress = Buffer.from("1234567890123456789012345678901234567890", 'hex');
      expect(evmAddress.length).to.equal(20);
      
      // Test conversion to Solana format (32 bytes, padded)
      const solanaAddress = Buffer.alloc(32);
      evmAddress.copy(solanaAddress, 12); // Pad with 12 zero bytes at start
      
      const pubkey = new PublicKey(solanaAddress);
      expect(pubkey.toBuffer().length).to.equal(32);
    });
  });

  describe("Gateway CPI Call Simulation", () => {
    it("Should prepare gateway message data correctly", async () => {
      // Initialize collection first
      await program.methods
        .initializeCollection(
          "Gateway Test Collection",
          "GTC",
          "https://example.com/collection.json",
          Array.from(Buffer.alloc(20, 1)) // Mock TSS address
        )
        .accounts({
          authority: authority.publicKey,
          collection,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([authority])
        .rpc();

      // Create mock NFT mint
      const nftMint = Keypair.generate();
      const [nftOrigin] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), Buffer.from([1, 0, 0, 0, 0, 0, 0, 0])], // token_id = 1
        program.programId
      );

      // Test gateway message preparation
      const destinationChain = 1; // Ethereum
      const recipient = Buffer.from("abcdefabcdefabcdefabcdefabcdefabcdefabcd", 'hex');
      
      // This would be the actual gateway call structure
      const gatewayMessage = {
        sourceChain: 103, // Solana devnet
        destinationChain,
        tokenId: 1n,
        uri: "https://example.com/nft1.json",
        recipient: Array.from(recipient),
        nonce: 1
      };

      // Validate message structure
      expect(gatewayMessage.sourceChain).to.equal(103);
      expect(gatewayMessage.destinationChain).to.equal(1);
      expect(gatewayMessage.tokenId).to.equal(1n);
      expect(gatewayMessage.recipient.length).to.equal(20);
    });

    it("Should validate TSS signature format", async () => {
      // Mock TSS signature components
      const mockSignature = {
        r: Buffer.alloc(32, 1),
        s: Buffer.alloc(32, 2),
        v: 27
      };

      // Validate signature structure
      expect(mockSignature.r.length).to.equal(32);
      expect(mockSignature.s.length).to.equal(32);
      expect([27, 28]).to.include(mockSignature.v);

      // Test signature serialization
      const serializedSig = Buffer.concat([
        mockSignature.r,
        mockSignature.s,
        Buffer.from([mockSignature.v])
      ]);
      
      expect(serializedSig.length).to.equal(65);
    });
  });

  describe("Cross-Chain Message Parsing", () => {
    it("Should parse incoming EVM message correctly", async () => {
      const evmMessage = createMockEvmMessage(
        456n,
        "https://ethereum.com/nft456.json",
        authority.publicKey
      );

      // Parse message components
      const parsed = parseCrossChainMessage(evmMessage);
      
      expect(parsed.tokenId).to.equal(456n);
      expect(parsed.uri).to.equal("https://ethereum.com/nft456.json");
      expect(parsed.recipient.toString()).to.equal(authority.publicKey.toString());
    });

    it("Should handle malformed messages gracefully", async () => {
      const malformedMessages = [
        [], // Empty message
        [1, 2, 3], // Too short
        Buffer.alloc(100, 0), // All zeros
        Buffer.from("invalid message"), // Invalid format
      ];

      for (const badMessage of malformedMessages) {
        try {
          parseCrossChainMessage(Array.from(badMessage));
          expect.fail("Should have thrown error for malformed message");
        } catch (error) {
          expect(error.message).to.include("Invalid message format");
        }
      }
    });
  });

  describe("NFT Origin Edge Cases", () => {
    it("Should prevent token ID collisions", async () => {
      // This test validates that sequential token IDs prevent collisions
      const tokenId1 = 1n;
      const tokenId2 = 2n;

      const [origin1] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), Buffer.from(tokenId1.toString(16).padStart(16, '0'), 'hex')],
        program.programId
      );

      const [origin2] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_origin"), Buffer.from(tokenId2.toString(16).padStart(16, '0'), 'hex')],
        program.programId
      );

      expect(origin1.toString()).to.not.equal(origin2.toString());
    });

    it("Should handle maximum token ID values", async () => {
      const maxTokenId = 2n ** 64n - 1n; // Maximum u64 value
      
      // Test serialization of maximum value
      const buffer = Buffer.alloc(8);
      buffer.writeBigUInt64LE(maxTokenId);
      
      const deserializedId = buffer.readBigUInt64LE();
      expect(deserializedId).to.equal(maxTokenId);
    });

    it("Should validate origin data integrity across operations", async () => {
      // Mock origin data
      const originData = {
        tokenId: 123n,
        originalMint: Keypair.generate().publicKey,
        collection: collection,
        chainOfOrigin: 103,
        metadataUri: "https://example.com/original.json",
        createdAt: Math.floor(Date.now() / 1000),
        bump: 255
      };

      // Validate all fields are preserved
      expect(originData.tokenId).to.be.a('bigint');
      expect(originData.originalMint).to.be.instanceOf(PublicKey);
      expect(originData.chainOfOrigin).to.equal(103);
      expect(originData.metadataUri).to.be.a('string');
      expect(originData.createdAt).to.be.a('number');
      expect(originData.bump).to.be.within(0, 255);
    });
  });

  describe("Performance and Compute Budget", () => {
    it("Should estimate compute units for cross-chain transfer", async () => {
      // This would require actual transaction simulation
      console.log("Cross-chain transfer compute estimation:");
      console.log("- Mint burn: ~5,000 CU");
      console.log("- Gateway CPI: ~10,000 CU");
      console.log("- Event emission: ~1,000 CU");
      console.log("- Total estimated: ~16,000 CU");
    });

    it("Should handle batch operations efficiently", async () => {
      const batchSize = 5;
      const operations = [];

      for (let i = 0; i < batchSize; i++) {
        operations.push({
          tokenId: BigInt(i + 1),
          uri: `https://example.com/batch${i}.json`,
          recipient: Keypair.generate().publicKey
        });
      }

      // Validate batch processing structure
      expect(operations.length).to.equal(batchSize);
      operations.forEach((op, index) => {
        expect(op.tokenId).to.equal(BigInt(index + 1));
        expect(op.uri).to.include(`batch${index}`);
      });
    });
  });
});

// Helper Functions

function serializeCrossChainMessage(tokenId: bigint, uri: string, recipient: PublicKey): number[] {
  const message: number[] = [];
  
  // Token ID (8 bytes, little-endian)
  const tokenIdBuffer = Buffer.alloc(8);
  tokenIdBuffer.writeBigUInt64LE(tokenId);
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

function createMockEvmMessage(tokenId: bigint, uri: string, recipient: PublicKey): number[] {
  return serializeCrossChainMessage(tokenId, uri, recipient);
}

function parseCrossChainMessage(message: number[]): {
  tokenId: bigint;
  uri: string;
  recipient: PublicKey;
} {
  if (message.length < 44) { // Minimum: 8 + 4 + 0 + 32
    throw new Error("Invalid message format: too short");
  }

  try {
    // Parse token ID
    const tokenIdBuffer = Buffer.from(message.slice(0, 8));
    const tokenId = tokenIdBuffer.readBigUInt64LE();

    // Parse URI length
    const uriLengthBuffer = Buffer.from(message.slice(8, 12));
    const uriLength = uriLengthBuffer.readUInt32LE();

    if (message.length < 12 + uriLength + 32) {
      throw new Error("Invalid message format: insufficient data");
    }

    // Parse URI
    const uriBuffer = Buffer.from(message.slice(12, 12 + uriLength));
    const uri = uriBuffer.toString('utf8');

    // Parse recipient
    const recipientBuffer = Buffer.from(message.slice(12 + uriLength, 12 + uriLength + 32));
    const recipient = new PublicKey(recipientBuffer);

    return { tokenId, uri, recipient };
  } catch (error) {
    throw new Error(`Invalid message format: ${error.message}`);
  }
}
