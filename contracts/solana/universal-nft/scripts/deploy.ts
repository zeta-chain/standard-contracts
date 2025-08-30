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
import { PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { getEvmAddressArray } from "../utils/address";

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

  // ZetaChain gateway program ID validation
  const gatewayProgramIdEnv = process.env.GATEWAY_PROGRAM_ID;
  if (!gatewayProgramIdEnv) {
    throw new Error(
      "GATEWAY_PROGRAM_ID environment variable is required. " +
      "Set it to the actual ZetaChain gateway program ID for your target network."
    );
  }
  
  let gatewayProgramId: PublicKey;
  try {
    gatewayProgramId = new PublicKey(gatewayProgramIdEnv);
    console.log(`✅ Gateway Program ID validated: ${gatewayProgramId.toString()}`);
  } catch (error) {
    throw new Error(
      `Invalid GATEWAY_PROGRAM_ID format: "${gatewayProgramIdEnv}". ` +
      "Must be a valid Solana public key (base58 encoded, 32 bytes)."
    );
  }
  
  // Additional validation: ensure it's not a placeholder or obviously invalid key
  const placeholderPatterns = [
    /placeholder/i,
    /111111+/,
    /000000+/,
    /test/i
  ];
  
  const gatewayProgramIdStr = gatewayProgramId.toString();
  const isPlaceholder = placeholderPatterns.some(pattern => 
    pattern.test(gatewayProgramIdStr)
  );
  
  if (isPlaceholder) {
    throw new Error(
      `Gateway Program ID appears to be a placeholder: "${gatewayProgramIdStr}". ` +
      "Please set a valid ZetaChain gateway program ID."
    );
  }

  // Derive PDAs
  const [programConfigPda, programConfigBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("universal_nft_program")],
    program.programId
  );

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
        metadataProgram: TOKEN_METADATA_PROGRAM_ID,
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
  
  let tssAddress: number[];
  if (tssAddressEnv) {
    // Enhanced TSS address validation
    const normalizedAddress = tssAddressEnv.toLowerCase();
    
    // Check if it starts with 0x and remove it
    const hexAddress = normalizedAddress.startsWith('0x') 
      ? normalizedAddress.slice(2) 
      : normalizedAddress;
    
    // Validate hex format (must be exactly 40 hex characters = 20 bytes)
    if (!/^[0-9a-f]{40}$/i.test(hexAddress)) {
      throw new Error(
        `Invalid TSS address format: must be 40 hex characters (20 bytes). Got: ${hexAddress.length} characters`
      );
    }
    
    // Convert to buffer and validate 20-byte length
    const addressBuffer = Buffer.from(hexAddress, 'hex');
    if (addressBuffer.length !== 20) {
      throw new Error(
        `TSS address must be exactly 20 bytes. Got: ${addressBuffer.length} bytes`
      );
    }
    
    // Validate not zero address (all zeros)
    const isZeroAddress = addressBuffer.every(byte => byte === 0);
    if (isZeroAddress && !process.env.ALLOW_ZERO_TSS) {
      throw new Error("Zero address not allowed in production. Set ALLOW_ZERO_TSS=1 for testing only.");
    }
    
    console.log(`✅ TSS address validated: 0x${hexAddress}`);
    tssAddress = Array.from(addressBuffer);
  } else {
    // Use zero address only in test mode
    console.log("⚠️  Using zero TSS address (test mode only)");
    tssAddress = Array.from(Buffer.alloc(20));
  }
  
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
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      nftMint.publicKey.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
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
  const destinationAddress = getEvmAddressArray("0x742C4883a7De56b4D90f8F6f1F6c6b8D8b4d4b42");

  console.log("🔥 Burning NFT for cross-chain transfer...");
  const burnTx = await program.methods
    .burnForCrossChain(
      new anchor.BN(destinationChainId),
      destinationAddress
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