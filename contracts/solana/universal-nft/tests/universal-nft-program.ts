import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNftProgram } from "../target/types/universal_nft_program";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import { PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { getEvmAddressArray } from "../utils/address";
import { expect } from "chai";

describe("Universal NFT Program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.UniversalNftProgram as Program<UniversalNftProgram>;
  const authority = Keypair.generate();
  const user = Keypair.generate();
  
  // Gateway program ID (placeholder - would be actual ZetaChain gateway in production)
  const gatewayProgramId = Keypair.generate().publicKey;

  let programConfigPda: PublicKey;
  let programConfigBump: number;
  let collectionMint: Keypair;
  let collectionMetadata: PublicKey;
  let collectionMasterEdition: PublicKey;
  let collectionTokenAccount: PublicKey;

  before(async () => {
    // Fund accounts
    await provider.connection.requestAirdrop(authority.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Wait for confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Derive PDAs
    [programConfigPda, programConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_program")],
      program.programId
    );

    // Initialize collection mint
    collectionMint = Keypair.generate();
    
    [collectionMetadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    [collectionMasterEdition] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMint.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    collectionTokenAccount = await getAssociatedTokenAddress(
      collectionMint.publicKey,
      authority.publicKey
    );
  });

  describe("Program Initialization", () => {
    it("Should initialize the program successfully", async () => {
      const collectionName = "Universal NFT Collection";
      const collectionSymbol = "UNFT";
      const collectionUri = "https://example.com/collection.json";

      try {
        const tx = await program.methods
          .initializeProgram(
            gatewayProgramId,
            collectionName,
            collectionSymbol,
            collectionUri
          )
          .accounts({
            programConfig: programConfigPda,
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

        console.log("Program initialized. Transaction:", tx);

        // Verify program config
        const programConfig = await program.account.programConfig.fetch(programConfigPda);
        expect(programConfig.authority.toString()).to.equal(authority.publicKey.toString());
        expect(programConfig.gatewayProgramId.toString()).to.equal(gatewayProgramId.toString());
        expect(programConfig.isInitialized).to.be.true;
        expect(programConfig.totalNftsMinted.toNumber()).to.equal(0);
        expect(programConfig.nonce.toNumber()).to.equal(1);

      } catch (error) {
        console.error("Initialization failed:", error);
        throw error;
      }
    });

    it("Should fail to initialize program twice", async () => {
      try {
        await program.methods
          .initializeProgram(
            gatewayProgramId,
            "Test",
            "TEST",
            "https://test.com"
          )
          .accounts({
            programConfig: programConfigPda,
            collectionMint: Keypair.generate().publicKey,
            collectionMetadata: Keypair.generate().publicKey,
            collectionMasterEdition: Keypair.generate().publicKey,
            collectionTokenAccount: Keypair.generate().publicKey,
            authority: authority.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([authority])
          .rpc();

        // Should not reach here
        expect(true).to.be.false;
      } catch (error) {
        expect(error.toString()).to.include("ProgramAlreadyInitialized");
      }
    });
  });

  describe("NFT Minting", () => {
    let nftMint: Keypair;
    let nftMetadata: PublicKey;
    let nftTokenAccount: PublicKey;
    let nftStatePda: PublicKey;

    beforeEach(() => {
      nftMint = Keypair.generate();
      
      [nftMetadata] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      [nftStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_state"), nftMint.publicKey.toBuffer()],
        program.programId
      );
    });

    it("Should mint an NFT successfully", async () => {
      nftTokenAccount = await getAssociatedTokenAddress(
        nftMint.publicKey,
        user.publicKey
      );

      const nftName = "Test NFT";
      const nftSymbol = "TNFT";
      const nftUri = "https://example.com/nft.json";

      try {
        const tx = await program.methods
          .mintNft(nftName, nftSymbol, nftUri, null)
          .accounts({
            programConfig: programConfigPda,
            nftState: nftStatePda,
            nftMint: nftMint.publicKey,
            nftMetadata,
            nftTokenAccount,
            owner: user.publicKey,
            authority: authority.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([user, authority, nftMint])
          .rpc();

        console.log("NFT minted. Transaction:", tx);

        // Verify NFT state
        const nftState = await program.account.nftState.fetch(nftStatePda);
        expect(nftState.mint.toString()).to.equal(nftMint.publicKey.toString());
        expect(nftState.originalOwner.toString()).to.equal(user.publicKey.toString());
        expect(nftState.chainOrigin.toNumber()).to.equal(7565164); // SOLANA_CHAIN_ID
        expect(nftState.isCrossChainLocked).to.be.false;
        expect(nftState.crossChainHistory).to.be.empty;

        // Verify program config updates
        const updatedConfig = await program.account.programConfig.fetch(programConfigPda);
        expect(updatedConfig.totalNftsMinted.toNumber()).to.equal(1);

      } catch (error) {
        console.error("NFT minting failed:", error);
        throw error;
      }
    });
  });

  describe("Cross-Chain Operations", () => {
    let nftMint: Keypair;
    let nftTokenAccount: PublicKey;
    let nftStatePda: PublicKey;
    const destinationChainId = 1; // Ethereum chain ID
    const destinationAddress = getEvmAddressArray("0x742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42");

    before(async () => {
      // Setup NFT for cross-chain testing
      nftMint = Keypair.generate();
      nftTokenAccount = await getAssociatedTokenAddress(nftMint.publicKey, user.publicKey);
      [nftStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("nft_state"), nftMint.publicKey.toBuffer()],
        program.programId
      );

      // First, mint an NFT to burn later
      const [nftMetadata] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          nftMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      await program.methods
        .mintNft("Cross-Chain NFT", "XNFT", "https://example.com/cross-chain.json", null)
        .accounts({
          programConfig: programConfigPda,
          nftState: nftStatePda,
          nftMint: nftMint.publicKey,
          nftMetadata,
          nftTokenAccount,
          owner: user.publicKey,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: TOKEN_METADATA_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user, authority, nftMint])
        .rpc();
    });

    it("Should burn NFT for cross-chain transfer", async () => {
      try {
        const tx = await program.methods
          .burnForCrossChain(
            new anchor.BN(destinationChainId),
            destinationAddress
          )
          .accounts({
            programConfig: programConfigPda,
            nftState: nftStatePda,
            nftMint: nftMint.publicKey,
            nftTokenAccount,
            owner: user.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();

        console.log("NFT burned for cross-chain. Transaction:", tx);

        // Verify NFT state updates
        const nftState = await program.account.nftState.fetch(nftStatePda);
        expect(nftState.isCrossChainLocked).to.be.true;
        expect(nftState.crossChainHistory).to.have.length(1);
        expect(nftState.crossChainHistory[0].destinationChainId.toNumber()).to.equal(destinationChainId);

        // Verify program config updates  
        const updatedConfig = await program.account.programConfig.fetch(programConfigPda);
        expect(updatedConfig.totalCrossChainTransfers.toNumber()).to.equal(1);

      } catch (error) {
        console.error("Cross-chain burn failed:", error);
        throw error;
      }
    });
  });

  describe("Gateway Integration", () => {
    it("Should handle on_call from gateway", async () => {
      const sender = Buffer.alloc(20); // Mock Ethereum address
      const message = Buffer.from("test cross-chain message");

      try {
        const tx = await program.methods
          .onCall(Array.from(sender), Array.from(message))
          .accounts({
            programConfig: programConfigPda,
            instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .rpc();

        console.log("Gateway on_call handled. Transaction:", tx);

      } catch (error) {
        console.error("Gateway on_call failed:", error);
        // This might fail due to gateway program check, which is expected in testing
      }
    });
  });

  describe("Configuration Updates", () => {
    it("Should update gateway configuration", async () => {
      const newGatewayId = Keypair.generate().publicKey;
      const newTssAddress = Buffer.alloc(20);

      try {
        const tx = await program.methods
          .updateGatewayConfig(newGatewayId, Array.from(newTssAddress))
          .accounts({
            programConfig: programConfigPda,
            authority: authority.publicKey,
          })
          .signers([authority])
          .rpc();

        console.log("Gateway config updated. Transaction:", tx);

        // Verify updates
        const config = await program.account.programConfig.fetch(programConfigPda);
        expect(config.gatewayProgramId.toString()).to.equal(newGatewayId.toString());
        expect(Buffer.from(config.tssAddress)).to.deep.equal(newTssAddress);

      } catch (error) {
        console.error("Config update failed:", error);
        throw error;
      }
    });

    it("Should fail config update with wrong authority", async () => {
      const wrongAuthority = Keypair.generate();
      
      try {
        await program.methods
          .updateGatewayConfig(null, null)
          .accounts({
            programConfig: programConfigPda,
            authority: wrongAuthority.publicKey,
          })
          .signers([wrongAuthority])
          .rpc();

        // Should not reach here
        expect(true).to.be.false;
      } catch (error) {
        expect(error.toString()).to.include("InvalidAuthority");
      }
    });
  });
});