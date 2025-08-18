// index.js - Client script to interact with ZetaMint program

const anchor = require('@project-serum/anchor');
const { Connection, PublicKey, clusterApiUrl } = require('@solana/web3.js');
const fs = require('fs');

// Load the keypair for the wallet
const walletKeyPair = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync('./target/deploy/zetamint-keypair.json', 'utf8')))
);

// Load the program ID from Anchor.toml
// You can also hardcode: const programID = new PublicKey("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");
const programID = new PublicKey("EteYkYdk3kTpHYqzc6Exrx9JqZDF2n2jw53sF318oKiU");

// Connect to Solana devnet
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');

// Set up the Anchor provider
const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(walletKeyPair), {
    preflightCommitment: 'processed',
});
anchor.setProvider(provider);

// Load the program
const idl = JSON.parse(fs.readFileSync('./target/idl/zetamint.json', 'utf8'));
const program = new anchor.Program(idl, programID, provider);

// Example: Mint NFT function
async function mintNFT() {
    try {
        console.log("Minting NFT...");
              const tx = await program.methods
                  .initialize()
                  .rpc();

        console.log("Transaction successful, signature:", tx);
    } catch (error) {
        console.error("Error minting NFT:", error);
    }
}

// Run example
mintNFT();
