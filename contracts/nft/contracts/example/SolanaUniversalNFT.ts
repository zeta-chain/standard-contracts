#!/usr/bin/env node

import { Connection, PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY, ComputeBudgetProgram } from "@solana/web3.js";
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
      [Buffer.from("connected")],
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

  async updateConfigCli(newGatewayProgramArg?: string, newGatewayPdaArg?: string, newAuthorityArg?: string, pauseArg?: string): Promise<void> {
    const toOptPubkey = (v?: string | null) => {
      if (!v || v === "-") return null;
      return new PublicKey(v);
    };
    const toOptBool = (v?: string | null) => {
      if (!v || v === "-") return null;
      if (v === "true" || v === "1") return true;
      if (v === "false" || v === "0") return false;
      throw new Error("pause must be true|false|- (empty)");
    };

    const configPda = this.deriveConfigPda();
    const newGatewayProgram = toOptPubkey(newGatewayProgramArg);
    const newGatewayPda = toOptPubkey(newGatewayPdaArg);
    const newAuthority = toOptPubkey(newAuthorityArg);
    const pause = toOptBool(pauseArg);

    const tx = await this.program.methods
      .updateConfig(newAuthority, newGatewayProgram, newGatewayPda, pause)
      .accountsStrict({
        config: configPda,
        authority: this.wallet.publicKey,
      })
      .rpc();
    console.log("Config updated. Tx:", tx);
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

    // Increase CU limit for heavy Metaplex CPIs (metadata + master edition)
    const computeIxs = [
      ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
    ];

    try {
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
        .preInstructions(computeIxs)
        .signers([mint])
        .rpc();

      console.log("Minted. Tx:", tx);
      console.log("Mint:", mint.publicKey.toBase58());
      console.log("Metadata:", metadataPda.toBase58());
      console.log("Token ID:", tokenId.toString("hex"));
    } catch (e: any) {
      console.error("mint failed:", e?.message || e);
      if (e?.stack) console.error(e.stack);
      if (e?.logs) console.error("Transaction logs:\n" + (e.logs as string[]).join("\n"));
      else if (typeof e?.getLogs === "function") {
        try {
          const logs = await e.getLogs();
          if (logs) console.error("Transaction logs:\n" + logs.join("\n"));
        } catch {}
      }
      throw e;
    }
  }

  async transferToZetachain(
    tokenIdHex: string,
    zcUniversalContractHex: string,
    destinationZrc20HexOrZero: string,
    finalRecipient: string,
    depositSol?: string
  ): Promise<void> {
    // Strict arg validations
    const tokenId = Buffer.from(tokenIdHex.replace(/^0x/, ''), 'hex');
    if (tokenId.length !== 32) throw new Error(`tokenId must be 32 bytes hex (got ${tokenId.length})`);
    const receiver = Buffer.from(zcUniversalContractHex.replace(/^0x/, ''), 'hex');
    if (receiver.length !== 20) throw new Error(`zetachain universal contract must be 20 bytes hex (got ${receiver.length})`);
    // destinationZrc20: accept '0' (stay on ZetaChain) or 20-byte hex address
    const destRaw = destinationZrc20HexOrZero.trim();
    let destinationZrc20: Buffer;
    if (destRaw === '0' || /^0x?0+$/.test(destRaw)) {
      destinationZrc20 = Buffer.alloc(20, 0);
    } else {
      const destBuf = Buffer.from(destRaw.replace(/^0x/, ''), 'hex');
      if (destBuf.length !== 20) throw new Error(`destinationZrc20 must be 20 bytes hex or '0' (got ${destBuf.length})`);
      destinationZrc20 = destBuf;
    }
    if (!finalRecipient) throw new Error("finalRecipient must be provided as a 20-byte EVM address (hex)");
    const recRaw = finalRecipient.trim();
    const recBuf = Buffer.from(recRaw.replace(/^0x/, ''), 'hex');
    if (recBuf.length !== 20) throw new Error(`finalRecipient must be a 20-byte hex EVM address (got ${recBuf.length} bytes)`);

    // Convert deposit SOL to lamports, default 0.02 SOL if not provided
    const depositSolStr = depositSol ?? "0.02";
    if (!/^\d*(?:\.\d+)?$/.test(depositSolStr)) throw new Error("depositSol must be a number (SOL), e.g., 0.02");
    const lamportsBig = this.solToLamportsBig(depositSolStr);
    if (lamportsBig < 2_000_000n) {
      throw new Error(`depositSol too low; must be >= 0.002 SOL to cover deposit fee (got ${depositSolStr} SOL)`);
    }

    const configPda = this.deriveConfigPda();
    const nftOriginPda = this.deriveNftOriginPda(tokenId);
    const origin = await this.program.account.nftOrigin.fetch(nftOriginPda);
    const mint = origin.originalMint as PublicKey;
    const metadataPda = origin.originalMetadata as PublicKey;
    const ata = await getAssociatedTokenAddress(mint, this.wallet.publicKey);
    const config = await this.program.account.universalNftConfig.fetch(configPda);
    const gatewayProgram = config.gatewayProgram as PublicKey;
    const gatewayPda = config.gatewayPda as PublicKey;

    // Optional diagnostics to catch IDL/program mismatches
    const idlAddr = (idl as any)?.address;
    if (idlAddr) {
      const idlInstr = (idl as any)?.instructions?.find((i: any) => i.name === 'transfer_to_zetachain' || i.name === 'transferToZetachain');
      console.log('IDL program address:', idlAddr);
      console.log('Loaded program address:', this.program.programId.toBase58());
      if (idlInstr) console.log('IDL transferToZetachain args:', idlInstr.args?.map((a: any) => `${a.name}:${JSON.stringify(a.type)}`));
    }

    // Ensure arrays are exact-sized
    const tokenIdArr = Array.from(tokenId);
    const receiverArr = Array.from(receiver);
    const destinationZrc20Arr = Array.from(destinationZrc20);
    if (tokenIdArr.length !== 32 || receiverArr.length !== 20 || destinationZrc20Arr.length !== 20) {
      throw new Error('Invalid arg lengths after conversion');
    }

    try {
      const tx = await this.program.methods
        .transferToZetachain(
          tokenIdArr,
          receiverArr,
          destinationZrc20Arr,
          finalRecipient,
          new BN(lamportsBig.toString())
        )
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
        .preInstructions([
          // Give headroom for burn + gateway CPI
          ComputeBudgetProgram.setComputeUnitLimit({ units: 600_000 }),
        ])
        .rpc();
      console.log("Transfer initiated. Tx:", tx);
    } catch (e: any) {
      // Surface common client-side coder errors and on-chain logs
      console.error('transferToZetachain failed:', e?.message || e);
      if (e?.stack) console.error(e.stack);
      if (e?.logs) console.error('Transaction logs:\n' + (e.logs as string[]).join('\n'));
      else if (typeof e?.getLogs === 'function') {
        try {
          const logs = await e.getLogs();
          if (logs) console.error('Transaction logs:\n' + logs.join('\n'));
        } catch {}
      }
      // Helpful hint for Buffer offset errors
      if ((e?.message || '').includes('offset') && (e?.message || '').includes('out of range')) {
        console.error('Hint: This often indicates an IDL/method args mismatch. Ensure anchor build was run and the CLI loads the fresh IDL/types for this deployed program.');
      }
      throw e;
    }
  }

  private solToLamportsBig(solStr: string): bigint {
    // Avoid floating errors: split integer and fractional parts
    const [intPart, fracPartRaw] = solStr.split(".");
    const fracPart = (fracPartRaw || "").padEnd(9, "0").slice(0, 9); // 9 dp for lamports
    const intLamports = BigInt(intPart || "0") * 1_000_000_000n;
    const fracLamports = BigInt(fracPart || "0");
    return intLamports + fracLamports;
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
        const [tokenIdHex, zcUniversalContractHex, destZrc20, finalRecipient, depositSol] = args;
        if (!tokenIdHex || !zcUniversalContractHex || destZrc20 == null || !finalRecipient) {
          throw new Error("usage: transfer <tokenIdHex32> <zcUniversalContractHex20> <destZrc20Hex20|'0'> <finalRecipientStr> [depositSol default 0.02]");
        }
        await cli.transferToZetachain(tokenIdHex, zcUniversalContractHex, destZrc20, finalRecipient, depositSol);
        break;
      }
      case "update-config": {
        // usage: update-config <newGatewayProgram|- or empty> <newGatewayPda|- or empty> [newAuthority|-] [pause true|false|-]
        const [ngp, ngpda, na, p] = args;
        await cli.updateConfigCli(ngp, ngpda, na, p);
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
  console.log("Commands:\n  initialize <gatewayProgramPubkey> <gatewayPdaPubkey>\n  update-config <newGatewayProgramPubkey|- or empty> <newGatewayPdaPubkey|- or empty> [newAuthorityPubkey|-] [pause true|false|-]\n  mint <uri> [name] [symbol]\n  transfer <tokenIdHex32> <zcUniversalContractHex20> <destZrc20Hex20|'0'> <finalRecipient> [depositSol default 0.02]\n  status\n  origin <tokenIdHex32>\n  balance");
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
