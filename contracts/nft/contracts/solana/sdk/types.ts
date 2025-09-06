/**
 * Universal NFT Program TypeScript Type Definitions
 * 
 * This module provides comprehensive type definitions for the Universal NFT program,
 * including all program accounts, instruction parameters, event types, and cross-chain
 * message formats. It also includes utility types and validation functions for
 * runtime type checking and developer experience.
 */

import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

// ============================================================================
// Program Constants
// ============================================================================

/** Universal NFT Program ID */
export const UNIVERSAL_NFT_PROGRAM_ID = new PublicKey('6RfVUT361yLWutQFXBdBmNCCFxiaj5XjC4LS7XrQYuke');

/** Metaplex Token Metadata Program ID */
export const TOKEN_METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

/** ZetaChain Gateway Program ID */
export const ZETACHAIN_GATEWAY_PROGRAM_ID = new PublicKey('ZETAjseVjuFsxdRxo6MmTCvqFwb3ZHUx56Co3vCmGis');

// ============================================================================
// Chain ID Constants
// ============================================================================

/** Supported blockchain chain IDs */
export const CHAIN_IDS = {
  // Mainnet chains
  ZETACHAIN: 7000,
  ETHEREUM: 1,
  BSC: 56,
  POLYGON: 137,
  BASE: 8453,
  ARBITRUM: 42161,
  OPTIMISM: 10,
  
  // Testnet chains
  SEPOLIA: 11155111,
  BSC_TESTNET: 97,
  MUMBAI: 80001,
  BASE_SEPOLIA: 84532,
  ARBITRUM_SEPOLIA: 421614,
  OPTIMISM_SEPOLIA: 11155420,
  ZETACHAIN_TESTNET: 7001,
  
  // Solana chains
  SOLANA_MAINNET: 101,
  SOLANA_TESTNET: 102,
  SOLANA_DEVNET: 103,
} as const;

export type ChainId = typeof CHAIN_IDS[keyof typeof CHAIN_IDS];

// ============================================================================
// Address Types
// ============================================================================

/** Ethereum-compatible address (20 bytes) */
export type EvmAddress = Uint8Array & { readonly length: 20 };

/** Solana address (32 bytes) */
export type SolanaAddress = Uint8Array & { readonly length: 32 };

/** TSS ECDSA address (20 bytes) */
export type TssAddress = Uint8Array & { readonly length: 20 };

/** Generic cross-chain address */
export type CrossChainAddress = EvmAddress | SolanaAddress;

// ============================================================================
// Program Account Types
// ============================================================================

/**
 * Universal NFT Collection account
 * 
 * Represents a collection of Universal NFTs that can be transferred
 * across multiple blockchains while maintaining origin tracking.
 */
export interface Collection {
  /** Collection authority public key */
  authority: PublicKey;
  /** Collection name (max 32 characters) */
  name: string;
  /** Collection symbol (max 10 characters) */
  symbol: string;
  /** Collection metadata URI (max 200 characters) */
  uri: string;
  /** ZetaChain TSS ECDSA address for signature verification */
  tssAddress: TssAddress;
  /** Optional gateway PDA for ZetaChain integration */
  gatewayAddress?: PublicKey;
  /** Optional universal contract address on ZetaChain */
  universalAddress?: PublicKey;
  /** Next token ID to be minted */
  nextTokenId: BN;
  /** Replay protection counter */
  nonce: BN;
  /** Total number of NFTs minted in this collection */
  totalMinted: BN;
  /** Number of Solana-native NFTs */
  solanaNativeCount: BN;
  /** PDA bump seed */
  bump: number;
}

/**
 * NFT Origin tracking account
 * 
 * Tracks the origin and lifecycle of each Universal NFT across chains.
 * This enables the two-scenario reception logic for returning vs new NFTs.
 */
