const anchor = require("@coral-xyz/anchor");
const { Connection, PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, Transaction, sendAndConfirmTransaction } = require("@solana/web3.js");
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } = require("@solana/spl-token");
const fs = require("fs");
const path = require("path");
const bs58 = require("bs58");

// Metaplex Token Metadata Program
const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

async function main() {
    console.log("\n🚀 Solana Universal NFT - Test Suite\n");
    
    // Load deployment config
    const deployment = JSON.parse(fs.readFileSync(path.join(__dirname, "../deployment.json"), "utf-8"));
    
    // Setup connection and wallet
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const walletPath = process.env.ANCHOR_WALLET || "/Users/ayushsrivastava/.config/solana/id.json";
    const walletKeypair = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
    );
    
    // Create provider manually
    const wallet = {
        publicKey: walletKeypair.publicKey,
        signTransaction: async (tx) => {
            tx.partialSign(walletKeypair);
            return tx;
        },
        signAllTransactions: async (txs) => {
            return txs.map(tx => {
                tx.partialSign(walletKeypair);
                return tx;
            });
        }
    };
    
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
        preflightCommitment: "confirmed"
    });
    anchor.setProvider(provider);
    
    // Load program with correct initialization
    const programId = new PublicKey(deployment.programId);
    const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/universal_nft.json"), "utf-8"));
    
    // Fix the IDL metadata
    if (idl.metadata) {
        idl.metadata.address = deployment.programId;
    } else {
        idl.metadata = { address: deployment.programId };
    }
    
    // Create program with explicit program ID
    const program = new anchor.Program(idl, programId, provider);
    
    console.log("📋 Configuration:");
    console.log(`   Program ID: ${programId.toBase58()}`);
    console.log(`   Collection: ${deployment.collectionPda}`);
    console.log(`   Authority: ${walletKeypair.publicKey.toBase58()}\n`);
    
    const authority = walletKeypair.publicKey;
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
        console.log(`   Tx: ${mintTx}`);
        console.log(`   View: https://explorer.solana.com/tx/${mintTx}?cluster=devnet\n`);
        
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
        console.log(`   Tx: ${transferTx}`);
        console.log(`   View: https://explorer.solana.com/tx/${transferTx}?cluster=devnet\n`);
        
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
            console.log(`   Tx: ${onCallTx}`);
            console.log(`   View: https://explorer.solana.com/tx/${onCallTx}?cluster=devnet\n`);
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
}

main().catch(console.error);
