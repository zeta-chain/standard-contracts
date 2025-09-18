// index.js - Client script to interact with ZetaMint program

const anchor = require('@coral-xyz/anchor');
const { Connection, PublicKey, clusterApiUrl } = require('@solana/web3.js');
const fs = require('fs');

// Initialize provider from environment (ANCHOR_WALLET, ANCHOR_PROVIDER_URL)
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

// Load IDL and derive Program ID from it
const idl = JSON.parse(fs.readFileSync('./target/idl/zetamint.json', 'utf8'));
const programID = new PublicKey(idl.metadata.address);

// Provider is already set from environment (ANCHOR_PROVIDER_URL). No duplicate setup needed.

// Initialize the Program with the environment provider
const program = new anchor.Program(idl, programID, anchor.getProvider());

// Example: Mint NFT function
async function mintNFT() {
    try {
        console.log("Minting NFT...");
              const tx = await program.methods.initialize().rpc();

        console.log("Transaction successful, signature:", tx);
    } catch (error) {
        console.error("Error minting NFT:", error);
    }
}

// Run example
mintNFT();