export interface NftOrigin {
  /** Original mint public key when NFT was first created */
  originalMint: PublicKey;
  /** Universal token ID used across all chains */
  tokenId: BN;
  /** Reference to the parent collection */
  collection: PublicKey;
  /** Chain ID where the NFT was originally minted */
  chainOfOrigin: ChainId;
  /** Timestamp when the NFT was created */
  createdAt: BN;
  /** Original metadata URI (max 200 characters) */
  metadataUri: string;
  /** PDA bump seed */
  bump: number;
}

/**
 * Connected chain configuration
 * 
 * Stores contract addresses for connected chains in the Universal NFT system.
 */
export interface Connected {
  /** Reference to the parent collection */
  collection: PublicKey;
  /** Chain ID as bytes (max 32 bytes) */
  chainId: Uint8Array;
  /** Contract address on the connected chain (max 64 bytes) */
  contractAddress: Uint8Array;
  /** PDA bump seed */
  bump: number;
}

// ============================================================================
// Cross-Chain Message Types
// ============================================================================

/**
 * Generic cross-chain message format
 */
export interface CrossChainMessage {
  /** Destination chain identifier */
  destinationChain: Uint8Array;
  /** Recipient address on destination chain */
  recipient: Uint8Array;
  /** Universal token ID */
  tokenId: BN;
  /** NFT metadata URI */
  uri: string;
  /** Sender address */
  sender: Uint8Array;
}

/**
 * ZetaChain-specific message format
 */
export interface ZetaChainMessage {
  /** Destination chain ID */
  destinationChainId: ChainId;
  /** Destination address (20 bytes for EVM) */
  destinationAddress: EvmAddress;
  /** Gas limit for destination chain execution */
  destinationGasLimit: BN;
  /** Encoded message payload */
  message: Uint8Array;
  /** Universal token ID */
  tokenId: BN;
  /** NFT metadata URI */
  uri: string;
  /** Sender address (32 bytes) */
  sender: SolanaAddress;
}

/**
 * EVM-compatible message format
 */
export interface EvmMessage {
  /** Universal token ID */
  tokenId: BN;
  /** Recipient EVM address */
  recipient: EvmAddress;
  /** NFT metadata URI */
  uri: string;
  /** Sender EVM address */
  sender: EvmAddress;
}

/**
 * Revert context for failed cross-chain transfers
 */
export interface RevertContext {
  /** Universal token ID */
  tokenId: BN;
  /** NFT metadata URI */
  uri: string;
  /** Original sender address */
  originalSender: Uint8Array;
  /** Reason for revert */
  revertReason: string;
  /** Revert message payload */
  revertMessage: Uint8Array;
}

// ============================================================================
// Instruction Parameter Types
// ============================================================================

/**
 * Parameters for initializing a new collection
 */
export interface InitializeCollectionParams {
  /** Collection name */
  name: string;
  /** Collection symbol */
  symbol: string;
  /** Collection metadata URI */
  uri: string;
  /** TSS address for signature verification */
  tssAddress: TssAddress;
}

/**
 * Parameters for minting a new NFT
 */
export interface MintNftParams {
  /** NFT name */
  name: string;
  /** NFT symbol */
  symbol: string;
  /** NFT metadata URI */
  uri: string;
}

/**
 * Parameters for cross-chain transfer
 */
export interface TransferCrossChainParams {
  /** Destination chain ID */
  destinationChainId: ChainId;
  /** Recipient address on destination chain */
  recipient: Uint8Array;
}

/**
 * Parameters for handling incoming cross-chain calls
 */
export interface OnCallParams {
  /** Sender address (20 bytes) */
  sender: EvmAddress;
  /** Source chain ID */
  sourceChainId: ChainId;
  /** Message payload */
  message: Uint8Array;
  /** Nonce for replay protection */
  nonce: BN;
}

/**
 * Parameters for receiving cross-chain transfers with TSS verification
 */
export interface ReceiveCrossChainParams {
  /** Message hash for signature verification */
  messageHash: Uint8Array & { readonly length: 32 };
  /** TSS signature (64 bytes) */
  signature: Uint8Array & { readonly length: 64 };
  /** Recovery ID for signature verification */
  recoveryId: number;
  /** Message data payload */
  messageData: Uint8Array;
  /** Nonce for replay protection */
  nonce: BN;
}

