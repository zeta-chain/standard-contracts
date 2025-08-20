#!/usr/bin/env node

import { Connection, PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import type { UniversalNft } from "../solana/target/types/universal_nft";
import idl from "../solana/target/idl/universal_nft.json";
import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import { homedir } from "os";

// Config
const SOLANA_DEVNET_RPC = "https://api.devnet.solana.com";
const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

function toU64LE(n: bigint): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(n);
  return b;
}

function sha256(...parts: Buffer[]): Buffer {
  const h = crypto.createHash("sha256");
  for (const p of parts) h.update(p);
  return h.digest();
}

export class UniversalNftCLI {
  private connection: Connection;
  private program: Program<UniversalNft>;
  private wallet: Keypair;

  constructor() {
    this.connection = new Connection(SOLANA_DEVNET_RPC, "confirmed");
    this.wallet = this.loadWallet();
    this.program = this.loadProgram();
  }

  private loadWallet(): Keypair {
    const walletPath = path.join(homedir(), ".config", "solana", "id.json");
    if (!fs.existsSync(walletPath)) {
      throw new Error("Wallet not found at ~/.config/solana/id.json");
    }
    const walletData = JSON.parse(fs.readFileSync(walletPath, "utf8"));
    return Keypair.fromSecretKey(new Uint8Array(walletData));
  }

  private loadProgram(): Program<UniversalNft> {
    const provider = new anchor.AnchorProvider(
      this.connection,
      new anchor.Wallet(this.wallet),
      { commitment: "confirmed" }
    );
    return new Program(idl as UniversalNft, provider) as Program<UniversalNft>;
  }

