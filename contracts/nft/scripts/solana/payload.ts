/**
 * Payload encoder for Solana Universal NFT Program
 * 
 * This module provides utilities for encoding cross-chain NFT transfer payloads
 * in a Borsh-compatible format for use with the ZetaChain Gateway.
 */
import { PublicKey } from '@solana/web3.js';

/**
 * Payload interface for cross-chain NFT transfers
 */
export interface Payload {
  /** Unique identifier for the NFT (32 bytes) */
  token_id: Uint8Array;
  /** NFT name */
  name: string;
  /** NFT symbol */
  symbol: string;
  /** Metadata URI */
  uri: string;
  /** Recipient Solana address (base58 encoded) */
  recipient: string;
  /** Origin chain identifier */
  origin_chain_id: number;
  /** Unique nonce to prevent replay attacks */
  nonce: bigint;
  /** Original Solana mint address if previously minted on Solana */
  original_solana_mint?: string | null;
}

/**
 * Validates and converts a base58 encoded public key string to a Buffer
 * @param pubkeyStr Base58 encoded Solana public key
 * @returns Buffer containing the 32-byte public key
 * @throws Error if the pubkey is invalid
 */
export function parsePubkey(pubkeyStr: string): Buffer {
  try {
    const pubkey = new PublicKey(pubkeyStr);
    return Buffer.from(pubkey.toBytes());
  } catch (error) {
    throw new Error(`Invalid public key: ${pubkeyStr}`);
  }
}

/**
 * Encodes a string in Borsh format (u32 length + UTF-8 bytes)
 * @param str String to encode
 * @returns Buffer containing the encoded string
 */
function encodeString(str: string): Buffer {
  const strBuffer = Buffer.from(str, 'utf8');
  const lenBuffer = Buffer.alloc(4);
  lenBuffer.writeUInt32LE(strBuffer.length, 0);
  return Buffer.concat([lenBuffer, strBuffer]);
}

/**
 * Encodes a u32 value in little-endian format
 * @param value Number to encode
 * @returns Buffer containing the encoded u32
 */
function encodeU32(value: number): Buffer {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32LE(value, 0);
  return buffer;
}

/**
 * Encodes a u64 value in little-endian format
 * @param value BigInt to encode
 * @returns Buffer containing the encoded u64
 */
function encodeU64(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value, 0);
  return buffer;
}

/**
 * Encodes an optional public key (Option<Pubkey>)
 * @param pubkeyStr Optional base58 encoded public key or null
 * @returns Buffer containing the encoded option
 */
function encodeOptionPubkey(pubkeyStr: string | null | undefined): Buffer {
  if (!pubkeyStr) {
    // None variant (0)
    return Buffer.from([0]);
  }
  
  // Some variant (1) + pubkey bytes
  const pubkeyBuffer = parsePubkey(pubkeyStr);
  return Buffer.concat([Buffer.from([1]), pubkeyBuffer]);
}

/**
 * Validates a payload object
 * @param payload Payload to validate
 * @throws Error if validation fails
 */
function validatePayload(payload: Payload): void {
  if (!payload.token_id || payload.token_id.length !== 32) {
    throw new Error('token_id must be a 32-byte array');
  }
  
  if (!payload.name) {
    throw new Error('name is required');
  }
  
  if (!payload.symbol) {
    throw new Error('symbol is required');
  }
  
  if (!payload.uri) {
    throw new Error('uri is required');
  }
  
  if (!payload.recipient) {
    throw new Error('recipient is required');
  }
  
  // Validate recipient is a valid Solana address
  try {
    new PublicKey(payload.recipient);
  } catch (error) {
    throw new Error(`Invalid recipient address: ${payload.recipient}`);
  }
  
  if (typeof payload.origin_chain_id !== 'number') {
    throw new Error('origin_chain_id must be a number');
  }
  
  if (typeof payload.nonce !== 'bigint') {
    throw new Error('nonce must be a bigint');
  }
  
  // Validate optional original_solana_mint if provided
  if (payload.original_solana_mint) {
    try {
      new PublicKey(payload.original_solana_mint);
    } catch (error) {
      throw new Error(`Invalid original_solana_mint address: ${payload.original_solana_mint}`);
    }
  }
}

/**
 * Encodes a payload into a Borsh-compatible Buffer
 * @param payload Payload to encode
 * @returns Buffer containing the encoded payload
 */
export function encodePayload(payload: Payload): Buffer {
  // Validate the payload
  validatePayload(payload);
  
  // Encode each field
  const encodedBuffers = [
    Buffer.from(payload.token_id),               // token_id: [u8; 32]
    encodeString(payload.name),                  // name: String
    encodeString(payload.symbol),                // symbol: String
    encodeString(payload.uri),                   // uri: String
    parsePubkey(payload.recipient),              // recipient: Pubkey
    encodeU32(payload.origin_chain_id),          // origin_chain_id: u32
    encodeU64(payload.nonce),                    // nonce: u64
    encodeOptionPubkey(payload.original_solana_mint), // original_solana_mint: Option<Pubkey>
  ];
  
  // Concatenate all encoded fields
  return Buffer.concat(encodedBuffers);
}

/**
 * Creates a payload with default values for testing
 * @param overrides Fields to override in the default payload
 * @returns A payload object with default values
 */
export function createTestPayload(overrides: Partial<Payload> = {}): Payload {
  // Create a default token_id if not provided
  const token_id = overrides.token_id || new Uint8Array(32).fill(1);
  
  return {
    token_id,
    name: "Test NFT",
    symbol: "TEST",
    uri: "https://example.com/metadata.json",
    recipient: "11111111111111111111111111111111",  // Default to system program
    origin_chain_id: 1,
    nonce: BigInt(Date.now()),
    original_solana_mint: null,
    ...overrides
  };
}