/**
 * Parameters for setting universal contract address
 */
export interface SetUniversalParams {
  /** Universal contract address on ZetaChain */
  universalAddress: PublicKey;
}

/**
 * Parameters for setting connected chain configuration
 */
export interface SetConnectedParams {
  /** Chain ID */
  chainId: Uint8Array;
  /** Contract address on the chain */
  contractAddress: Uint8Array;
}

/**
 * Parameters for handling reverted transfers
 */
export interface OnRevertParams {
  /** Universal token ID */
  tokenId: BN;
  /** NFT metadata URI */
  uri: string;
  /** Original sender to refund */
  originalSender: PublicKey;
  /** Refund amount */
  refundAmount: BN;
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * Event emitted when a collection is initialized
 */
export interface CollectionInitializedEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Collection authority */
  authority: PublicKey;
  /** Collection name */
  name: string;
  /** Collection symbol */
  symbol: string;
  /** TSS address */
  tssAddress: TssAddress;
}

/**
 * Event emitted when an NFT is minted
 */
export interface TokenMintedEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Universal token ID */
  tokenId: BN;
  /** NFT mint public key */
  mint: PublicKey;
  /** Recipient public key */
  recipient: PublicKey;
  /** NFT name */
  name: string;
  /** NFT metadata URI */
  uri: string;
  /** Origin chain ID */
  originChain: ChainId;
  /** Whether the NFT is native to Solana */
  isSolanaNative: boolean;
}

/**
 * Event emitted when an NFT is transferred cross-chain
 */
export interface TokenTransferEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Universal token ID */
  tokenId: BN;
  /** Destination chain ID */
  destinationChainId: ChainId;
  /** Recipient address */
  recipient: Uint8Array;
  /** NFT metadata URI */
  uri: string;
  /** Sender public key */
  sender: PublicKey;
  /** Message payload */
  message: Uint8Array;
  /** Origin chain ID (if known) */
  originChain?: ChainId;
  /** Original mint (if returning to Solana) */
  originalMint?: PublicKey;
  /** Whether this is a returning NFT */
  isReturning: boolean;
}

/**
 * Event emitted when a cross-chain transfer is received
 */
export interface TokenTransferReceivedEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Universal token ID */
  tokenId: BN;
  /** Recipient public key */
  recipient: PublicKey;
  /** NFT metadata URI */
  uri: string;
  /** Original sender address */
  originalSender: Uint8Array;
  /** Nonce used */
  nonce: BN;
  /** Origin chain ID (if known) */
  originChain?: ChainId;
  /** Original mint (if returning to Solana) */
  originalMint?: PublicKey;
  /** Whether this is a returning NFT */
  isReturning: boolean;
}

/**
 * Event emitted when a transfer is reverted
 */
export interface TokenTransferRevertedEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Universal token ID */
  tokenId: BN;
  /** Sender public key */
  sender: PublicKey;
  /** NFT metadata URI */
  uri: string;
  /** Refund amount */
  refundAmount: BN;
  /** Origin chain ID (if known) */
  originChain?: ChainId;
  /** Original mint (if applicable) */
  originalMint?: PublicKey;
}

/**
 * Event emitted when NFT Origin is created
 */
export interface NftOriginCreatedEvent {
  /** Universal token ID */
  tokenId: BN;
  /** Original mint public key */
  originalMint: PublicKey;
  /** Collection public key */
  collection: PublicKey;
  /** Origin chain ID */
  originChain: ChainId;
  /** Metadata URI */
  metadataUri: string;
}

/**
 * Event emitted when NFT Origin is updated
 */
export interface NftOriginUpdatedEvent {
  /** Universal token ID */
  tokenId: BN;
  /** Original mint public key */
  originalMint: PublicKey;
  /** List of updated fields */
  updatedFields: string[];
}

