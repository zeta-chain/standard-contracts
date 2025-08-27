import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { PublicKey, Keypair, Connection, Commitment } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PROGRAM_ID as METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";

// Global test configuration
declare global {
  namespace NodeJS {
    interface Global {
      provider: anchor.AnchorProvider;
      program: Program<UniversalNft>;
      connection: Connection;
      payer: Keypair;
      testAccounts: {
        recipient: Keypair;
        mint: Keypair;
        metadataPda: PublicKey;
        masterEditionPda: PublicKey;
        nftOriginPda: PublicKey;
        gatewayConfigPda: PublicKey;
        replayMarkerPda: PublicKey;
        recipientTokenAccount: PublicKey;
      };
    }
  }
}

// Test configuration
const TEST_CONFIG = {
  commitment: "confirmed" as Commitment,
  timeout: 60000, // 60 seconds
  retries: 3,
};

// Initialize test environment
beforeAll(async () => {
  // Set up provider
  const connection = new Connection("http://localhost:8899", TEST_CONFIG.commitment);
  const wallet = new anchor.Wallet(Keypair.generate());
  
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    { commitment: TEST_CONFIG.commitment }
  );
  
  anchor.setProvider(provider);
  
  // Load program
  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;
  
  // Generate test accounts
  const payer = Keypair.generate();
  const recipient = Keypair.generate();
  const mint = Keypair.generate();
  
  // Derive PDAs
  const [metadataPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
    METADATA_PROGRAM_ID
  );
  
  const [masterEditionPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer(), Buffer.from("edition")],
    METADATA_PROGRAM_ID
  );
  
  // Generate test token ID
  const tokenId = new Uint8Array(32);
  crypto.getRandomValues(tokenId);
  
  const [nftOriginPda] = PublicKey.findProgramAddressSync(
    [tokenId, Buffer.from("nft_origin")],
    program.programId
  );
  
  const [gatewayConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("gateway_config")],
    program.programId
  );
  
  const [replayMarkerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("replay"), tokenId, new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  
  const [recipientTokenAccount] = PublicKey.findProgramAddressSync(
    [recipient.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  
  // Set up global test context
  global.provider = provider;
  global.program = program;
  global.connection = connection;
  global.payer = payer;
  global.testAccounts = {
    recipient,
    mint,
    metadataPda,
    masterEditionPda,
    nftOriginPda,
    gatewayConfigPda,
    replayMarkerPda,
    recipientTokenAccount,
  };
  
  // Airdrop SOL to payer for testing
  try {
    const signature = await connection.requestAirdrop(payer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await connection.confirmTransaction(signature, TEST_CONFIG.commitment);
    console.log("✅ Airdrop successful for test payer");
  } catch (error) {
    console.warn("⚠️ Airdrop failed, continuing with existing balance:", error);
  }
}, TEST_CONFIG.timeout);

// Clean up after all tests
afterAll(async () => {
  // Clean up any test resources if needed
  console.log("🧹 Test cleanup completed");
});

// Utility functions for tests
export const utils = {
  // Create a test token ID
  createTestTokenId(): Uint8Array {
    const tokenId = new Uint8Array(32);
    crypto.getRandomValues(tokenId);
    return tokenId;
  },
  
  // Serialize cross-chain payload
  serializeCrossChainPayload(payload: {
    tokenId: Uint8Array;
    originChainId: anchor.BN;
    originMint: PublicKey;
    recipient: PublicKey;
    metadataUri: string;
    nonce: anchor.BN;
  }): Buffer {
    // This is a simplified serialization
    // In a real implementation, this would match the Rust struct serialization
    const buffer = Buffer.alloc(0);
    // Add proper serialization logic here
    return buffer;
  },
  
  // Wait for transaction confirmation
  async waitForConfirmation(signature: string, commitment: Commitment = "confirmed"): Promise<void> {
    await global.connection.confirmTransaction(signature, commitment);
  },
  
  // Get account balance
  async getBalance(publicKey: PublicKey): Promise<number> {
    return await global.connection.getBalance(publicKey);
  },
  
  // Create a new keypair for testing
  createTestKeypair(): Keypair {
    return Keypair.generate();
  },
  
  // Derive PDA for testing
  derivePda(seeds: (Buffer | Uint8Array)[], programId: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(seeds, programId);
  },
  
  // Sleep utility
  sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  },
  
  // Retry function with exponential backoff
  async retry<T>(
    fn: () => Promise<T>,
    maxRetries: number = TEST_CONFIG.retries,
    delay: number = 1000
  ): Promise<T> {
    let lastError: Error;
    
    for (let i = 0; i < maxRetries; i++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error as Error;
        if (i < maxRetries - 1) {
          await utils.sleep(delay * Math.pow(2, i));
        }
      }
    }
    
    throw lastError!;
  },
};

// Test helpers
export const testHelpers = {
  // Setup test environment for each test
  async setupTest(): Promise<void> {
    // Additional test setup if needed
  },
  
  // Cleanup after each test
  async cleanupTest(): Promise<void> {
    // Additional test cleanup if needed
  },
  
  // Create test NFT metadata
  createTestMetadata(): {
    name: string;
    symbol: string;
    uri: string;
  } {
    return {
      name: "Test Universal NFT",
      symbol: "TUNFT",
      uri: "https://example.com/metadata.json",
    };
  },
  
  // Validate transaction success
  async validateTransaction(signature: string): Promise<void> {
    const txInfo = await global.connection.getTransaction(signature, {
      commitment: TEST_CONFIG.commitment,
    });
    
    if (txInfo?.meta?.err) {
      throw new Error(`Transaction failed: ${JSON.stringify(txInfo.meta.err)}`);
    }
  },
};

// Export test configuration
export { TEST_CONFIG };
