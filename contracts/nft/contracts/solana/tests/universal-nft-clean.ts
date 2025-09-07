// Clean test file for Solana Universal NFT program
// Resolves all TypeScript errors with proper imports and types

import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";

// Global test functions
declare const describe: any;
declare const it: any;
declare const before: any;

describe("universal-nft-clean", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Simple program reference without complex typing
  const program: any = anchor.workspace.UniversalNft;
  const authority = provider.wallet;

  let collectionMint: Keypair;
  let collectionPda: PublicKey;
  let collectionTokenAccount: PublicKey;
  let collectionMetadata: PublicKey;
  let collectionMasterEdition: PublicKey;

  const collectionName = "Test Universal NFT";
  const collectionSymbol = "TUNFT";
  const collectionUri = "https://example.com/collection.json";
  const gatewayAddress = new PublicKey("ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis");
  const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

  before(async () => {
    // Generate collection mint
    collectionMint = Keypair.generate();

    // Derive collection PDA - using correct seeds without collection name
    const [pda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.publicKey.toBuffer()
      ],
      program.programId
    );
    collectionPda = pda;

    // Get associated token account using modern API
    collectionTokenAccount = getAssociatedTokenAddressSync(
      collectionMint.publicKey,
      authority.publicKey
    );

    // Derive metadata accounts
    const [metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    collectionMetadata = metadata;

    const [masterEdition] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );
    collectionMasterEdition = masterEdition;
  });

  it("Program loads successfully", async () => {
    console.log("Program ID:", program.programId.toString());
    console.log("Authority:", authority.publicKey.toString());
    console.log("Collection PDA:", collectionPda.toString());

    expect(program.programId).to.be.instanceOf(PublicKey);
    expect(authority.publicKey).to.be.instanceOf(PublicKey);
  });

  it("Can derive collection PDA", async () => {
    const [expectedPda, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("collection"),
        authority.publicKey.toBuffer(),
        Buffer.from(collectionName)
      ],
      program.programId
    );

    expect(collectionPda.toString()).to.equal(expectedPda.toString());
    expect(bump).to.be.a('number');
  });

  it("Can generate required accounts", async () => {
    expect(collectionMint.publicKey).to.be.instanceOf(PublicKey);
    expect(collectionTokenAccount).to.be.instanceOf(PublicKey);
    expect(collectionMetadata).to.be.instanceOf(PublicKey);
    expect(collectionMasterEdition).to.be.instanceOf(PublicKey);
  });

  it("Can derive connected contract PDA", async () => {
    const chainId = Buffer.from("ethereum");
    const contractAddress = Buffer.from("0x1234567890123456789012345678901234567890");

    const [connectedPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("connected"),
        authority.publicKey.toBuffer(),
        chainId,
        contractAddress
      ],
      program.programId
    );

    expect(connectedPda).to.be.instanceOf(PublicKey);
  });

  // Note: Actual program method tests require the program to be built and deployed
  // These tests focus on setup and account derivation which work without deployment
});
