import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalNft } from "../contracts/solana/target/types/universal_nft";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import * as crypto from "crypto";

// Configuration
const PROGRAM_ID = new PublicKey("6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke");
const GATEWAY_ADDRESS = new PublicKey("GatewayAddress111111111111111111111111111");

// Chain IDs for cross-chain operations
const BASE_CHAIN_ID = 84532; // Base Sepolia
const ZETACHAIN_ID = 7001; // ZetaChain Testnet

export class SolanaCrossChain {
    private program: Program<UniversalNft>;
    private provider: anchor.AnchorProvider;

    constructor() {
        // Set up the provider and program
        this.provider = anchor.AnchorProvider.env();
        anchor.setProvider(this.provider);
        this.program = anchor.workspace.UniversalNft as Program<UniversalNft>;
    }

    /**
     * Initialize a new NFT collection on Solana
     */
    async initializeCollection(name: string, symbol: string, uri: string): Promise<string> {
        const authority = this.provider.wallet.publicKey;
        
        // Derive collection PDA
        const [collectionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("collection"), authority.toBuffer(), Buffer.from(name)],
            this.program.programId
        );

        console.log(`🔧 Initializing collection: ${name}`);
        console.log(`📍 Collection PDA: ${collectionPda.toString()}`);

        const tx = await this.program.methods
            .initializeCollection(name, symbol, uri)
            .accounts({
                collection: collectionPda,
                authority: authority,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
            })
            .rpc();

        console.log(`✅ Collection initialized. TX: ${tx}`);
        return tx;
    }

    /**
     * Mint an NFT on Solana
     */
    async mintNft(
        collectionName: string,
        tokenUri: string,
        recipient?: PublicKey
    ): Promise<{ tokenId: string; txHash: string }> {
        const authority = this.provider.wallet.publicKey;
        const mintRecipient = recipient || authority;

        // Derive collection PDA
        const [collectionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("collection"), authority.toBuffer(), Buffer.from(collectionName)],
            this.program.programId
        );

        // Generate deterministic token ID
        const tokenIdHash = crypto.createHash('sha256')
            .update(collectionPda.toBuffer())
            .update(Buffer.from(tokenUri))
            .update(mintRecipient.toBuffer())
            .digest();
        
        const tokenId = tokenIdHash.readBigUInt64LE(0);
        const tokenIdBuffer = Buffer.alloc(8);
        tokenIdBuffer.writeBigUInt64LE(tokenId);

        // Derive NFT Origin PDA
        const [nftOriginPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("nft_origin"), tokenIdBuffer],
            this.program.programId
        );

        console.log(`🎨 Minting NFT with token ID: ${tokenId.toString()}`);
        console.log(`📍 NFT Origin PDA: ${nftOriginPda.toString()}`);

        const tx = await this.program.methods
            .mintNft(tokenUri)
            .accounts({
                collection: collectionPda,
                nftOrigin: nftOriginPda,
                authority: authority,
                recipient: mintRecipient,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
            })
            .rpc();

        console.log(`✅ NFT minted. TX: ${tx}`);
        return { tokenId: tokenId.toString(), txHash: tx };
    }

    /**
     * Transfer NFT cross-chain from Solana to another chain
     */
    async transferCrossChain(
        collectionName: string,
        tokenId: string,
        destinationChainId: number,
        destinationAddress: string,
        gasLimit: number = 500000
    ): Promise<string> {
        const authority = this.provider.wallet.publicKey;
        const tokenIdBN = BigInt(tokenId);
        const tokenIdBuffer = Buffer.alloc(8);
        tokenIdBuffer.writeBigUInt64LE(tokenIdBN);

        // Derive PDAs
        const [collectionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("collection"), authority.toBuffer(), Buffer.from(collectionName)],
            this.program.programId
        );

        const [nftOriginPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("nft_origin"), tokenIdBuffer],
            this.program.programId
        );

        const [connectedPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("connected"), collectionPda.toBuffer(), Buffer.from(destinationChainId.toString())],
            this.program.programId
        );

        console.log(`🌉 Initiating cross-chain transfer`);
        console.log(`📤 From: Solana`);
        console.log(`📥 To: Chain ${destinationChainId}`);
        console.log(`🎯 Destination: ${destinationAddress}`);
        console.log(`🆔 Token ID: ${tokenId}`);

        const tx = await this.program.methods
            .transferCrossChain(
                destinationChainId,
                Buffer.from(destinationAddress.replace('0x', ''), 'hex'),
                gasLimit
            )
            .accounts({
                collection: collectionPda,
                nftOrigin: nftOriginPda,
                connected: connectedPda,
                authority: authority,
                gateway: GATEWAY_ADDRESS,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        console.log(`✅ Cross-chain transfer initiated. TX: ${tx}`);
        return tx;
    }

    /**
     * Set up connection to another chain
     */
    async setConnected(
        collectionName: string,
        chainId: number,
        connectedAddress: string,
        gasToken: string
    ): Promise<string> {
        const authority = this.provider.wallet.publicKey;

        const [collectionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("collection"), authority.toBuffer(), Buffer.from(collectionName)],
            this.program.programId
        );

        const [connectedPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("connected"), collectionPda.toBuffer(), Buffer.from(chainId.toString())],
            this.program.programId
        );

        console.log(`🔗 Setting up connection to chain ${chainId}`);
        console.log(`📍 Connected contract: ${connectedAddress}`);

        const tx = await this.program.methods
            .setConnected(
                chainId,
                Buffer.from(connectedAddress.replace('0x', ''), 'hex'),
                Buffer.from(gasToken.replace('0x', ''), 'hex')
            )
            .accounts({
                collection: collectionPda,
                connected: connectedPda,
                authority: authority,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
            })
            .rpc();

        console.log(`✅ Connection established. TX: ${tx}`);
        return tx;
    }
}