/**
 * Event emitted when NFT returns to Solana
 */
export interface NftReturningToSolanaEvent {
  /** Universal token ID */
  tokenId: BN;
  /** Original mint public key */
  originalMint: PublicKey;
  /** New mint public key */
  newMint: PublicKey;
  /** Whether metadata was preserved */
  metadataPreserved: boolean;
}

/**
 * Event emitted when a cross-chain cycle is completed
 */
export interface CrossChainCycleCompletedEvent {
  /** Universal token ID */
  tokenId: BN;
  /** Origin chain ID */
  originChain: ChainId;
  /** Destination chain ID */
  destinationChain: ChainId;
  /** Number of cycles completed */
  cycleCount: BN;
}

/**
 * Event emitted when universal address is set
 */
export interface SetUniversalEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Universal contract address */
  universalAddress: PublicKey;
}

/**
 * Event emitted when connected chain is configured
 */
export interface SetConnectedEvent {
  /** Collection public key */
  collection: PublicKey;
  /** Chain ID */
  chainId: Uint8Array;
  /** Contract address */
  contractAddress: Uint8Array;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Universal NFT program error codes
 */
export enum UniversalNftErrorCode {
  InvalidTssSignature = 6000,
  InvalidMessageHash = 6001,
  InvalidMessage = 6002,
  InvalidRecipient = 6003,
  InvalidSignature = 6004,
  UnauthorizedTssAddress = 6005,
  InvalidNonce = 6006,
  TokenDoesNotExist = 6007,
  NotTokenOwner = 6008,
  InvalidDestinationChain = 6009,
  InvalidRecipientAddress = 6010,
  InsufficientGasAmount = 6011,
  UnauthorizedGateway = 6012,
  UnsupportedChain = 6013,
  InvalidTokenId = 6014,
}

/**
 * Universal NFT program error
 */
export interface UniversalNftError {
  /** Error code */
  code: UniversalNftErrorCode;
  /** Error message */
  message: string;
  /** Additional error context */
  context?: Record<string, any>;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Chain configuration for different networks
 */
export interface ChainConfig {
  /** Chain ID */
  chainId: ChainId;
  /** Chain name */
  name: string;
  /** Whether it's a testnet */
  isTestnet: boolean;
  /** Address format (EVM or Solana) */
  addressFormat: 'evm' | 'solana';
  /** Expected address length in bytes */
  addressLength: 20 | 32;
  /** Base gas cost for transactions */
  baseGasCost: number;
  /** RPC endpoint (optional) */
  rpcEndpoint?: string;
}

/**
 * NFT metadata following Metaplex standard
 */
export interface NftMetadata {
  /** NFT name */
  name: string;
  /** NFT symbol */
  symbol: string;
  /** Metadata URI */
  uri: string;
  /** Seller fee basis points */
  sellerFeeBasisPoints: number;
  /** Creators array */
  creators?: Array<{
    address: PublicKey;
    verified: boolean;
    share: number;
  }>;
  /** Collection information */
  collection?: {
    verified: boolean;
    key: PublicKey;
  };
  /** Uses information */
  uses?: {
    useMethod: 'burn' | 'multiple' | 'single';
    remaining: number;
    total: number;
  };
}

/**
 * PDA derivation result
 */
export interface PdaResult {
  /** Derived public key */
  publicKey: PublicKey;
  /** Bump seed */
  bump: number;
}

/**
 * Transaction result with additional context
 */
export interface TransactionResult {
  /** Transaction signature */
  signature: string;
  /** Whether transaction was successful */
  success: boolean;
  /** Error message if failed */
  error?: string;
  /** Additional context */
  context?: Record<string, any>;
}

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Type guard to check if a value is a valid chain ID
 */
export function isValidChainId(value: any): value is ChainId {
  return typeof value === 'number' && Object.values(CHAIN_IDS).includes(value);
}

/**
 * Type guard to check if an address is EVM format
 */
export function isEvmAddress(address: Uint8Array): address is EvmAddress {
  return address.length === 20;
}

/**
 * Type guard to check if an address is Solana format
 */
export function isSolanaAddress(address: Uint8Array): address is SolanaAddress {
  return address.length === 32;
}

/**
 * Type guard to check if a chain uses EVM addresses
 */
export function isEvmChain(chainId: ChainId): boolean {
  return ![CHAIN_IDS.SOLANA_MAINNET, CHAIN_IDS.SOLANA_TESTNET, CHAIN_IDS.SOLANA_DEVNET].includes(chainId);
}

/**
 * Type guard to check if a chain is a testnet
 */
export function isTestnetChain(chainId: ChainId): boolean {
  return [
    CHAIN_IDS.SEPOLIA,
    CHAIN_IDS.BSC_TESTNET,
    CHAIN_IDS.MUMBAI,
    CHAIN_IDS.BASE_SEPOLIA,
    CHAIN_IDS.ARBITRUM_SEPOLIA,
    CHAIN_IDS.OPTIMISM_SEPOLIA,
    CHAIN_IDS.ZETACHAIN_TESTNET,
    CHAIN_IDS.SOLANA_TESTNET,
    CHAIN_IDS.SOLANA_DEVNET,
  ].includes(chainId);
}

// ============================================================================
// Validation Functions
// ============================================================================

/**
 * Validate EVM address format
 */
export function validateEvmAddress(address: Uint8Array): EvmAddress {
  if (address.length !== 20) {
    throw new Error(`Invalid EVM address length: expected 20 bytes, got ${address.length}`);
  }
  return address as EvmAddress;
}

/**
 * Validate Solana address format
 */
export function validateSolanaAddress(address: Uint8Array): SolanaAddress {
  if (address.length !== 32) {
    throw new Error(`Invalid Solana address length: expected 32 bytes, got ${address.length}`);
  }
  return address as SolanaAddress;
}

/**
 * Validate TSS address format
 */
export function validateTssAddress(address: Uint8Array): TssAddress {
  if (address.length !== 20) {
    throw new Error(`Invalid TSS address length: expected 20 bytes, got ${address.length}`);
  }
  return address as TssAddress;
}

/**
 * Validate chain ID
 */
export function validateChainId(chainId: number): ChainId {
  if (!isValidChainId(chainId)) {
    throw new Error(`Unsupported chain ID: ${chainId}`);
  }
  return chainId;
}

/**
 * Validate message hash format
 */
export function validateMessageHash(hash: Uint8Array): Uint8Array & { readonly length: 32 } {
  if (hash.length !== 32) {
    throw new Error(`Invalid message hash length: expected 32 bytes, got ${hash.length}`);
  }
  return hash as Uint8Array & { readonly length: 32 };
}

/**
 * Validate signature format
 */
export function validateSignature(signature: Uint8Array): Uint8Array & { readonly length: 64 } {
  if (signature.length !== 64) {
    throw new Error(`Invalid signature length: expected 64 bytes, got ${signature.length}`);
  }
  return signature as Uint8Array & { readonly length: 64 };
}

/**
 * Validate recovery ID
 */
export function validateRecoveryId(recoveryId: number): number {
  if (recoveryId < 0 || recoveryId > 3) {
    throw new Error(`Invalid recovery ID: expected 0-3, got ${recoveryId}`);
  }
  return recoveryId;
}

/**
 * Validate URI format
 */
export function validateUri(uri: string, maxLength: number = 200): string {
  if (uri.length > maxLength) {
    throw new Error(`URI too long: expected max ${maxLength} characters, got ${uri.length}`);
  }
  if (!uri.startsWith('http://') && !uri.startsWith('https://') && !uri.startsWith('ipfs://') && !uri.startsWith('ar://')) {
    throw new Error(`Invalid URI format: must start with http://, https://, ipfs://, or ar://`);
  }
  return uri;
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get chain configuration by chain ID
 */
export function getChainConfig(chainId: ChainId): ChainConfig {
  const configs: Record<ChainId, ChainConfig> = {
    [CHAIN_IDS.ETHEREUM]: {
      chainId: CHAIN_IDS.ETHEREUM,
      name: 'Ethereum',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 150000,
    },
    [CHAIN_IDS.BSC]: {
      chainId: CHAIN_IDS.BSC,
      name: 'BSC',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 80000,
    },
    [CHAIN_IDS.POLYGON]: {
      chainId: CHAIN_IDS.POLYGON,
      name: 'Polygon',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 80000,
    },
    [CHAIN_IDS.BASE]: {
      chainId: CHAIN_IDS.BASE,
      name: 'Base',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.ARBITRUM]: {
      chainId: CHAIN_IDS.ARBITRUM,
      name: 'Arbitrum',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.OPTIMISM]: {
      chainId: CHAIN_IDS.OPTIMISM,
      name: 'Optimism',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.ZETACHAIN]: {
      chainId: CHAIN_IDS.ZETACHAIN,
      name: 'ZetaChain',
      isTestnet: false,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 50000,
    },
    [CHAIN_IDS.SEPOLIA]: {
      chainId: CHAIN_IDS.SEPOLIA,
      name: 'Ethereum Sepolia',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 150000,
    },
    [CHAIN_IDS.BSC_TESTNET]: {
      chainId: CHAIN_IDS.BSC_TESTNET,
      name: 'BSC Testnet',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 80000,
    },
    [CHAIN_IDS.MUMBAI]: {
      chainId: CHAIN_IDS.MUMBAI,
      name: 'Polygon Mumbai',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 80000,
    },
    [CHAIN_IDS.BASE_SEPOLIA]: {
      chainId: CHAIN_IDS.BASE_SEPOLIA,
      name: 'Base Sepolia',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.ARBITRUM_SEPOLIA]: {
      chainId: CHAIN_IDS.ARBITRUM_SEPOLIA,
      name: 'Arbitrum Sepolia',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.OPTIMISM_SEPOLIA]: {
      chainId: CHAIN_IDS.OPTIMISM_SEPOLIA,
      name: 'Optimism Sepolia',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 100000,
    },
    [CHAIN_IDS.ZETACHAIN_TESTNET]: {
      chainId: CHAIN_IDS.ZETACHAIN_TESTNET,
      name: 'ZetaChain Testnet',
      isTestnet: true,
      addressFormat: 'evm',
      addressLength: 20,
      baseGasCost: 50000,
    },
    [CHAIN_IDS.SOLANA_MAINNET]: {
      chainId: CHAIN_IDS.SOLANA_MAINNET,
      name: 'Solana Mainnet',
      isTestnet: false,
      addressFormat: 'solana',
      addressLength: 32,
      baseGasCost: 10000000, // 0.01 SOL in lamports
    },
    [CHAIN_IDS.SOLANA_TESTNET]: {
      chainId: CHAIN_IDS.SOLANA_TESTNET,
      name: 'Solana Testnet',
      isTestnet: true,
      addressFormat: 'solana',
      addressLength: 32,
      baseGasCost: 10000000,
    },
    [CHAIN_IDS.SOLANA_DEVNET]: {
      chainId: CHAIN_IDS.SOLANA_DEVNET,
      name: 'Solana Devnet',
      isTestnet: true,
      addressFormat: 'solana',
      addressLength: 32,
      baseGasCost: 10000000,
    },
  };

  const config = configs[chainId];
  if (!config) {
    throw new Error(`No configuration found for chain ID: ${chainId}`);
  }
  return config;
}

/**
 * Convert address format for cross-chain compatibility
 */
export function convertAddressFormat(
  address: Uint8Array,
  targetChain: ChainId
): Uint8Array {
  const targetConfig = getChainConfig(targetChain);
  
  if (targetConfig.addressFormat === 'evm') {
    if (address.length === 20) {
      return address;
    } else if (address.length === 32) {
      // For Solana to EVM, take first 20 bytes (simplified approach)
      return address.slice(0, 20);
    } else {
      throw new Error(`Cannot convert address of length ${address.length} to EVM format`);
    }
  } else if (targetConfig.addressFormat === 'solana') {
    if (address.length === 32) {
      return address;
    } else if (address.length === 20) {
      // For EVM to Solana, pad with zeros (simplified approach)
      const padded = new Uint8Array(32);
      padded.set(address, 12); // Place EVM address in last 20 bytes
      return padded;
    } else {
      throw new Error(`Cannot convert address of length ${address.length} to Solana format`);
    }
  }
  
  throw new Error(`Unsupported address format: ${targetConfig.addressFormat}`);
}

/**
 * Calculate gas fee for cross-chain transfer
 */
export function calculateGasFee(destinationChain: ChainId, gasAmount: BN): BN {
  const config = getChainConfig(destinationChain);
  const baseFee = new BN(config.baseGasCost);
  const totalFee = baseFee.mul(gasAmount);
  
  // Ensure minimum gas fee (0.01 SOL in lamports)
  const minFee = new BN(10_000_000);
  return BN.max(totalFee, minFee);
}

/**
 * Generate deterministic token ID
 */
export function generateTokenId(mint: PublicKey, blockNumber: BN, nextTokenId: BN): BN {
  // This is a simplified version - actual implementation would use keccak hash
  const combined = mint.toBytes().toString() + blockNumber.toString() + nextTokenId.toString();
  const hash = combined.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  return new BN(hash % Number.MAX_SAFE_INTEGER);
}

// ============================================================================
// PDA Derivation Functions
// ============================================================================

/**
 * Derive Collection PDA
 */
export function deriveCollectionPda(
  authority: PublicKey,
  name: string,
  programId: PublicKey = UNIVERSAL_NFT_PROGRAM_ID
): PdaResult {
  const [publicKey, bump] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('collection'),
      authority.toBuffer(),
      Buffer.from(name),
    ],
    programId
  );
  return { publicKey, bump };
}

/**
 * Derive NFT Origin PDA
 */
export function deriveNftOriginPda(
  tokenId: BN,
  programId: PublicKey = UNIVERSAL_NFT_PROGRAM_ID
): PdaResult {
  const [publicKey, bump] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('nft_origin'),
      tokenId.toArrayLike(Buffer, 'le', 8),
    ],
    programId
  );
  return { publicKey, bump };
}

