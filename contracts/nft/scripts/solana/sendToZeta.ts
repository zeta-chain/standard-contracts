#!/usr/bin/env ts-node
/**
 * Send to ZetaChain - Payload Encoder
 * 
 * This script encodes NFT transfer payload data for use with the ZetaChain Gateway.
 * It does not perform any network calls, just encodes and prints the payload.
 * 
 * Usage:
 *   ts-node sendToZeta.ts <token_id_hex_32> <name> <symbol> <uri> <recipient_base58> <origin_chain_id> [<original_solana_mint>] [<nonce>]
 * 
 * Example:
 *   ts-node sendToZeta.ts 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f "My NFT" "MNFT" "https://example.com/1.json" EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 7000
 */

import { encodePayload } from './payload';

// Parse command line arguments
const args = process.argv.slice(2);

if (args.length < 6) {
  console.error(`
Usage: ts-node sendToZeta.ts <token_id_hex_32> <name> <symbol> <uri> <recipient_base58> <origin_chain_id> [<original_solana_mint>] [<nonce>]

Arguments:
  token_id_hex_32     - 64 hex characters (32 bytes)
  name                - NFT name
  symbol              - NFT symbol
  uri                 - Metadata URI
  recipient_base58    - Solana recipient address (base58)
  origin_chain_id     - Origin chain identifier (number)
  original_solana_mint - [Optional] Original Solana mint if previously on Solana
  nonce               - [Optional] Unique nonce (defaults to current timestamp)
  `);
  process.exit(1);
}

// Parse token_id (convert from hex string to Uint8Array)
const tokenIdHex = args[0];
if (tokenIdHex.length !== 64 || !/^[0-9a-fA-F]+$/.test(tokenIdHex)) {
  console.error('Error: token_id must be exactly 64 hex characters (32 bytes)');
  process.exit(1);
}

const tokenIdBytes = new Uint8Array(32);
for (let i = 0; i < 32; i++) {
  tokenIdBytes[i] = parseInt(tokenIdHex.substring(i * 2, i * 2 + 2), 16);
}

// Parse other arguments
const name = args[1];
const symbol = args[2];
const uri = args[3];
const recipient = args[4];
const originChainId = parseInt(args[5], 10);
const originalSolanaMint = args.length > 6 ? args[6] : null;
const nonce = args.length > 7 ? BigInt(args[7]) : BigInt(Date.now());

// Create the payload
const payload = {
  token_id: tokenIdBytes,
  name,
  symbol,
  uri,
  recipient,
  origin_chain_id: originChainId,
  nonce,
  original_solana_mint: originalSolanaMint
};

try {
  // Encode the payload
  const encodedPayload = encodePayload(payload);
  
  // Convert to different formats
  const base64Payload = encodedPayload.toString('base64');
  const hexPayload = encodedPayload.toString('hex');
  
  // Create a JSON blob for gateway call
  const gatewayCallJson = {
    zrc20: "0x0000000000000000000000000000000000000000", // ZRC20_NULL for message-only calls
    amount: "0",
    message: base64Payload,
    to: "Un1v3rsa1Nft111111111111111111111111111111", // Universal NFT program ID
    // Note: In a real implementation, you would include gas parameters
  };
  
  // Print the results
  console.log('\n=== ENCODED PAYLOAD ===');
  console.log('\nBase64:');
  console.log(base64Payload);
  
  console.log('\nHex:');
  console.log(hexPayload);
  
  console.log('\n=== GATEWAY CALL JSON ===');
  console.log(JSON.stringify(gatewayCallJson, null, 2));
  
  console.log('\n=== PAYLOAD DETAILS ===');
  console.log(JSON.stringify({
    token_id: tokenIdHex,
    name,
    symbol,
    uri,
    recipient,
    origin_chain_id: originChainId,
    nonce: nonce.toString(),
    original_solana_mint: originalSolanaMint
  }, null, 2));
  
  console.log('\nTo use with ZetaChain Gateway:');
  console.log('1. Fund your account with SOL for deposit fee (0.002 SOL)');
  console.log('2. Call Gateway.depositAndCall with the above payload');
  console.log('3. Include a compute budget instruction if needed (200k+ CU recommended)');
  
} catch (error) {
  console.error('Error encoding payload:', error.message);
  process.exit(1);
}
