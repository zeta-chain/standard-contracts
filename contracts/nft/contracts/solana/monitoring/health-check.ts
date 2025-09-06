import { Connection, PublicKey, AccountInfo, ParsedAccountData } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { createHash } from 'crypto';
import bs58 from 'bs58';

// Types for monitoring data
interface HealthMetrics {
  timestamp: number;
  programHealth: ProgramHealthStatus;
  accountHealth: AccountHealthMetrics;
  crossChainMetrics: CrossChainMetrics;
  performanceMetrics: PerformanceMetrics;
  securityMetrics: SecurityMetrics;
  alertLevel: AlertLevel;
}

interface ProgramHealthStatus {
  isOperational: boolean;
  programId: string;
  lastUpdated: number;
  errorCount: number;
  successRate: number;
  computeUnitsUsed: number;
  averageTransactionTime: number;
}

interface AccountHealthMetrics {
  collections: CollectionHealthData[];
  nftOrigins: NftOriginHealthData[];
  connectedChains: ConnectedChainData[];
  totalAccounts: number;
  healthyAccounts: number;
  corruptedAccounts: number;
}

interface CollectionHealthData {
  address: string;
  name: string;
  authority: string;
  totalMinted: number;
  solanaNativeCount: number;
  crossChainCount: number;
  nonce: number;
  lastActivity: number;
  isHealthy: boolean;
  issues: string[];
}

interface NftOriginHealthData {
  tokenId: number;
  originalMint: string;
  collection: string;
  chainOfOrigin: number;
  createdAt: number;
  isValid: boolean;
  metadataAccessible: boolean;
  issues: string[];
}

interface ConnectedChainData {
  chainId: number;
  chainName: string;
  contractAddress: string;
  isActive: boolean;
  lastSeen: number;
  transferCount: number;
  errorRate: number;
}

interface CrossChainMetrics {
  totalTransfers: number;
  successfulTransfers: number;
  failedTransfers: number;
  pendingTransfers: number;
  averageTransferTime: number;
  chainBreakdown: ChainTransferMetrics[];
  tssSignatureMetrics: TssMetrics;
  gatewayMetrics: GatewayMetrics;
}

interface ChainTransferMetrics {
  chainId: number;
  chainName: string;
  inboundTransfers: number;
  outboundTransfers: number;
  successRate: number;
  averageTime: number;
  lastTransfer: number;
}

interface TssMetrics {
  totalSignatures: number;
  validSignatures: number;
  invalidSignatures: number;
  averageVerificationTime: number;
  lastSignatureTime: number;
  tssAddress: string;
}

interface GatewayMetrics {
  isConnected: boolean;
  gatewayAddress: string;
  totalCalls: number;
  successfulCalls: number;
  failedCalls: number;
  averageResponseTime: number;
  lastCallTime: number;
}

interface PerformanceMetrics {
  rpcLatency: number;
  blockHeight: number;
  currentSlot: number;
  epochInfo: EpochMetrics;
  memoryUsage: number;
  cpuUsage: number;
  transactionThroughput: number;
}

interface EpochMetrics {
  epoch: number;
  slotIndex: number;
  slotsInEpoch: number;
  absoluteSlot: number;
}

interface SecurityMetrics {
  replayAttackAttempts: number;
  invalidSignatureAttempts: number;
  unauthorizedAccessAttempts: number;
  suspiciousTransactions: SuspiciousTransaction[];
  lastSecurityScan: number;
  vulnerabilityCount: number;
}

interface SuspiciousTransaction {
  signature: string;
  timestamp: number;
  type: 'replay_attack' | 'invalid_signature' | 'unauthorized_access' | 'unusual_pattern';
  details: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

enum AlertLevel {
  GREEN = 'green',
  YELLOW = 'yellow',
  ORANGE = 'orange',
  RED = 'red',
  CRITICAL = 'critical'
}

interface AlertConfig {
  maxErrorRate: number;
  minSuccessRate: number;
  maxResponseTime: number;
  maxReplayAttempts: number;
  maxInvalidSignatures: number;
}

interface MonitoringConfig {
  rpcEndpoint: string;
  programId: string;
  checkInterval: number;
  alertConfig: AlertConfig;
  enabledChains: number[];
  monitoringEnabled: boolean;
}

// Event types for real-time monitoring
interface NFTLifecycleEvent {
  type: 'mint' | 'transfer' | 'cross_chain_transfer' | 'revert' | 'origin_created';
  timestamp: number;
  collection: string;
  tokenId?: number;
  txSignature: string;
  details: any;
}

interface SystemAlert {
  id: string;
  timestamp: number;
  level: AlertLevel;
  category: 'performance' | 'security' | 'functionality' | 'cross_chain';
  message: string;
  details: any;
  resolved: boolean;
}

class UniversalNFTHealthMonitor {
  private connection: Connection;
  private programId: PublicKey;
  private config: MonitoringConfig;
  private metrics: HealthMetrics;
  private alerts: SystemAlert[] = [];
  private eventHistory: NFTLifecycleEvent[] = [];
  private isMonitoring: boolean = false;
  private monitoringInterval?: NodeJS.Timeout;