  private deriveConfigPda(): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("universal_nft_config")],
      this.program.programId
    )[0];
  }

  private deriveMetadataPda(mint: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      METADATA_PROGRAM_ID
    )[0];
  }

  private deriveMasterEditionPda(mint: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from("edition")],
      METADATA_PROGRAM_ID
    )[0];
  }

  private deriveNftOriginPda(tokenId: Buffer): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("nft_origin"), tokenId],
      this.program.programId
    )[0];
  }

  private deriveTicketPda(mint: PublicKey, authority: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("mint_ticket"), mint.toBuffer(), authority.toBuffer()],
      this.program.programId
    )[0];
  }

  async initialize(gatewayProgram: string, gatewayPda: string): Promise<void> {
    const configPda = this.deriveConfigPda();
    const tx = await this.program.methods
      .initialize(new PublicKey(gatewayProgram))
      .accountsStrict({
        config: configPda,
        authority: this.wallet.publicKey,
        gatewayProgram: new PublicKey(gatewayProgram),
        gatewayPda: new PublicKey(gatewayPda),
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Initialized. Tx:", tx);
    console.log("Config:", configPda.toBase58());
  }

  async mint(uri: string, name = "Universal NFT", symbol = "UNFT"): Promise<void> {
    // Two-step flow: reserve -> mint. Reservation yields token_id to derive nft_origin PDA.
    const mint = Keypair.generate();
    const configPda = this.deriveConfigPda();
    const metadataPda = this.deriveMetadataPda(mint.publicKey);
    const masterEditionPda = this.deriveMasterEditionPda(mint.publicKey);
    const ata = await getAssociatedTokenAddress(mint.publicKey, this.wallet.publicKey);

    // Reserve ticket
    const ticketPda = this.deriveTicketPda(mint.publicKey, this.wallet.publicKey);
    await this.program.methods
      .reserveNextTokenId()
      .accountsStrict({
        config: configPda,
        ticket: ticketPda,
        mint: mint.publicKey,
        authority: this.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Read ticket for token_id
    const ticket = await (this.program.account as any).mintTicket.fetch(ticketPda);
    const tokenId: Buffer = Buffer.from(ticket.tokenId as number[]);
    const nftOriginPda = this.deriveNftOriginPda(tokenId);

    const tx = await this.program.methods
      .mintNft(name, symbol, uri)
      .accountsStrict({
        config: configPda,
        ticket: ticketPda,
        nftOrigin: nftOriginPda,
        mint: mint.publicKey,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        tokenAccount: ata,
        authority: this.wallet.publicKey,
        payer: this.wallet.publicKey,
        recipient: this.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        metadataProgram: METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mint])
      .rpc();

    console.log("Minted. Tx:", tx);
    console.log("Mint:", mint.publicKey.toBase58());
    console.log("Token ID:", tokenId.toString("hex"));
  }

  async transferToZetachain(tokenIdHex: string, zcUniversalContractHex: string, destinationChain: string, finalRecipient: string): Promise<void> {
    const tokenId = Buffer.from(tokenIdHex.replace(/^0x/, ''), 'hex');
    if (tokenId.length !== 32) throw new Error("tokenId must be 32 bytes hex");
    const receiver = Buffer.from(zcUniversalContractHex.replace(/^0x/, ''), 'hex');
    if (receiver.length !== 20) throw new Error("zetachain universal contract must be 20 bytes hex");
    // Validate destinationChain parses to u64
    if (!/^\d+$/.test(destinationChain)) throw new Error("destinationChain must be an unsigned integer string");
    const destChainBig = BigInt(destinationChain);
    if (destChainBig < 0n || destChainBig > (1n << 64n) - 1n) throw new Error("destinationChain out of range for u64");
    // Validate final recipient length
    if (!finalRecipient || finalRecipient.length > 128) throw new Error("finalRecipient must be non-empty and <= 128 chars");

    const configPda = this.deriveConfigPda();
    const nftOriginPda = this.deriveNftOriginPda(tokenId);
    const origin = await this.program.account.nftOrigin.fetch(nftOriginPda);
    const mint = (origin as any).originalMint as PublicKey;
    const metadataPda = (origin as any).originalMetadata as PublicKey;
    const ata = await getAssociatedTokenAddress(mint, this.wallet.publicKey);
    const config = await this.program.account.universalNftConfig.fetch(configPda);
    const gatewayProgram = (config as any).gatewayProgram as PublicKey;
    const gatewayPda = (config as any).gatewayPda as PublicKey;

    const tx = await this.program.methods
      .transferToZetachain(Array.from(tokenId), Array.from(receiver), new BN(destChainBig.toString()), finalRecipient)
      .accountsStrict({
        config: configPda,
        nftOrigin: nftOriginPda,
        mint,
        tokenAccount: ata,
        metadata: metadataPda,
        owner: this.wallet.publicKey,
        gatewayProgram,
        gatewayPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Transfer initiated. Tx:", tx);
  }

  async balance(): Promise<void> {
    const lamports = await this.connection.getBalance(this.wallet.publicKey);
    console.log("Balance:", lamports / 1e9, "SOL");
  }

  async status(): Promise<void> {
    try {
      const configPda = this.deriveConfigPda();
      const config = await this.program.account.universalNftConfig.fetch(configPda);
      console.log("Authority:", (config as any).authority.toBase58());
      console.log("Gateway Program:", (config as any).gatewayProgram.toBase58());
      console.log("Gateway PDA:", (config as any).gatewayPda.toBase58());
      console.log("Paused:", (config as any).isPaused);
      console.log("Next Token ID:", (config as any).nextTokenId.toString());
    } catch (e) {
      console.log("Config not initialized or fetch failed:", (e as Error).message);
    }
  }

  async origin(tokenIdHex: string): Promise<void> {
    const tokenId = Buffer.from(tokenIdHex.replace(/^0x/, ''), 'hex');
    const pda = this.deriveNftOriginPda(tokenId);
    const origin = await this.program.account.nftOrigin.fetch(pda);
    console.log("Origin:", {
      mint: (origin as any).originalMint.toBase58(),
      metadata: (origin as any).originalMetadata.toBase58(),
      uri: (origin as any).originalUri as string,
      isOnSolana: (origin as any).isOnSolana as boolean,
    });
  }
}

async function main() {
  const [cmd, ...args] = process.argv.slice(2);
  const cli = new UniversalNftCLI();
  try {
    switch (cmd) {
      case "initialize": {
        const gateway = args[0];
        const gatewayPda = args[1];
        if (!gateway || !gatewayPda) throw new Error("usage: initialize <gatewayProgramPubkey> <gatewayPdaPubkey>");
        await cli.initialize(gateway, gatewayPda);
        break;
      }
      case "mint": {
        const uri = args[0];
        const name = args[1] || "Universal NFT";
        const symbol = args[2] || "UNFT";
        if (!uri) throw new Error("usage: mint <uri> [name] [symbol]");
        await cli.mint(uri, name, symbol);
        break;
      }
      case "transfer": {
        const [tokenIdHex, zcUniversalContractHex, destChain, finalRecipient] = args;
        if (!tokenIdHex || !zcUniversalContractHex || !destChain || !finalRecipient) {
          throw new Error("usage: transfer <tokenIdHex32> <zcUniversalContractHex20> <destChainU64> <finalRecipientStr>");
        }
        await cli.transferToZetachain(tokenIdHex, zcUniversalContractHex, destChain, finalRecipient);
        break;
      }
      case "status":
        await cli.status();
        break;
      case "origin": {
        const tokenIdHex = args[0];
        if (!tokenIdHex) throw new Error("usage: origin <tokenIdHex32>");
        await cli.origin(tokenIdHex);
        break;
      }
      case "balance":
        await cli.balance();
        break;
      default:
        console.log("Commands:\n  initialize <gatewayProgramPubkey> <gatewayPdaPubkey>\n  mint <uri> [name] [symbol]\n  transfer <tokenIdHex32> <zcUniversalContractHex20> <destChainU64> <finalRecipient>\n  status\n  origin <tokenIdHex32>\n  balance");
    }
  } catch (e) {
    console.error("Error:", (e as Error).message);
    process.exitCode = 1;
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}
