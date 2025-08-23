import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, Connection, clusterApiUrl } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

// Configuration for devnet deployment
const DEVNET_CONFIG = {
  cluster: "devnet",
  rpcUrl: clusterApiUrl("devnet"),
  programId: "UNFTfPKLjpJqVxfCcqvdJUPDAXZnVqUcVaQ3CYwx2TU",
  gatewayProgramId: "ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis",
  gatewayPda: "2f9SLuUNb7TNeM6gzBwT4ZjbL5ZyKzzHg1Ce9yiquEjj",
  // ZetaChain testnet TSS address (example - replace with actual)
  tssAddress: [
    0x85, 0x31, 0xa5, 0xab, 0x84, 0x7f, 0xf5, 0xb2,
    0x2d, 0x85, 0x56, 0x33, 0xc2, 0x5e, 0xd1, 0xda,
    0x32, 0x55, 0x24, 0x7e
  ]
};

async function main() {
  console.log("Deploying Universal NFT to Solana Devnet...");

  // Setup connection and wallet
  const connection = new Connection(DEVNET_CONFIG.rpcUrl, "confirmed");

  // Load wallet from local keypair
  const walletPath = path.join(process.env.HOME || "", ".config/solana/id.json");
  if (!fs.existsSync(walletPath)) {
    throw new Error("Wallet not found. Please run: solana-keygen new");
  }

  const walletKeypair = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
  );

  const wallet = new anchor.Wallet(walletKeypair);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  // Load the program
  const programId = new PublicKey(DEVNET_CONFIG.programId);
  const idl = JSON.parse(
    fs.readFileSync(
      path.join(__dirname, "../target/idl/universal_nft.json"),
      "utf-8"
    )
  );

  const program: Program = new anchor.Program(idl as anchor.Idl, programId, provider);

  // Derive collection PDA
  const [collectionPda, collectionBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("collection"), wallet.publicKey.toBuffer()],
    program.programId
  );

  console.log("📝 Configuration:");
  console.log("  Program ID:", programId.toString());
  console.log("  Gateway Program:", DEVNET_CONFIG.gatewayProgramId);
  console.log("  Gateway PDA:", DEVNET_CONFIG.gatewayPda);
  console.log("  Collection PDA:", collectionPda.toString());
  console.log("  Deployer:", wallet.publicKey.toString());

  try {
    // Check if collection already exists
    const collectionAccount = await connection.getAccountInfo(collectionPda);

    if (collectionAccount) {
      console.log("✅ Collection already initialized at:", collectionPda.toString());

      // Fetch collection data
      const collection = (await program.account.collection.fetch(collectionPda)) as any;
      console.log("📊 Collection Info:");
      console.log("  Name:", collection.name);
      console.log("  Symbol:", collection.symbol);
      console.log("  Next Token ID:", (collection.nextTokenId as anchor.BN)?.toString?.() ?? String(collection.nextTokenId));
      console.log("  TSS Address:", Buffer.from(collection.tssAddress as number[]).toString("hex"));
    } else {
      console.log("🔨 Initializing new collection...");

      // Initialize collection
      const tx = await program.methods
        .initializeCollection(
          "Universal NFT",
          "UNFT",
          "https://universal-nft.zetachain.com/metadata/",
          DEVNET_CONFIG.tssAddress
        )
        .accounts({
          collection: collectionPda,
          authority: wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Collection initialized!");
      console.log("  Transaction:", tx);
      console.log("  Collection PDA:", collectionPda.toString());

      // Wait for confirmation
      await connection.confirmTransaction(tx, "confirmed");

      // Fetch and display collection data
      const collection = (await program.account.collection.fetch(collectionPda)) as any;
      console.log("📊 Collection Info:");
      console.log("  Name:", collection.name);
      console.log("  Symbol:", collection.symbol);
      console.log("  Base URI:", (collection.baseUri ?? collection.uri ?? ""));
      console.log("  TSS Address:", Buffer.from(collection.tssAddress as number[]).toString("hex"));
    }

    // Save deployment info
    const deploymentInfo = {
      network: "devnet",
      programId: programId.toString(),
      collectionPda: collectionPda.toString(),
      gatewayProgramId: DEVNET_CONFIG.gatewayProgramId,
      gatewayPda: DEVNET_CONFIG.gatewayPda,
      deployer: wallet.publicKey.toString(),
      timestamp: new Date().toISOString(),
    };

    const deploymentPath = path.join(__dirname, "../deployment-devnet.json");
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
    console.log("\n💾 Deployment info saved to:", deploymentPath);

    console.log("\n🎉 Deployment complete!");
    console.log("\n📋 Next steps:");
    console.log("1. Fund the collection PDA with SOL for transaction fees");
    console.log("2. Configure ZetaChain gateway to recognize this program");
    console.log("3. Test minting and cross-chain transfers");
    console.log("\n🔗 View on Solana Explorer:");
    console.log(`https://explorer.solana.com/address/${collectionPda.toString()}?cluster=devnet`);

  } catch (error) {
    console.error("Deployment failed:", error);
    throw error;
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
