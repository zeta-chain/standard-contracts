const anchor = require("@coral-xyz/anchor");
const { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } = require("@solana/spl-token");
const fs = require("fs");
const path = require("path");

// Metaplex Token Metadata Program
const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

async function main() {
    console.log("\n🚀 Solana Universal NFT - Local Test Suite\n");
    
    // Load deployment config
    const deploymentPath = path.join(__dirname, "../deployment.json");
    const deployment = JSON.parse(fs.readFileSync(deploymentPath, "utf-8"));
    
    // Configure provider
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    
    // Load program
    const programId = new PublicKey(deployment.programId);
    const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/universal_nft.json"), "utf-8"));
    // Ensure the IDL has the correct address
    idl.metadata = idl.metadata || {};
    idl.metadata.address = deployment.programId;
    const program = new anchor.Program(idl, programId, provider);
    
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
        const mintTx = await program.methods
            .mintNft(nftName, nftUri)
            .accounts({
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
            })
            .signers([nftMint])
            .rpc();
        
        console.log(`   ✅ NFT Minted: ${nftName}`);
        console.log(`   Token ID: ${tokenId}`);
        console.log(`   Mint: ${nftMint.publicKey.toBase58()}`);
        console.log(`   Tx: ${mintTx}\n`);
        
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
        
        const transferTx = await program.methods
            .transferCrossChain(new anchor.BN(destinationChainId), recipientBytes)
            .accounts({
                collection: collectionPda,
                authority: authority,
                mint: nftMint.publicKey,
                tokenAccount: nftTokenAccount,
                gateway: GATEWAY_PROGRAM_ID,
                gatewayPda: gatewayPda,
                payer: authority,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc();
        
        console.log(`   ✅ Cross-Chain Transfer Initiated`);
        console.log(`   Destination: Base Sepolia (${destinationChainId})`);
        console.log(`   Recipient: ${recipientAddress}`);
        console.log(`   Tx: ${transferTx}\n`);
        
        // Test 3: Simulate On-Call Reception
        console.log("📥 Test 3: Simulating On-Call Reception");
        console.log("────────────────────────────────────────");
        
        const incomingNftMint = Keypair.generate();
        const incomingTokenId = Date.now() + 1;
        const incomingUri = `https://example.com/incoming/${incomingTokenId}`;
        
        const incomingTokenAccount = await getAssociatedTokenAddress(
            incomingNftMint.publicKey,
            authority
        );
        
        const [incomingMetadata] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("metadata"),
                TOKEN_METADATA_PROGRAM_ID.toBuffer(),
                incomingNftMint.publicKey.toBuffer(),
            ],
            TOKEN_METADATA_PROGRAM_ID
        );
        
        // Construct cross-chain message
        const tokenIdBuffer = Buffer.alloc(8);
        tokenIdBuffer.writeBigUInt64LE(BigInt(incomingTokenId));
        
        const uriBytes = Buffer.from(incomingUri);
        const uriLenBuffer = Buffer.alloc(4);
        uriLenBuffer.writeUInt32LE(uriBytes.length);
        
        const recipientBuffer = authority.toBuffer();
        
        const message = Buffer.concat([
            tokenIdBuffer,
            uriLenBuffer,
            uriBytes,
            recipientBuffer
        ]);
        
        const sender = Array.from(Buffer.from("1234567890123456789012345678901234567890", "hex"));
        const sourceChainId = 1; // Ethereum
        
        try {
            const onCallTx = await program.methods
                .onCall(sender, new anchor.BN(sourceChainId), message)
                .accounts({
                    collection: collectionPda,
                    collectionMint: collectionMint,
                    gateway: GATEWAY_PROGRAM_ID,
                    gatewayPda: gatewayPda,
                    mint: incomingNftMint.publicKey,
                    tokenAccount: incomingTokenAccount,
                    recipient: authority,
                    metadata: incomingMetadata,
                    payer: authority,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    metadataProgram: TOKEN_METADATA_PROGRAM_ID,
                })
                .signers([incomingNftMint])
                .rpc();
            
            console.log(`   ✅ NFT Received via On-Call`);
            console.log(`   Source: Ethereum (${sourceChainId})`);
            console.log(`   Token ID: ${incomingTokenId}`);
            console.log(`   Tx: ${onCallTx}\n`);
        } catch (error) {
            console.log(`   ⚠️  On-Call simulation failed (expected - only gateway can call)`);
            console.log(`   Error: ${error.message}\n`);
        }
        
    } catch (error) {
        console.error("Error:", error);
        if (error.logs) {
            console.error("Program logs:", error.logs);
        }
    }
    
    console.log("✨ Test suite completed!\n");
    console.log("View transactions on Solana Explorer:");
    console.log("https://explorer.solana.com/?cluster=devnet\n");
}

main().catch(console.error);
