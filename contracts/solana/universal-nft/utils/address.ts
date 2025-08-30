// EIP-55 uses Keccak-256 (not SHA-256)
import { keccak_256 } from "@noble/hashes/sha3";
import { bytesToHex, utf8ToBytes } from "@noble/hashes/utils";

/**
 * Normalizes an Ethereum address with EIP-55 checksum validation
 * @param address - The Ethereum address (with or without 0x prefix)
 * @returns Normalized address with proper checksum
 * @throws Error if address is invalid
 */
export function normalizeEthereumAddress(address: string): string {
  // Trim, remove 0x prefix if present, and lowercase
  const cleanAddress = address.trim().replace(/^0x/i, "").toLowerCase();
  
  // Validate hex format (must be exactly 40 hex characters = 20 bytes)
  if (!/^[0-9a-f]{40}$/i.test(cleanAddress)) {
    throw new Error(
      `Invalid Ethereum address format: must be 40 hex characters (20 bytes). Got: ${cleanAddress.length} characters`
    );
  }
  
  // Zero address policy is context-specific; allow but do not log here.
  
  const checksummedAddress = toChecksumAddress(cleanAddress);
  return checksummedAddress;
}

/**
 * Converts an Ethereum address to EIP-55 checksum format
 * @param address - Lowercase hex address (without 0x prefix)
 * @returns Address with EIP-55 checksum
 */
function toChecksumAddress(address: string): string {
  const lower = address.toLowerCase(); // 40 hex chars, no 0x
  const hash = bytesToHex(keccak_256(utf8ToBytes(lower)));
  let out = "0x";
  for (let i = 0; i < lower.length; i++) {
    out += parseInt(hash[i], 16) >= 8 ? lower[i].toUpperCase() : lower[i];
  }
  return out;
}

/**
 * Validates an EIP-55 checksum address
 * @param address - The address to validate
 * @returns true if checksum is valid, false otherwise
 */
export function isValidChecksumAddress(address: string): boolean {
  const cleanAddress = address.replace(/^0x/i, "");
  if (!/^[0-9a-fA-F]{40}$/.test(cleanAddress)) {
    return false;
  }
  
  // If all lowercase or all uppercase, it's valid but not checksummed
  if (cleanAddress === cleanAddress.toLowerCase() || cleanAddress === cleanAddress.toUpperCase()) {
    return true;
  }
  
  // Validate checksum
  const expectedChecksum = toChecksumAddress(cleanAddress.toLowerCase());
  return address === expectedChecksum;
}

/**
 * Converts a normalized Ethereum address to a Buffer for Solana usage
 * @param address - Normalized Ethereum address (with 0x prefix)
 * @returns Buffer containing the 20-byte address
 */
export function ethereumAddressToBuffer(address: string): Buffer {
  const normalized = normalizeEthereumAddress(address);
  return Buffer.from(normalized.slice(2), "hex");
}

/**
 * Converts a Buffer to a normalized Ethereum address string
 * @param buffer - 20-byte buffer containing the address
 * @returns Normalized Ethereum address with checksum
 */
export function bufferToEthereumAddress(buffer: Buffer): string {
  if (buffer.length !== 20) {
    throw new Error(`Ethereum address buffer must be 20 bytes. Got: ${buffer.length} bytes`);
  }
  
  const hexString = buffer.toString("hex");
  return toChecksumAddress(hexString);
}

/**
 * Validates and returns a consistent EVM address array for Solana programs
 * @param address - The Ethereum address string
 * @returns Array of numbers representing the 20-byte address
 */
export function getEvmAddressArray(address: string): number[] {
  const buffer = ethereumAddressToBuffer(address);
  return Array.from(buffer);
}