const anchor = require("@coral-xyz/anchor");
const { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } = require("@solana/spl-token");
const fs = require("fs");
const path = require("path");

// Metaplex Token Metadata Program
const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

async function main() {
    console.log("\n🚀 Solana Universal NFT - Simple Test\n");
    
    // Load deployment config
    const deployment = JSON.parse(fs.readFileSync(path.join(__dirname, "../deployment.json"), "utf-8"));
    
    // Configure provider
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    
    // Load IDL and create program without type checking
    const programId = new PublicKey(deployment.programId);
    const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/universal_nft.json"), "utf-8"));
    
    // Create program instance using the IDL directly
    const program = new anchor.Program(idl, provider);
    program.programId = programId;
    
    console.log("📋 Configuration:");
    console.log(`   Program ID: ${programId.toBase58()}`);
    console.log(`   Collection: ${deployment.collectionPda}`);
    console.log(`   Authority: ${provider.wallet.publicKey.toBase58()}\n`);
    
    const authority = provider.wallet.publicKey;
    const collectionPda = new PublicKey(deployment.collectionPda);
    const collectionMint = new PublicKey(deployment.collectionMint);
    
    // Test 1: Mint NFT
    console.log("🎨 Test 1: Minting NFT");
    console.log("─────────────────────");
    
    const nftMint = Keypair.generate();
    const tokenId = Date.now();
    const nftName = `Test NFT #${tokenId}`;
    const nftUri = `https://example.com/nft/${tokenId}`;
    
    const nftTokenAccount = await getAssociatedTokenAddress(
        nftMint.publicKey,
        authority
    );
    
    const [nftMetadata] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("metadata"),
            TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            nftMint.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
    );
    
    try {
        // Build the instruction manually
        const accounts = {
            collection: collectionPda,
            mint: nftMint.publicKey,
            tokenAccount: nftTokenAccount,
            authority: authority,
            metadata: nftMetadata,
            payer: authority,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            metadataProgram: TOKEN_METADATA_PROGRAM_ID,
        };
        
        const instruction = await program.instruction.mintNft(
            nftName,
            nftUri,
            {
                accounts: accounts,
            }
        );
        
        const tx = new anchor.web3.Transaction().add(instruction);
        const signature = await provider.sendAndConfirm(tx, [nftMint]);
        
        console.log(`   ✅ NFT Minted: ${nftName}`);
        console.log(`   Token ID: ${tokenId}`);
        console.log(`   Mint: ${nftMint.publicKey.toBase58()}`);
        console.log(`   Tx: ${signature}\n`);
        
        // Test 2: Cross-Chain Transfer
        console.log("🌉 Test 2: Cross-Chain Transfer");
        console.log("───────────────────────────────");
        
        const destinationChainId = 84532; // Base Sepolia
        const recipientAddress = "0x1234567890123456789012345678901234567890";
        const recipientBytes = Buffer.from(recipientAddress.slice(2), "hex");
        
        const GATEWAY_PROGRAM_ID = new PublicKey("ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis");
        const [gatewayPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("meta")],
            GATEWAY_PROGRAM_ID
        );
        
        const transferAccounts = {
            collection: collectionPda,
            authority: authority,
            mint: nftMint.publicKey,
            tokenAccount: nftTokenAccount,
            gateway: GATEWAY_PROGRAM_ID,
            gatewayPda: gatewayPda,
            payer: authority,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        };
        
        const transferInstruction = await program.instruction.transferCrossChain(
            new anchor.BN(destinationChainId),
            recipientBytes,
            {
                accounts: transferAccounts,
            }
        );
        
        const transferTx = new anchor.web3.Transaction().add(transferInstruction);
        const transferSignature = await provider.sendAndConfirm(transferTx);
        
        console.log(`   ✅ Cross-Chain Transfer Initiated`);
        console.log(`   Destination: Base Sepolia (${destinationChainId})`);
        console.log(`   Recipient: ${recipientAddress}`);
        console.log(`   Tx: ${transferSignature}\n`);
        
    } catch (error) {
        console.error("Error:", error);
        if (error.logs) {
            console.error("Program logs:", error.logs);
        }
    }
    
    console.log("✨ Test completed!\n");
    console.log("View transactions on Solana Explorer:");
    console.log("https://explorer.solana.com/?cluster=devnet\n");
}

main().catch(console.error);