  constructor(config: MonitoringConfig) {
    this.config = config;
    this.connection = new Connection(config.rpcEndpoint, 'confirmed');
    this.programId = new PublicKey(config.programId);
    this.metrics = this.initializeMetrics();
  }

  private initializeMetrics(): HealthMetrics {
    return {
      timestamp: Date.now(),
      programHealth: {
        isOperational: false,
        programId: this.config.programId,
        lastUpdated: 0,
        errorCount: 0,
        successRate: 0,
        computeUnitsUsed: 0,
        averageTransactionTime: 0
      },
      accountHealth: {
        collections: [],
        nftOrigins: [],
        connectedChains: [],
        totalAccounts: 0,
        healthyAccounts: 0,
        corruptedAccounts: 0
      },
      crossChainMetrics: {
        totalTransfers: 0,
        successfulTransfers: 0,
        failedTransfers: 0,
        pendingTransfers: 0,
        averageTransferTime: 0,
        chainBreakdown: [],
        tssSignatureMetrics: {
          totalSignatures: 0,
          validSignatures: 0,
          invalidSignatures: 0,
          averageVerificationTime: 0,
          lastSignatureTime: 0,
          tssAddress: ''
        },
        gatewayMetrics: {
          isConnected: false,
          gatewayAddress: '',
          totalCalls: 0,
          successfulCalls: 0,
          failedCalls: 0,
          averageResponseTime: 0,
          lastCallTime: 0
        }
      },
      performanceMetrics: {
        rpcLatency: 0,
        blockHeight: 0,
        currentSlot: 0,
        epochInfo: {
          epoch: 0,
          slotIndex: 0,
          slotsInEpoch: 0,
          absoluteSlot: 0
        },
        memoryUsage: 0,
        cpuUsage: 0,
        transactionThroughput: 0
      },
      securityMetrics: {
        replayAttackAttempts: 0,
        invalidSignatureAttempts: 0,
        unauthorizedAccessAttempts: 0,
        suspiciousTransactions: [],
        lastSecurityScan: 0,
        vulnerabilityCount: 0
      },
      alertLevel: AlertLevel.GREEN
    };
  }

  // Start monitoring system
  async startMonitoring(): Promise<void> {
    if (this.isMonitoring) {
      console.log('Monitoring already active');
      return;
    }

    console.log('Starting Universal NFT health monitoring...');
    this.isMonitoring = true;

    // Initial health check
    await this.performHealthCheck();

    // Set up periodic monitoring
    this.monitoringInterval = setInterval(async () => {
      try {
        await this.performHealthCheck();
        await this.checkForAlerts();
        await this.cleanupOldData();
      } catch (error) {
        console.error('Error during monitoring cycle:', error);
        this.createAlert(AlertLevel.RED, 'functionality', 'Monitoring cycle failed', { error });
      }
    }, this.config.checkInterval);

    console.log(`Monitoring started with ${this.config.checkInterval}ms interval`);
  }