// Main execution function for cross-chain flows
export async function executeCrossChainFlows() {
    console.log("🚀 Starting Solana Cross-Chain Operations");
    
    const solana = new SolanaCrossChain();
    const collectionName = "CrossChainNFT";
    const collectionSymbol = "CCNFT";
    const collectionUri = "https://example.com/collection-metadata.json";

    try {
        // Step 1: Initialize collection (if not already done)
        console.log("\n📋 Step 1: Initialize Collection");
        try {
            await solana.initializeCollection(collectionName, collectionSymbol, collectionUri);
        } catch (error) {
            console.log("ℹ️ Collection might already exist, continuing...");
        }

        // Step 2: Set up connections to other chains
        console.log("\n🔗 Step 2: Set up cross-chain connections");
        
        // Connect to Base Sepolia (placeholder addresses - replace with actual deployed contracts)
        await solana.setConnected(
            collectionName,
            BASE_CHAIN_ID,
            "0x1234567890123456789012345678901234567890", // Replace with actual Base contract
            "0x236b0DE675cC8F46AE186897fCCeFe3370C9eDeD"  // ZRC20 Base token
        );

        // Step 3: Mint NFT on Solana
        console.log("\n🎨 Step 3: Mint NFT on Solana");
        const { tokenId, txHash } = await solana.mintNft(
            collectionName,
            "https://example.com/nft-metadata/1.json"
        );

        // Step 4: Transfer Solana → Base
        console.log("\n🌉 Step 4: Transfer Solana → Base");
        const transferTx = await solana.transferCrossChain(
            collectionName,
            tokenId,
            BASE_CHAIN_ID,
            "0x1234567890123456789012345678901234567890", // Recipient on Base
            500000
        );

        console.log("\n🎉 Cross-chain operations completed!");
        console.log(`📊 Summary:`);
        console.log(`  Collection: ${collectionName}`);
        console.log(`  Mint TX: ${txHash}`);
        console.log(`  Transfer TX: ${transferTx}`);
        console.log(`  Token ID: ${tokenId}`);

    } catch (error) {
        console.error("❌ Error during cross-chain operations:", error);
        throw error;
    }
}

// Execute if run directly
if (require.main === module) {
    executeCrossChainFlows()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}
