import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNftProgram } from "../target/types/universal_nft_program";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Connection,
  clusterApiUrl,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";

async function deployUniversalNftProgram() {
  console.log("🚀 Starting Universal NFT Program deployment...");

  // Setup connection
  const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
  
  // Setup wallet (you should use your actual keypair)
  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );
  anchor.setProvider(provider);

  const program = anchor.workspace.UniversalNftProgram as Program<UniversalNftProgram>;
  
  console.log("📋 Program ID:", program.programId.toString());
  console.log("👤 Authority:", wallet.publicKey.toString());

  // ZetaChain gateway program ID (replace with actual gateway program ID for devnet)
  const gatewayProgramId = new PublicKey("GatewayProgramIdPlaceholder1111111111111111111");

  // Derive PDAs
  const [programConfigPda, programConfigBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("universal_nft_program")],
    program.programId
  );

  const collectionMint = Keypair.generate();
  
  const [collectionMetadata] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      collectionMint.publicKey.toBuffer(),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  );

  const [collectionMasterEdition] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      collectionMint.publicKey.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  );

  const collectionTokenAccount = await getAssociatedTokenAddress(
    collectionMint.publicKey,
    wallet.publicKey
  );

  // Collection metadata
  const collectionName = "Universal Cross-Chain NFTs";
  const collectionSymbol = "XCNFT";
  const collectionUri = "https://universal-nft.example.com/collection.json";

  console.log("🏗️  Initializing program...");
  
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
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([collectionMint])
      .rpc();

    console.log("✅ Program initialized successfully!");
    console.log("📝 Transaction signature:", tx);
    console.log("🏪 Collection mint:", collectionMint.publicKey.toString());
    console.log("⚙️  Program config PDA:", programConfigPda.toString());

    // Verify deployment
    const programConfig = await program.account.programConfig.fetch(programConfigPda);
    console.log("\n📊 Program Configuration:");
    console.log("   Authority:", programConfig.authority.toString());
    console.log("   Gateway Program:", programConfig.gatewayProgramId.toString());
    console.log("   Collection Mint:", programConfig.collectionMint.toString());
    console.log("   Is Initialized:", programConfig.isInitialized);
    console.log("   Total NFTs Minted:", programConfig.totalNftsMinted.toString());

    // Create deployment info file
    const deploymentInfo = {
      network: "devnet",
      programId: program.programId.toString(),
      programConfigPda: programConfigPda.toString(),
      authority: wallet.publicKey.toString(),
      gatewayProgramId: gatewayProgramId.toString(),
      collectionMint: collectionMint.publicKey.toString(),
      collectionMetadata: collectionMetadata.toString(),
      collectionTokenAccount: collectionTokenAccount.toString(),
      transactionSignature: tx,
      timestamp: new Date().toISOString(),
    };

    const fs = require('fs');
    fs.writeFileSync('deployment-info.json', JSON.stringify(deploymentInfo, null, 2));
    console.log("\n💾 Deployment info saved to deployment-info.json");

  } catch (error) {
    console.error("❌ Deployment failed:", error);
    throw error;
  }
}

async function updateTssAddress() {
  console.log("🔄 Updating TSS address (run this after getting actual TSS address from ZetaChain)...");
  
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNftProgram as Program<UniversalNftProgram>;
  
  // Validate TSS address - never use zero address in production
  const tssAddressEnv = process.env.TSS_ADDRESS;
  if (!tssAddressEnv && !process.env.ALLOW_ZERO_TSS) {
    throw new Error("TSS_ADDRESS env var is required. Set ALLOW_ZERO_TSS=1 for testing only.");
  }
  
  const tssAddress = tssAddressEnv 
    ? Array.from(Buffer.from(tssAddressEnv.replace('0x', ''), 'hex'))
    : Array.from(Buffer.alloc(20)); // Only allowed with ALLOW_ZERO_TSS=1
  
  const [programConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("universal_nft_program")],
    program.programId
  );

  try {
    const tx = await program.methods
      .updateGatewayConfig(null, tssAddress)
      .accounts({
        programConfig: programConfigPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("✅ TSS address updated successfully!");
    console.log("📝 Transaction signature:", tx);
  } catch (error) {
    console.error("❌ TSS address update failed:", error);
  }
}

async function demonstrateCrossChainFlow() {
  console.log("🌐 Demonstrating cross-chain NFT flow...");
  
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.UniversalNftProgram as Program<UniversalNftProgram>;
  
  const [programConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("universal_nft_program")],
    program.programId
  );

  // Step 1: Mint an NFT
  const nftMint = Keypair.generate();
  const nftTokenAccount = await getAssociatedTokenAddress(
    nftMint.publicKey,
    provider.wallet.publicKey
  );
  
  const [nftStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft_state"), nftMint.publicKey.toBuffer()],
    program.programId
  );

  const [nftMetadata] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      nftMint.publicKey.toBuffer(),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  );

  console.log("🎨 Minting demonstration NFT...");
  const mintTx = await program.methods
    .mintNft("Demo Cross-Chain NFT", "DEMO", "https://example.com/demo.json", null)
    .accounts({
      programConfig: programConfigPda,
      nftState: nftStatePda,
      nftMint: nftMint.publicKey,
      nftMetadata,
      nftTokenAccount,
      owner: provider.wallet.publicKey,
      authority: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .signers([nftMint])
    .rpc();

  console.log("✅ NFT minted:", mintTx);
  console.log("🆔 NFT Mint:", nftMint.publicKey.toString());

  // Step 2: Burn for cross-chain transfer
  const destinationChainId = 1; // Ethereum
  const destinationAddress = Buffer.from("742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42", "hex");

  console.log("🔥 Burning NFT for cross-chain transfer...");
  const burnTx = await program.methods
    .burnForCrossChain(
      new anchor.BN(destinationChainId),
      Array.from(destinationAddress)
    )
    .accounts({
      programConfig: programConfigPda,
      nftState: nftStatePda,
      nftMint: nftMint.publicKey,
      nftTokenAccount,
      owner: provider.wallet.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("✅ NFT burned for cross-chain:", burnTx);
  console.log("🌐 Destination Chain ID:", destinationChainId);
  console.log("📍 Destination Address:", destinationAddress.toString("hex"));

  // Fetch and display updated state
  const nftState = await program.account.nftState.fetch(nftStatePda);
  console.log("\n📊 Final NFT State:");
  console.log("   Token ID:", nftState.tokenId.toString());
  console.log("   Is Cross-Chain Locked:", nftState.isCrossChainLocked);
  console.log("   Cross-Chain History Length:", nftState.crossChainHistory.length);
  
  const programConfig = await program.account.programConfig.fetch(programConfigPda);
  console.log("\n📊 Program Statistics:");
  console.log("   Total NFTs Minted:", programConfig.totalNftsMinted.toString());
  console.log("   Total Cross-Chain Transfers:", programConfig.totalCrossChainTransfers.toString());
}

// Main deployment function
async function main() {
  const args = process.argv.slice(2);
  
  switch (args[0]) {
    case "deploy":
      await deployUniversalNftProgram();
      break;
    case "update-tss":
      await updateTssAddress();
      break;
    case "demo":
      await demonstrateCrossChainFlow();
      break;
    default:
      console.log("Usage:");
      console.log("  npm run deploy:devnet deploy     - Deploy the program");
      console.log("  npm run deploy:devnet update-tss - Update TSS address");
      console.log("  npm run deploy:devnet demo       - Run cross-chain demo");
      break;
  }
}

main().catch(console.error);