  // Stop monitoring system
  stopMonitoring(): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = undefined;
    }
    this.isMonitoring = false;
    console.log('Monitoring stopped');
  }

  // Perform comprehensive health check
  async performHealthCheck(): Promise<HealthMetrics> {
    const startTime = Date.now();

    try {
      // Update timestamp
      this.metrics.timestamp = startTime;

      // Check program health
      await this.checkProgramHealth();

      // Check account health
      await this.checkAccountHealth();

      // Check cross-chain metrics
      await this.checkCrossChainMetrics();

      // Check performance metrics
      await this.checkPerformanceMetrics();

      // Check security metrics
      await this.checkSecurityMetrics();

      // Calculate overall alert level
      this.calculateAlertLevel();

      console.log(`Health check completed in ${Date.now() - startTime}ms`);
      return this.metrics;

    } catch (error) {
      console.error('Health check failed:', error);
      this.metrics.programHealth.isOperational = false;
      this.metrics.alertLevel = AlertLevel.CRITICAL;
      this.createAlert(AlertLevel.CRITICAL, 'functionality', 'Health check failed', { error });
      throw error;
    }
  }

  // Check program operational status
  private async checkProgramHealth(): Promise<void> {
    try {
      const programAccount = await this.connection.getAccountInfo(this.programId);
      
      if (!programAccount) {
        throw new Error('Program account not found');
      }

      // Check if program is executable
      const isExecutable = programAccount.executable;
      
      // Get recent transaction signatures for the program
      const signatures = await this.connection.getSignaturesForAddress(
        this.programId,
        { limit: 100 }
      );

      // Calculate success rate from recent transactions
      let successCount = 0;
      let totalCount = signatures.length;
      let totalComputeUnits = 0;
      let totalTime = 0;

      for (const sig of signatures.slice(0, 50)) { // Check last 50 transactions
        try {
          const tx = await this.connection.getTransaction(sig.signature, {
            maxSupportedTransactionVersion: 0
          });
          
          if (tx) {
            if (!tx.meta?.err) {
              successCount++;
            }
            
            if (tx.meta?.computeUnitsConsumed) {
              totalComputeUnits += tx.meta.computeUnitsConsumed;
            }
            
            if (tx.blockTime && sig.blockTime) {
              totalTime += Math.abs(tx.blockTime - sig.blockTime);
            }
          }
        } catch (error) {
          // Transaction fetch failed, count as error
        }
      }

      this.metrics.programHealth = {
        isOperational: isExecutable && successCount > 0,
        programId: this.programId.toString(),
        lastUpdated: Date.now(),
        errorCount: totalCount - successCount,
        successRate: totalCount > 0 ? (successCount / totalCount) * 100 : 0,
        computeUnitsUsed: totalCount > 0 ? totalComputeUnits / totalCount : 0,
        averageTransactionTime: totalCount > 0 ? totalTime / totalCount : 0
      };

    } catch (error) {
      this.metrics.programHealth.isOperational = false;
      this.metrics.programHealth.lastUpdated = Date.now();
      throw error;
    }
  }

  // Check health of program accounts
  private async checkAccountHealth(): Promise<void> {
    try {
      // Get all collection accounts
      const collections = await this.getAllCollections();
      const nftOrigins = await this.getAllNftOrigins();
      const connectedChains = await this.getAllConnectedChains();

      let healthyAccounts = 0;
      let corruptedAccounts = 0;

      // Check collection health
      const collectionHealthData: CollectionHealthData[] = [];
      for (const collection of collections) {
        const healthData = await this.checkCollectionHealth(collection);
        collectionHealthData.push(healthData);
        
        if (healthData.isHealthy) {
          healthyAccounts++;
        } else {
          corruptedAccounts++;
        }
      }

      // Check NFT Origin health
      const nftOriginHealthData: NftOriginHealthData[] = [];
      for (const origin of nftOrigins) {
        const healthData = await this.checkNftOriginHealth(origin);
        nftOriginHealthData.push(healthData);
        
        if (healthData.isValid) {
          healthyAccounts++;
        } else {
          corruptedAccounts++;
        }
      }

      // Check connected chain health
      const connectedChainData: ConnectedChainData[] = [];
      for (const connected of connectedChains) {
        const chainData = await this.checkConnectedChainHealth(connected);
        connectedChainData.push(chainData);
        
        if (chainData.isActive) {
          healthyAccounts++;
        } else {
          corruptedAccounts++;
        }
      }

      this.metrics.accountHealth = {
        collections: collectionHealthData,
        nftOrigins: nftOriginHealthData,
        connectedChains: connectedChainData,
        totalAccounts: collections.length + nftOrigins.length + connectedChains.length,
        healthyAccounts,
        corruptedAccounts
      };

    } catch (error) {
      console.error('Account health check failed:', error);
      this.createAlert(AlertLevel.ORANGE, 'functionality', 'Account health check failed', { error });
    }
  }

  // Check cross-chain transfer metrics
  private async checkCrossChainMetrics(): Promise<void> {
    try {
      // Get recent cross-chain events from transaction logs
      const signatures = await this.connection.getSignaturesForAddress(
        this.programId,
        { limit: 1000 }
      );

      let totalTransfers = 0;
      let successfulTransfers = 0;
      let failedTransfers = 0;
      let totalTransferTime = 0;
      let tssSignatures = 0;
      let validTssSignatures = 0;
      let gatewayCallsTotal = 0;
      let gatewayCallsSuccess = 0;

      const chainMetrics = new Map<number, ChainTransferMetrics>();

      for (const sig of signatures) {
        try {
          const tx = await this.connection.getTransaction(sig.signature, {
            maxSupportedTransactionVersion: 0
          });
          
          if (tx && tx.meta?.logMessages) {
            // Parse logs for cross-chain events
            const events = this.parseTransactionLogs(tx.meta.logMessages);
            
            for (const event of events) {
              if (event.type === 'cross_chain_transfer') {
                totalTransfers++;
                
                if (!tx.meta.err) {
                  successfulTransfers++;
                } else {
                  failedTransfers++;
                }

                // Update chain metrics
                const chainId = event.details.destinationChain;
                if (!chainMetrics.has(chainId)) {
                  chainMetrics.set(chainId, {
                    chainId,
                    chainName: this.getChainName(chainId),
                    inboundTransfers: 0,
                    outboundTransfers: 1,
                    successRate: 0,
                    averageTime: 0,
                    lastTransfer: sig.blockTime || 0
                  });
                } else {
                  const metrics = chainMetrics.get(chainId)!;
                  metrics.outboundTransfers++;
                  metrics.lastTransfer = Math.max(metrics.lastTransfer, sig.blockTime || 0);
                }
              }

              if (event.type === 'tss_signature') {
                tssSignatures++;
                if (event.details.valid) {
                  validTssSignatures++;
                }
              }

              if (event.type === 'gateway_call') {
                gatewayCallsTotal++;
                if (event.details.success) {
                  gatewayCallsSuccess++;
                }
              }
            }
          }
        } catch (error) {
          // Skip failed transaction parsing
        }
      }

      // Calculate metrics
      const chainBreakdown = Array.from(chainMetrics.values());
      chainBreakdown.forEach(chain => {
        const totalChainTransfers = chain.inboundTransfers + chain.outboundTransfers;
        chain.successRate = totalChainTransfers > 0 ? 
          (chain.outboundTransfers / totalChainTransfers) * 100 : 0;
      });

      this.metrics.crossChainMetrics = {
        totalTransfers,
        successfulTransfers,
        failedTransfers,
        pendingTransfers: 0, // Would need to track pending state
        averageTransferTime: totalTransfers > 0 ? totalTransferTime / totalTransfers : 0,
        chainBreakdown,
        tssSignatureMetrics: {
          totalSignatures: tssSignatures,
          validSignatures: validTssSignatures,
          invalidSignatures: tssSignatures - validTssSignatures,
          averageVerificationTime: 0, // Would need timing data
          lastSignatureTime: Date.now(),
          tssAddress: '' // Would get from collection data
        },
        gatewayMetrics: {
          isConnected: gatewayCallsTotal > 0,
          gatewayAddress: '', // Would get from program data
          totalCalls: gatewayCallsTotal,
          successfulCalls: gatewayCallsSuccess,
          failedCalls: gatewayCallsTotal - gatewayCallsSuccess,
          averageResponseTime: 0, // Would need timing data
          lastCallTime: Date.now()
        }
      };

    } catch (error) {
      console.error('Cross-chain metrics check failed:', error);
      this.createAlert(AlertLevel.YELLOW, 'cross_chain', 'Cross-chain metrics check failed', { error });
    }
  }

  // Check performance metrics
  private async checkPerformanceMetrics(): Promise<void> {
    try {
      const startTime = Date.now();
      
      // Check RPC latency
      const healthResponse = await this.connection.getHealth();
      const rpcLatency = Date.now() - startTime;

      // Get current blockchain state
      const epochInfo = await this.connection.getEpochInfo();
      const blockHeight = await this.connection.getBlockHeight();
      const slot = await this.connection.getSlot();

      // Get recent performance samples
      const perfSamples = await this.connection.getRecentPerformanceSamples(10);
      
      let avgThroughput = 0;
      if (perfSamples.length > 0) {
        const totalTransactions = perfSamples.reduce((sum, sample) => sum + sample.numTransactions, 0);
        const totalSlots = perfSamples.reduce((sum, sample) => sum + sample.numSlots, 0);
        avgThroughput = totalSlots > 0 ? totalTransactions / totalSlots : 0;
      }

      this.metrics.performanceMetrics = {
        rpcLatency,
        blockHeight,
        currentSlot: slot,
        epochInfo: {
          epoch: epochInfo.epoch,
          slotIndex: epochInfo.slotIndex,
          slotsInEpoch: epochInfo.slotsInEpoch,
          absoluteSlot: epochInfo.absoluteSlot
        },
        memoryUsage: process.memoryUsage().heapUsed / 1024 / 1024, // MB
        cpuUsage: process.cpuUsage().user / 1000000, // Convert to seconds
        transactionThroughput: avgThroughput
      };

    } catch (error) {
      console.error('Performance metrics check failed:', error);
      this.createAlert(AlertLevel.YELLOW, 'performance', 'Performance metrics check failed', { error });
    }
  }

  // Check security metrics
  private async checkSecurityMetrics(): Promise<void> {
    try {
      // Analyze recent transactions for security issues
      const signatures = await this.connection.getSignaturesForAddress(
        this.programId,
        { limit: 500 }
      );

      let replayAttempts = 0;
      let invalidSignatures = 0;
      let unauthorizedAccess = 0;
      const suspiciousTransactions: SuspiciousTransaction[] = [];

      for (const sig of signatures.slice(0, 100)) { // Check recent 100 transactions
        try {
          const tx = await this.connection.getTransaction(sig.signature, {
            maxSupportedTransactionVersion: 0
          });
          
          if (tx && tx.meta?.logMessages) {
            // Check for security-related errors in logs
            const logs = tx.meta.logMessages.join(' ');
            
            if (logs.includes('InvalidNonce') || logs.includes('replay')) {
              replayAttempts++;
              suspiciousTransactions.push({
                signature: sig.signature,
                timestamp: sig.blockTime || 0,
                type: 'replay_attack',
                details: 'Potential replay attack detected',
                severity: 'high'
              });
            }
            
            if (logs.includes('InvalidSignature') || logs.includes('InvalidTssSignature')) {
              invalidSignatures++;
              suspiciousTransactions.push({
                signature: sig.signature,
                timestamp: sig.blockTime || 0,
                type: 'invalid_signature',
                details: 'Invalid signature detected',
                severity: 'medium'
              });
            }
            
            if (logs.includes('Unauthorized') || logs.includes('NotTokenOwner')) {
              unauthorizedAccess++;
              suspiciousTransactions.push({
                signature: sig.signature,
                timestamp: sig.blockTime || 0,
                type: 'unauthorized_access',
                details: 'Unauthorized access attempt',
                severity: 'high'
              });
            }
          }
        } catch (error) {
          // Skip failed transaction analysis
        }
      }

      this.metrics.securityMetrics = {
        replayAttackAttempts: replayAttempts,
        invalidSignatureAttempts: invalidSignatures,
        unauthorizedAccessAttempts: unauthorizedAccess,
        suspiciousTransactions: suspiciousTransactions.slice(0, 50), // Keep last 50
        lastSecurityScan: Date.now(),
        vulnerabilityCount: replayAttempts + invalidSignatures + unauthorizedAccess
      };

    } catch (error) {
      console.error('Security metrics check failed:', error);
      this.createAlert(AlertLevel.ORANGE, 'security', 'Security metrics check failed', { error });
    }
  }

  // Calculate overall alert level
  private calculateAlertLevel(): void {
    let alertLevel = AlertLevel.GREEN;

    // Check program health
    if (!this.metrics.programHealth.isOperational) {
      alertLevel = AlertLevel.CRITICAL;
    } else if (this.metrics.programHealth.successRate < this.config.alertConfig.minSuccessRate) {
      alertLevel = AlertLevel.RED;
    }

    // Check performance
    if (this.metrics.performanceMetrics.rpcLatency > this.config.alertConfig.maxResponseTime) {
      alertLevel = this.upgradeAlertLevel(alertLevel, AlertLevel.YELLOW);
    }

    // Check security
    if (this.metrics.securityMetrics.replayAttackAttempts > this.config.alertConfig.maxReplayAttempts) {
      alertLevel = this.upgradeAlertLevel(alertLevel, AlertLevel.ORANGE);
    }

    if (this.metrics.securityMetrics.invalidSignatureAttempts > this.config.alertConfig.maxInvalidSignatures) {
      alertLevel = this.upgradeAlertLevel(alertLevel, AlertLevel.RED);
    }

    // Check cross-chain health
    const crossChainSuccessRate = this.metrics.crossChainMetrics.totalTransfers > 0 ?
      (this.metrics.crossChainMetrics.successfulTransfers / this.metrics.crossChainMetrics.totalTransfers) * 100 : 100;
    
    if (crossChainSuccessRate < this.config.alertConfig.minSuccessRate) {
      alertLevel = this.upgradeAlertLevel(alertLevel, AlertLevel.ORANGE);
    }

    this.metrics.alertLevel = alertLevel;
  }

  private upgradeAlertLevel(current: AlertLevel, proposed: AlertLevel): AlertLevel {
    const levels = [AlertLevel.GREEN, AlertLevel.YELLOW, AlertLevel.ORANGE, AlertLevel.RED, AlertLevel.CRITICAL];
    const currentIndex = levels.indexOf(current);
    const proposedIndex = levels.indexOf(proposed);
    return proposedIndex > currentIndex ? proposed : current;
  }

  // Alert management
  private async checkForAlerts(): Promise<void> {
    const config = this.config.alertConfig;

    // Check for new alerts based on current metrics
    if (this.metrics.programHealth.successRate < config.minSuccessRate) {
      this.createAlert(
        AlertLevel.RED,
        'performance',
        `Program success rate below threshold: ${this.metrics.programHealth.successRate.toFixed(2)}%`,
        { threshold: config.minSuccessRate, actual: this.metrics.programHealth.successRate }
      );
    }

    if (this.metrics.performanceMetrics.rpcLatency > config.maxResponseTime) {
      this.createAlert(
        AlertLevel.YELLOW,
        'performance',
        `High RPC latency: ${this.metrics.performanceMetrics.rpcLatency}ms`,
        { threshold: config.maxResponseTime, actual: this.metrics.performanceMetrics.rpcLatency }
      );
    }

    if (this.metrics.securityMetrics.replayAttackAttempts > config.maxReplayAttempts) {
      this.createAlert(
        AlertLevel.ORANGE,
        'security',
        `High number of replay attack attempts: ${this.metrics.securityMetrics.replayAttackAttempts}`,
        { threshold: config.maxReplayAttempts, actual: this.metrics.securityMetrics.replayAttackAttempts }
      );
    }

    // Auto-resolve old alerts
    this.autoResolveAlerts();
  }

  private createAlert(level: AlertLevel, category: string, message: string, details: any): void {
    const alert: SystemAlert = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      timestamp: Date.now(),
      level,
      category: category as any,
      message,
      details,
      resolved: false
    };

    this.alerts.unshift(alert);
    
    // Keep only last 1000 alerts
    if (this.alerts.length > 1000) {
      this.alerts = this.alerts.slice(0, 1000);
    }

    console.log(`🚨 ALERT [${level}] ${category}: ${message}`);
    
    // Trigger external alerting if configured
    this.triggerExternalAlert(alert);
  }

  private autoResolveAlerts(): void {
    const now = Date.now();
    const autoResolveTime = 24 * 60 * 60 * 1000; // 24 hours

    this.alerts.forEach(alert => {
      if (!alert.resolved && (now - alert.timestamp) > autoResolveTime) {
        alert.resolved = true;
      }
    });
  }

  private triggerExternalAlert(alert: SystemAlert): void {
    // Implement external alerting (email, Slack, Discord, etc.)
    // This is a placeholder for integration with external services
    if (alert.level === AlertLevel.CRITICAL || alert.level === AlertLevel.RED) {
      console.log(`🔥 CRITICAL ALERT: ${alert.message}`);
      // Send to external monitoring service
    }
  }

  // Utility methods for account health checks
  private getDiscriminator(accountName: string): string {
    const hash = createHash('sha256');
    hash.update(`account:${accountName}`);
    const discriminator = hash.digest().slice(0, 8);
    return bs58.encode(discriminator);
  }

  private async getAllCollections(): Promise<any[]> {
    // Implementation would use getProgramAccounts to fetch all collection accounts
    try {
      const accounts = await this.connection.getProgramAccounts(this.programId, {
        filters: [
          {
            memcmp: {
              offset: 0,
              bytes: this.getDiscriminator('Collection')
            }
          }
        ]
      });
      return [...accounts];
    } catch (error) {
      console.error('Failed to fetch collections:', error);
      return [];
    }
  }

  private async getAllNftOrigins(): Promise<any[]> {
    // Implementation would fetch all NFT Origin accounts
    try {
      const accounts = await this.connection.getProgramAccounts(this.programId, {
        filters: [
          {
            memcmp: {
              offset: 0,
              bytes: this.getDiscriminator('NftOrigin')
            }
          }
        ]
      });
      return [...accounts];
    } catch (error) {
      console.error('Failed to fetch NFT origins:', error);
      return [];
    }
  }

  private async getAllConnectedChains(): Promise<any[]> {
    // Implementation would fetch all connected chain accounts
    try {
      const accounts = await this.connection.getProgramAccounts(this.programId, {
        filters: [
          {
            memcmp: {
              offset: 0,
              bytes: this.getDiscriminator('Connected')
            }
          }
        ]
      });
      return [...accounts];
    } catch (error) {
      console.error('Failed to fetch connected chains:', error);
      return [];
    }
  }

  private async checkCollectionHealth(collection: any): Promise<CollectionHealthData> {
    // Parse collection account data and check health
    try {
      const data = collection.account.data;
      // This would parse the actual collection data structure
      
      return {
        address: collection.pubkey.toString(),
        name: 'Collection Name', // Parse from data
        authority: 'Authority Address', // Parse from data
        totalMinted: 0, // Parse from data
        solanaNativeCount: 0, // Parse from data
        crossChainCount: 0, // Calculate
        nonce: 0, // Parse from data
        lastActivity: Date.now(),
        isHealthy: true,
        issues: []
      };
    } catch (error) {
      return {
        address: collection.pubkey.toString(),
        name: 'Unknown',
        authority: 'Unknown',
        totalMinted: 0,
        solanaNativeCount: 0,
        crossChainCount: 0,
        nonce: 0,
        lastActivity: 0,
        isHealthy: false,
        issues: ['Failed to parse collection data']
      };
    }
  }

  private async checkNftOriginHealth(origin: any): Promise<NftOriginHealthData> {
    // Parse NFT Origin account data and check health
    try {
      const data = origin.account.data;
      // This would parse the actual NFT Origin data structure
      
      return {
        tokenId: 0, // Parse from data
        originalMint: 'Mint Address', // Parse from data
        collection: 'Collection Address', // Parse from data
        chainOfOrigin: 103, // Parse from data
        createdAt: Date.now(), // Parse from data
        isValid: true,
        metadataAccessible: true,
        issues: []
      };
    } catch (error) {
      return {
        tokenId: 0,
        originalMint: 'Unknown',
        collection: 'Unknown',
        chainOfOrigin: 0,
        createdAt: 0,
        isValid: false,
        metadataAccessible: false,
        issues: ['Failed to parse NFT Origin data']
      };
    }
  }

  private async checkConnectedChainHealth(connected: any): Promise<ConnectedChainData> {
    // Parse connected chain account data and check health
    try {
      const data = connected.account.data;
      // This would parse the actual connected chain data structure
      
      return {
        chainId: 1, // Parse from data
        chainName: 'Ethereum', // Derive from chain ID
        contractAddress: 'Contract Address', // Parse from data
        isActive: true,
        lastSeen: Date.now(),
        transferCount: 0,
        errorRate: 0
      };
    } catch (error) {
      return {
        chainId: 0,
        chainName: 'Unknown',
        contractAddress: 'Unknown',
        isActive: false,
        lastSeen: 0,
        transferCount: 0,
        errorRate: 100
      };
    }
  }

  private parseTransactionLogs(logs: string[]): NFTLifecycleEvent[] {
    const events: NFTLifecycleEvent[] = [];
    
    // Parse logs for specific event patterns
    logs.forEach(log => {
      if (log.includes('TokenMinted')) {
        events.push({
          type: 'mint',
          timestamp: Date.now(),
          collection: 'parsed_collection',
          tokenId: 0, // Parse from log
          txSignature: 'signature',
          details: {}
        });
      }
      
      if (log.includes('TokenTransfer')) {
        events.push({
          type: 'cross_chain_transfer',
          timestamp: Date.now(),
          collection: 'parsed_collection',
          tokenId: 0, // Parse from log
          txSignature: 'signature',
          details: { destinationChain: 1 }
        });
      }
      
      // Add more event parsing as needed
    });
    
    return events;
  }

  private getChainName(chainId: number): string {
    const chainNames: { [key: number]: string } = {
      1: 'Ethereum',
      56: 'BSC',
      137: 'Polygon',
      8453: 'Base',
      42161: 'Arbitrum',
      10: 'Optimism',
      7000: 'ZetaChain',
      11155111: 'Ethereum Sepolia',
      97: 'BSC Testnet',
      80001: 'Polygon Mumbai',
      84532: 'Base Sepolia',
      421614: 'Arbitrum Sepolia',
      11155420: 'Optimism Sepolia',
      7001: 'ZetaChain Testnet',
      101: 'Solana Mainnet',
      102: 'Solana Testnet',
      103: 'Solana Devnet'
    };
    
    return chainNames[chainId] || `Chain ${chainId}`;
  }

  private async cleanupOldData(): Promise<void> {
    const maxAge = 7 * 24 * 60 * 60 * 1000; // 7 days
    const cutoff = Date.now() - maxAge;
    
    // Clean up old events
    this.eventHistory = this.eventHistory.filter(event => event.timestamp > cutoff);
    
    // Clean up old alerts (keep resolved alerts for 24 hours)
    this.alerts = this.alerts.filter(alert => 
      !alert.resolved || (alert.timestamp > (Date.now() - 24 * 60 * 60 * 1000))
    );
  }

  // Public API methods
  getHealthMetrics(): HealthMetrics {
    return { ...this.metrics };
  }

  getAlerts(includeResolved: boolean = false): SystemAlert[] {
    return this.alerts.filter(alert => includeResolved || !alert.resolved);
  }

  getEventHistory(limit: number = 100): NFTLifecycleEvent[] {
    return this.eventHistory.slice(0, limit);
  }

  async generateHealthReport(): Promise<string> {
    const metrics = this.getHealthMetrics();
    const alerts = this.getAlerts();
    
    return `
# Universal NFT Health Report
Generated: ${new Date(metrics.timestamp).toISOString()}
Alert Level: ${metrics.alertLevel}

## Program Health
- Operational: ${metrics.programHealth.isOperational ? '✅' : '❌'}
- Success Rate: ${metrics.programHealth.successRate.toFixed(2)}%
- Average Compute Units: ${metrics.programHealth.computeUnitsUsed}
- Average Transaction Time: ${metrics.programHealth.averageTransactionTime}ms

## Account Health
- Total Accounts: ${metrics.accountHealth.totalAccounts}
- Healthy: ${metrics.accountHealth.healthyAccounts}
- Corrupted: ${metrics.accountHealth.corruptedAccounts}
- Collections: ${metrics.accountHealth.collections.length}
- NFT Origins: ${metrics.accountHealth.nftOrigins.length}

## Cross-Chain Metrics
- Total Transfers: ${metrics.crossChainMetrics.totalTransfers}
- Success Rate: ${((metrics.crossChainMetrics.successfulTransfers / Math.max(metrics.crossChainMetrics.totalTransfers, 1)) * 100).toFixed(2)}%
- TSS Signatures: ${metrics.crossChainMetrics.tssSignatureMetrics.totalSignatures}
- Gateway Connected: ${metrics.crossChainMetrics.gatewayMetrics.isConnected ? '✅' : '❌'}

## Performance
- RPC Latency: ${metrics.performanceMetrics.rpcLatency}ms
- Current Slot: ${metrics.performanceMetrics.currentSlot}
- Block Height: ${metrics.performanceMetrics.blockHeight}
- Transaction Throughput: ${metrics.performanceMetrics.transactionThroughput.toFixed(2)} TPS

## Security
- Replay Attempts: ${metrics.securityMetrics.replayAttackAttempts}
- Invalid Signatures: ${metrics.securityMetrics.invalidSignatureAttempts}
- Unauthorized Access: ${metrics.securityMetrics.unauthorizedAccessAttempts}
- Vulnerability Count: ${metrics.securityMetrics.vulnerabilityCount}

## Active Alerts
${alerts.length === 0 ? 'No active alerts' : alerts.map(alert => 
  `- [${alert.level.toUpperCase()}] ${alert.category}: ${alert.message}`
).join('\n')}
    `.trim();
  }

  // Dashboard data for frontend integration
  getDashboardData() {
    return {
      overview: {
        status: this.metrics.alertLevel,
        isOperational: this.metrics.programHealth.isOperational,
        totalCollections: this.metrics.accountHealth.collections.length,
        totalTransfers: this.metrics.crossChainMetrics.totalTransfers,
        activeAlerts: this.getAlerts().length
      },
      charts: {
        successRate: this.metrics.programHealth.successRate,
        crossChainBreakdown: this.metrics.crossChainMetrics.chainBreakdown,
        performanceMetrics: {
          latency: this.metrics.performanceMetrics.rpcLatency,
          throughput: this.metrics.performanceMetrics.transactionThroughput,
          blockHeight: this.metrics.performanceMetrics.blockHeight
        },
        securityMetrics: {
          replayAttempts: this.metrics.securityMetrics.replayAttackAttempts,
          invalidSignatures: this.metrics.securityMetrics.invalidSignatureAttempts,
          vulnerabilities: this.metrics.securityMetrics.vulnerabilityCount
        }
      },
      recentEvents: this.getEventHistory(20),
      recentAlerts: this.getAlerts().slice(0, 10)
    };
  }
}

// Export the monitoring system and types
export {
  UniversalNFTHealthMonitor,
  HealthMetrics,
  AlertLevel,
  MonitoringConfig,
  SystemAlert,
  NFTLifecycleEvent
};

// Example usage and configuration
export const createDefaultConfig = (rpcEndpoint: string, programId: string): MonitoringConfig => ({
  rpcEndpoint,
  programId,
  checkInterval: 30000, // 30 seconds
  alertConfig: {
    maxErrorRate: 10, // 10%
    minSuccessRate: 90, // 90%
    maxResponseTime: 5000, // 5 seconds
    maxReplayAttempts: 5,
    maxInvalidSignatures: 10
  },
  enabledChains: [1, 56, 137, 8453, 42161, 10, 7000], // Mainnet chains
  monitoringEnabled: true
});

// Example initialization
export const initializeMonitoring = async (config: MonitoringConfig) => {
  const monitor = new UniversalNFTHealthMonitor(config);
  await monitor.startMonitoring();
  
  // Set up graceful shutdown
  process.on('SIGINT', () => {
    console.log('Shutting down monitoring...');
    monitor.stopMonitoring();
    process.exit(0);
  });
  
  return monitor;
};