/**
 * Derive Connected PDA
 */
export function deriveConnectedPda(
  collection: PublicKey,
  chainId: Uint8Array,
  programId: PublicKey = UNIVERSAL_NFT_PROGRAM_ID
): PdaResult {
  const [publicKey, bump] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('connected'),
      collection.toBuffer(),
      Buffer.from(chainId),
    ],
    programId
  );
  return { publicKey, bump };
}

/**
 * Derive Gateway PDA
 */
export function deriveGatewayPda(
  programId: PublicKey = ZETACHAIN_GATEWAY_PROGRAM_ID
): PdaResult {
  const [publicKey, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from('meta')],
    programId
  );
  return { publicKey, bump };
}

// ============================================================================
// Export All Types
// ============================================================================

export type {
  Collection,
  NftOrigin,
  Connected,
  CrossChainMessage,
  ZetaChainMessage,
  EvmMessage,
  RevertContext,
  InitializeCollectionParams,
  MintNftParams,
  TransferCrossChainParams,
  OnCallParams,
  ReceiveCrossChainParams,
  SetUniversalParams,
  SetConnectedParams,
  OnRevertParams,
  CollectionInitializedEvent,
  TokenMintedEvent,
  TokenTransferEvent,
  TokenTransferReceivedEvent,
  TokenTransferRevertedEvent,
  NftOriginCreatedEvent,
  NftOriginUpdatedEvent,
  NftReturningToSolanaEvent,
  CrossChainCycleCompletedEvent,
  SetUniversalEvent,
  SetConnectedEvent,
  UniversalNftError,
  ChainConfig,
  NftMetadata,
  PdaResult,
  TransactionResult,
};