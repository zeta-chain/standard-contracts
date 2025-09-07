# Universal NFT Integration Examples

This document provides comprehensive integration examples for the Solana Universal NFT protocol, demonstrating how to build applications that leverage cross-chain NFT functionality.

## Table of Contents

1. [Frontend Integration](#frontend-integration)
2. [Backend Integration](#backend-integration)
3. [Cross-Chain Scenarios](#cross-chain-scenarios)
4. [Testing Examples](#testing-examples)
5. [Production Examples](#production-examples)

## Frontend Integration

### React/Next.js Integration with UniversalNftClient SDK

#### Basic Setup and Wallet Connection

```tsx
// components/WalletProvider.tsx
import React, { createContext, useContext, useState, useEffect } from 'react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { ConnectionProvider, WalletProvider as SolanaWalletProvider } from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter, SolflareWalletAdapter } from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import { UniversalNftClient, Network } from '../sdk/client';

interface UniversalNftContextType {
  client: UniversalNftClient | null;
  isConnected: boolean;
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
}

const UniversalNftContext = createContext<UniversalNftContextType>({
  client: null,
  isConnected: false,
  error: null,
  connect: async () => {},
  disconnect: () => {},
});

export const useUniversalNft = () => useContext(UniversalNftContext);

export const UniversalNftProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [client, setClient] = useState<UniversalNftClient | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const network = WalletAdapterNetwork.Devnet;
  const endpoint = clusterApiUrl(network);
  
  const wallets = [
    new PhantomWalletAdapter(),
    new SolflareWalletAdapter(),
  ];

  const connect = async () => {
    try {
      setError(null);
      
      // Get wallet adapter from context
      const wallet = window.solana;
      if (!wallet) {
        throw new Error('Wallet not found');
      }

      await wallet.connect();
      
      const universalClient = await UniversalNftClient.create(
        {
          network: Network.DEVNET,
          endpoint,
          commitment: 'confirmed',
        },
        {
          adapter: wallet,
        }
      );

      setClient(universalClient);
      setIsConnected(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect');
      console.error('Connection error:', err);
    }
  };

  const disconnect = () => {
    if (client) {
      client.dispose();
    }
    setClient(null);
    setIsConnected(false);
    setError(null);
  };

  return (
    <ConnectionProvider endpoint={endpoint}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <UniversalNftContext.Provider value={{
            client,
            isConnected,
            error,
            connect,
            disconnect,
          }}>
            {children}
          </UniversalNftContext.Provider>
        </WalletModalProvider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
};
```

#### NFT Minting Component

```tsx
// components/MintNFT.tsx
import React, { useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useUniversalNft } from './WalletProvider';
import { MintResult } from '../sdk/client';

interface MintNFTProps {
  collectionAddress: string;
}

export const MintNFT: React.FC<MintNFTProps> = ({ collectionAddress }) => {
  const { client, isConnected } = useUniversalNft();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<MintResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  const [formData, setFormData] = useState({
    name: '',
    symbol: '',
    uri: '',
    recipient: '',
  });

  const handleMint = async () => {
    if (!client || !isConnected) {
      setError('Wallet not connected');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const collection = new PublicKey(collectionAddress);
      const recipient = formData.recipient 
        ? new PublicKey(formData.recipient) 
        : undefined;

      const mintResult = await client.mintNft(
        collection,
        formData.name,
        formData.symbol,
        formData.uri,
        recipient
      );

      setResult(mintResult);
      
      // Reset form
      setFormData({ name: '', symbol: '', uri: '', recipient: '' });
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Minting failed');
      console.error('Mint error:', err);
    } finally {
      setLoading(false);
    }
  };

  if (!isConnected) {
    return (
      <div className="p-4 border rounded-lg">
        <p className="text-gray-600">Please connect your wallet to mint NFTs</p>
      </div>
    );
  }

  return (
    <div className="p-6 border rounded-lg space-y-4">
      <h3 className="text-xl font-bold">Mint Universal NFT</h3>
      
      <div className="space-y-3">
        <input
          type="text"
          placeholder="NFT Name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          className="w-full p-2 border rounded"
        />
        
        <input
          type="text"
          placeholder="Symbol"
          value={formData.symbol}
          onChange={(e) => setFormData({ ...formData, symbol: e.target.value })}
          className="w-full p-2 border rounded"
        />
        
        <input
          type="url"
          placeholder="Metadata URI"
          value={formData.uri}
          onChange={(e) => setFormData({ ...formData, uri: e.target.value })}
          className="w-full p-2 border rounded"
        />
        
        <input
          type="text"
          placeholder="Recipient (optional)"
          value={formData.recipient}
          onChange={(e) => setFormData({ ...formData, recipient: e.target.value })}
          className="w-full p-2 border rounded"
        />
      </div>

      <button
        onClick={handleMint}
        disabled={loading || !formData.name || !formData.symbol || !formData.uri}
        className="w-full bg-blue-500 text-white p-2 rounded disabled:bg-gray-300"
      >
        {loading ? 'Minting...' : 'Mint NFT'}
      </button>

      {error && (
        <div className="p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      {result && (
        <div className="p-3 bg-green-100 border border-green-400 text-green-700 rounded">
          <h4 className="font-bold">NFT Minted Successfully!</h4>
          <p><strong>Mint:</strong> {result.mint.toString()}</p>
          <p><strong>Token Account:</strong> {result.tokenAccount.toString()}</p>
          <p><strong>Transaction:</strong> {result.signature}</p>
        </div>
      )}
    </div>
  );
};
```

#### Cross-Chain Transfer Component

```tsx
// components/CrossChainTransfer.tsx
import React, { useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useUniversalNft } from './WalletProvider';
import { UniversalNftUtils } from '../sdk/client';

interface CrossChainTransferProps {
  collectionAddress: string;
}

const SUPPORTED_CHAINS = {
  1: 'Ethereum Mainnet',
  11155111: 'Ethereum Sepolia',
  56: 'BNB Smart Chain',
  97: 'BNB Testnet',
  8453: 'Base',
  84532: 'Base Sepolia',
};

export const CrossChainTransfer: React.FC<CrossChainTransferProps> = ({ collectionAddress }) => {
  const { client, isConnected } = useUniversalNft();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  
  const [formData, setFormData] = useState({
    nftMint: '',
    destinationChain: '',
    recipientAddress: '',
  });

  const handleTransfer = async () => {
    if (!client || !isConnected) {
      setError('Wallet not connected');
      return;
    }

    try {
      setLoading(true);
      setError(null);
      setSuccess(null);

      // Validate inputs
      if (!UniversalNftUtils.isValidPublicKey(formData.nftMint)) {
        throw new Error('Invalid NFT mint address');
      }

      if (!UniversalNftUtils.isValidEthAddress(formData.recipientAddress)) {
        throw new Error('Invalid recipient address');
      }

      const collection = new PublicKey(collectionAddress);
      const nftMint = new PublicKey(formData.nftMint);
      const destinationChainId = parseInt(formData.destinationChain);
      const recipientBytes = UniversalNftUtils.ethAddressToBytes(formData.recipientAddress);

      const result = await client.transferCrossChain(
        collection,
        nftMint,
        destinationChainId,
        recipientBytes
      );

      setSuccess(`Transfer initiated! Transaction: ${result.signature}`);
      setFormData({ nftMint: '', destinationChain: '', recipientAddress: '' });
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Transfer failed');
      console.error('Transfer error:', err);
    } finally {
      setLoading(false);
    }
  };

  if (!isConnected) {
    return (
      <div className="p-4 border rounded-lg">
        <p className="text-gray-600">Please connect your wallet to transfer NFTs</p>
      </div>
    );
  }

  return (
    <div className="p-6 border rounded-lg space-y-4">
      <h3 className="text-xl font-bold">Cross-Chain Transfer</h3>
      
      <div className="space-y-3">
        <input
          type="text"
          placeholder="NFT Mint Address"
          value={formData.nftMint}
          onChange={(e) => setFormData({ ...formData, nftMint: e.target.value })}
          className="w-full p-2 border rounded"
        />
        
        <select
          value={formData.destinationChain}
          onChange={(e) => setFormData({ ...formData, destinationChain: e.target.value })}
          className="w-full p-2 border rounded"
        >
          <option value="">Select Destination Chain</option>
          {Object.entries(SUPPORTED_CHAINS).map(([chainId, name]) => (
            <option key={chainId} value={chainId}>{name}</option>
          ))}
        </select>
        
        <input
          type="text"
          placeholder="Recipient Address (0x...)"
          value={formData.recipientAddress}
          onChange={(e) => setFormData({ ...formData, recipientAddress: e.target.value })}
          className="w-full p-2 border rounded"
        />
      </div>

      <button
        onClick={handleTransfer}
        disabled={loading || !formData.nftMint || !formData.destinationChain || !formData.recipientAddress}
        className="w-full bg-purple-500 text-white p-2 rounded disabled:bg-gray-300"
      >
        {loading ? 'Transferring...' : 'Transfer Cross-Chain'}
      </button>

      {error && (
        <div className="p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      {success && (
        <div className="p-3 bg-green-100 border border-green-400 text-green-700 rounded">
          {success}
        </div>
      )}
    </div>
  );
};
```

#### Real-Time Event Monitoring

```tsx
// components/EventMonitor.tsx
import React, { useState, useEffect } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useUniversalNft } from './WalletProvider';
import { NftMintedEvent, CrossChainTransferEvent, CrossChainReceiveEvent } from '../sdk/client';

interface EventMonitorProps {
  collectionAddress?: string;
}

interface EventLog {
  id: string;
  type: 'mint' | 'transfer' | 'receive';
  timestamp: Date;
  data: any;
}

export const EventMonitor: React.FC<EventMonitorProps> = ({ collectionAddress }) => {
  const { client, isConnected } = useUniversalNft();
  const [events, setEvents] = useState<EventLog[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);

  useEffect(() => {
    if (!client || !isConnected) return;

    const subscriptions: any[] = [];
    
    const startMonitoring = async () => {
      try {
        const collection = collectionAddress ? new PublicKey(collectionAddress) : undefined;

        // Subscribe to NFT minted events
        const mintSub = await client.onNftMinted((event: NftMintedEvent) => {
          const eventLog: EventLog = {
            id: `mint-${Date.now()}-${Math.random()}`,
            type: 'mint',
            timestamp: new Date(),
            data: {
              collection: event.collection.toString(),
              mint: event.mint.toString(),
              recipient: event.recipient.toString(),
              tokenId: event.tokenId.toString(),
              name: event.name,
              symbol: event.symbol,
            },
          };
          setEvents(prev => [eventLog, ...prev].slice(0, 50)); // Keep last 50 events
        }, collection);

        // Subscribe to cross-chain transfer events
        const transferSub = await client.onCrossChainTransfer((event: CrossChainTransferEvent) => {
          const eventLog: EventLog = {
            id: `transfer-${Date.now()}-${Math.random()}`,
            type: 'transfer',
            timestamp: new Date(),
            data: {
              collection: event.collection.toString(),
              mint: event.mint.toString(),
              sender: event.sender.toString(),
              destinationChain: event.destinationChain,
              recipient: event.recipient,
              tokenId: event.tokenId.toString(),
            },
          };
          setEvents(prev => [eventLog, ...prev].slice(0, 50));
        }, collection);

        // Subscribe to cross-chain receive events
        const receiveSub = await client.onCrossChainReceive((event: CrossChainReceiveEvent) => {
          const eventLog: EventLog = {
            id: `receive-${Date.now()}-${Math.random()}`,
            type: 'receive',
            timestamp: new Date(),
            data: {
              collection: event.collection.toString(),
              mint: event.mint.toString(),
              recipient: event.recipient.toString(),
              sourceChain: event.sourceChain,
              sender: event.sender,
              tokenId: event.tokenId.toString(),
              isReturning: event.isReturning,
            },
          };
          setEvents(prev => [eventLog, ...prev].slice(0, 50));
        }, collection);

        subscriptions.push(mintSub, transferSub, receiveSub);
        setIsMonitoring(true);

      } catch (error) {
        console.error('Failed to start monitoring:', error);
      }
    };

    startMonitoring();

    return () => {
      subscriptions.forEach(sub => sub.unsubscribe());
      setIsMonitoring(false);
    };
  }, [client, isConnected, collectionAddress]);

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'mint': return '🎨';
      case 'transfer': return '🌉';
      case 'receive': return '📥';
      default: return '📝';
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case 'mint': return 'bg-green-100 border-green-400 text-green-700';
      case 'transfer': return 'bg-blue-100 border-blue-400 text-blue-700';
      case 'receive': return 'bg-purple-100 border-purple-400 text-purple-700';
      default: return 'bg-gray-100 border-gray-400 text-gray-700';
    }
  };

  return (
    <div className="p-6 border rounded-lg">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-xl font-bold">Event Monitor</h3>
        <div className="flex items-center space-x-2">
          <div className={`w-3 h-3 rounded-full ${isMonitoring ? 'bg-green-500' : 'bg-red-500'}`}></div>
          <span className="text-sm">{isMonitoring ? 'Monitoring' : 'Disconnected'}</span>
        </div>
      </div>

      {!isConnected && (
        <p className="text-gray-600">Connect wallet to monitor events</p>
      )}

      {isConnected && events.length === 0 && (
        <p className="text-gray-600">No events yet. Perform some operations to see them here.</p>
      )}

      <div className="space-y-2 max-h-96 overflow-y-auto">
        {events.map((event) => (
          <div key={event.id} className={`p-3 border rounded ${getEventColor(event.type)}`}>
            <div className="flex justify-between items-start">
              <div className="flex items-center space-x-2">
                <span className="text-lg">{getEventIcon(event.type)}</span>
                <span className="font-semibold capitalize">{event.type}</span>
              </div>
              <span className="text-xs">{event.timestamp.toLocaleTimeString()}</span>
            </div>
            <div className="mt-2 text-sm">
              <pre className="whitespace-pre-wrap">{JSON.stringify(event.data, null, 2)}</pre>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
```

## Backend Integration

### Node.js Server with Express

```typescript
// server/app.ts
import express from 'express';
import cors from 'cors';
import { Connection, Keypair } from '@solana/web3.js';
import { UniversalNftClient, Network } from '../sdk/client';
import { NFTDatabase } from './database';
import { WebhookHandler } from './webhooks';
import { EventProcessor } from './events';

const app = express();
const port = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());

// Initialize services
let nftClient: UniversalNftClient;
let database: NFTDatabase;
let webhookHandler: WebhookHandler;
let eventProcessor: EventProcessor;

async function initializeServices() {
  try {
    // Load server keypair from environment
    const privateKey = process.env.SERVER_PRIVATE_KEY;
    if (!privateKey) {
      throw new Error('SERVER_PRIVATE_KEY environment variable required');
    }

    const serverKeypair = Keypair.fromSecretKey(
      new Uint8Array(JSON.parse(privateKey))
    );

    // Initialize Universal NFT client
    nftClient = await UniversalNftClient.create(
      {
        network: process.env.NODE_ENV === 'production' ? Network.MAINNET : Network.DEVNET,
        commitment: 'confirmed',
      },
      {
        keypair: serverKeypair,
      }
    );

    // Initialize database
    database = new NFTDatabase(process.env.DATABASE_URL || 'sqlite:./nfts.db');
    await database.initialize();

    // Initialize webhook handler
    webhookHandler = new WebhookHandler(database);

    // Initialize event processor
    eventProcessor = new EventProcessor(nftClient, database, webhookHandler);
    await eventProcessor.start();

    console.log('✅ All services initialized successfully');
  } catch (error) {
    console.error('❌ Failed to initialize services:', error);
    process.exit(1);
  }
}

// API Routes

// Get collection information
app.get('/api/collections/:address', async (req, res) => {
  try {
    const { address } = req.params;
    const collection = await nftClient.getCollection(new PublicKey(address));
    
    const stats = await database.getCollectionStats(address);
    
    res.json({
      ...collection,
      stats,
    });
  } catch (error) {
    res.status(404).json({ error: 'Collection not found' });
  }
});

// Get NFT by token ID
app.get('/api/nfts/:tokenId', async (req, res) => {
  try {
    const { tokenId } = req.params;
    const nft = await database.getNFTByTokenId(parseInt(tokenId));
    
    if (!nft) {
      return res.status(404).json({ error: 'NFT not found' });
    }

    // Get current on-chain data
    const onChainData = await nftClient.getNftByTokenId(new BN(tokenId));
    
    res.json({
      ...nft,
      onChain: onChainData,
    });
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch NFT' });
  }
});

// Mint NFT endpoint
app.post('/api/mint', async (req, res) => {
  try {
    const { collection, name, symbol, uri, recipient } = req.body;

    // Validate inputs
    if (!collection || !name || !symbol || !uri) {
      return res.status(400).json({ error: 'Missing required fields' });
    }

    const result = await nftClient.mintNft(
      new PublicKey(collection),
      name,
      symbol,
      uri,
      recipient ? new PublicKey(recipient) : undefined
    );

    // Store in database
    await database.createNFT({
      tokenId: result.tokenId,
      mint: result.mint.toString(),
      collection,
      name,
      symbol,
      uri,
      recipient: recipient || nftClient.provider.wallet.publicKey.toString(),
      signature: result.signature,
      status: 'minted',
    });

    res.json(result);
  } catch (error) {
    console.error('Mint error:', error);
    res.status(500).json({ error: 'Failed to mint NFT' });
  }
});

// Cross-chain transfer endpoint
app.post('/api/transfer', async (req, res) => {
  try {
    const { collection, nftMint, destinationChain, recipient } = req.body;

    if (!collection || !nftMint || !destinationChain || !recipient) {
      return res.status(400).json({ error: 'Missing required fields' });
    }

    const result = await nftClient.transferCrossChain(
      new PublicKey(collection),
      new PublicKey(nftMint),
      destinationChain,
      recipient
    );

    // Update database
    await database.updateNFTStatus(result.tokenId.toString(), 'transferred', {
      destinationChain,
      recipient,
      transferSignature: result.signature,
    });

    res.json(result);
  } catch (error) {
    console.error('Transfer error:', error);
    res.status(500).json({ error: 'Failed to transfer NFT' });
  }
});

// Webhook endpoint for cross-chain events
app.post('/api/webhooks/cross-chain', async (req, res) => {
  try {
    const event = req.body;
    await webhookHandler.processEvent(event);
    res.json({ success: true });
  } catch (error) {
    console.error('Webhook error:', error);
    res.status(500).json({ error: 'Failed to process webhook' });
  }
});

// Health check endpoint
app.get('/api/health', async (req, res) => {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      services: {
        database: await database.healthCheck(),
        solana: await nftClient.connection.getHealth(),
        eventProcessor: eventProcessor.isRunning(),
      },
    };

    res.json(health);
  } catch (error) {
    res.status(500).json({
      status: 'unhealthy',
      error: error.message,
    });
  }
});

// Start server
async function startServer() {
  await initializeServices();
  
  app.listen(port, () => {
    console.log(`🚀 Server running on port ${port}`);
  });
}

startServer().catch(console.error);

export default app;
```

### Database Integration

```typescript
// server/database.ts
import { Sequelize, DataTypes, Model } from 'sequelize';

export interface NFTRecord {
  id?: number;
  tokenId: string;
  mint: string;
  collection: string;
  name: string;
  symbol: string;
  uri: string;
  recipient: string;
  signature: string;
  status: 'minted' | 'transferred' | 'received' | 'burned';
  metadata?: any;
  createdAt?: Date;
  updatedAt?: Date;
}

export interface TransferRecord {
  id?: number;
  tokenId: string;
  fromChain: number;
  toChain: number;
  sender: string;
  recipient: string;
  signature: string;
  status: 'pending' | 'completed' | 'failed';
  createdAt?: Date;
  updatedAt?: Date;
}

class NFT extends Model<NFTRecord> implements NFTRecord {
  public id!: number;
  public tokenId!: string;
  public mint!: string;
  public collection!: string;
  public name!: string;
  public symbol!: string;
  public uri!: string;
  public recipient!: string;
  public signature!: string;
  public status!: 'minted' | 'transferred' | 'received' | 'burned';
  public metadata?: any;
  public readonly createdAt!: Date;
  public readonly updatedAt!: Date;
}

class Transfer extends Model<TransferRecord> implements TransferRecord {
  public id!: number;
  public tokenId!: string;
  public fromChain!: number;
  public toChain!: number;
  public sender!: string;
  public recipient!: string;
  public signature!: string;
  public status!: 'pending' | 'completed' | 'failed';
  public readonly createdAt!: Date;
  public readonly updatedAt!: Date;
}

export class NFTDatabase {
  private sequelize: Sequelize;

  constructor(databaseUrl: string) {
    this.sequelize = new Sequelize(databaseUrl, {
      logging: process.env.NODE_ENV === 'development' ? console.log : false,
    });
  }

  async initialize() {
    // Define NFT model
    NFT.init({
      id: {
        type: DataTypes.INTEGER,
        autoIncrement: true,
        primaryKey: true,
      },
      tokenId: {
        type: DataTypes.STRING,
        allowNull: false,
        unique: true,
      },
      mint: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      collection: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      name: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      symbol: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      uri: {
        type: DataTypes.TEXT,
        allowNull: false,
      },
      recipient: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      signature: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      status: {
        type: DataTypes.ENUM('minted', 'transferred', 'received', 'burned'),
        allowNull: false,
        defaultValue: 'minted',
      },
      metadata: {
        type: DataTypes.JSON,
        allowNull: true,
      },
    }, {
      sequelize: this.sequelize,
      modelName: 'NFT',
      tableName: 'nfts',
    });

    // Define Transfer model
    Transfer.init({
      id: {
        type: DataTypes.INTEGER,
        autoIncrement: true,
        primaryKey: true,
      },
      tokenId: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      fromChain: {
        type: DataTypes.INTEGER,
        allowNull: false,
      },
      toChain: {
        type: DataTypes.INTEGER,
        allowNull: false,
      },
      sender: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      recipient: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      signature: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      status: {
        type: DataTypes.ENUM('pending', 'completed', 'failed'),
        allowNull: false,
        defaultValue: 'pending',
      },
    }, {
      sequelize: this.sequelize,
      modelName: 'Transfer',
      tableName: 'transfers',
    });

    // Define associations
    NFT.hasMany(Transfer, { foreignKey: 'tokenId', sourceKey: 'tokenId' });
    Transfer.belongsTo(NFT, { foreignKey: 'tokenId', targetKey: 'tokenId' });

    // Sync database
    await this.sequelize.sync();
  }

  async createNFT(nftData: Omit<NFTRecord, 'id' | 'createdAt' | 'updatedAt'>): Promise<NFT> {
    return await NFT.create(nftData);
  }

  async getNFTByTokenId(tokenId: string): Promise<NFT | null> {
    return await NFT.findOne({
      where: { tokenId },
      include: [Transfer],
    });
  }

  async updateNFTStatus(tokenId: string, status: NFTRecord['status'], metadata?: any): Promise<void> {
    await NFT.update(
      { status, metadata },
      { where: { tokenId } }
    );
  }

  async createTransfer(transferData: Omit<TransferRecord, 'id' | 'createdAt' | 'updatedAt'>): Promise<Transfer> {
    return await Transfer.create(transferData);
  }

  async getCollectionStats(collection: string) {
    const stats = await NFT.findAll({
      where: { collection },
      attributes: [
        'status',
        [this.sequelize.fn('COUNT', this.sequelize.col('status')), 'count'],
      ],
      group: ['status'],
      raw: true,
    });

    return stats.reduce((acc, stat: any) => {
      acc[stat.status] = parseInt(stat.count);
      return acc;
    }, {} as Record<string, number>);
  }

  async healthCheck(): Promise<boolean> {
    try {
      await this.sequelize.authenticate();
      return true;
    } catch {
      return false;
    }
  }
}
```

### Webhook Handler

```typescript
// server/webhooks.ts
import { NFTDatabase } from './database';

export interface CrossChainEvent {
  type: 'transfer' | 'receive' | 'revert';
  tokenId: string;
  fromChain: number;
  toChain: number;
  sender: string;
  recipient: string;
  signature: string;
  timestamp: string;
  metadata?: any;
}

export class WebhookHandler {
  constructor(private database: NFTDatabase) {}

  async processEvent(event: CrossChainEvent): Promise<void> {
    console.log(`Processing ${event.type} event for token ${event.tokenId}`);

    try {
      switch (event.type) {
        case 'transfer':
          await this.handleTransferEvent(event);
          break;
        case 'receive':
          await this.handleReceiveEvent(event);
          break;
        case 'revert':
          await this.handleRevertEvent(event);
          break;
        default:
          console.warn(`Unknown event type: ${event.type}`);
      }
    } catch (error) {
      console.error(`Failed to process ${event.type} event:`, error);
      throw error;
    }
  }

  private async handleTransferEvent(event: CrossChainEvent): Promise<void> {
    // Create transfer record
    await this.database.createTransfer({
      tokenId: event.tokenId,
      fromChain: event.fromChain,
      toChain: event.toChain,
      sender: event.sender,
      recipient: event.recipient,
      signature: event.signature,
      status: 'pending',
    });

    // Update NFT status
    await this.database.updateNFTStatus(event.tokenId, 'transferred', {
      transferEvent: event,
    });
  }

  private async handleReceiveEvent(event: CrossChainEvent): Promise<void> {
    // Update transfer status
    const transfer = await this.database.createTransfer({
      tokenId: event.tokenId,
      fromChain: event.fromChain,
      toChain: event.toChain,
      sender: event.sender,
      recipient: event.recipient,
      signature: event.signature,
      status: 'completed',
    });

    // Update or create NFT record
    const existingNFT = await this.database.getNFTByTokenId(event.tokenId);
    
    if (existingNFT) {
      await this.database.updateNFTStatus(event.tokenId, 'received', {
        receiveEvent: event,
      });
    } else {
      // Create new NFT record for incoming NFT
      await this.database.createNFT({
        tokenId: event.tokenId,
        mint: event.metadata?.mint || '',
        collection: event.metadata?.collection || '',
        name: event.metadata?.name || `NFT #${event.tokenId}`,
        symbol: event.metadata?.symbol || 'XNFT',
        uri: event.metadata?.uri || '',
        recipient: event.recipient,
        signature: event.signature,
        status: 'received',
        metadata: event.metadata,
      });
    }
  }

  private async handleRevertEvent(event: CrossChainEvent): Promise<void> {
    // Update NFT status back to original state
    await this.database.updateNFTStatus(event.tokenId, 'minted', {
      revertEvent: event,
    });

    // Mark transfer as failed
    await this.database.createTransfer({
      tokenId: event.tokenId,
      fromChain: event.fromChain,
      toChain: event.toChain,
      sender: event.sender,
      recipient: event.recipient,
      signature: event.signature,
      status: 'failed',
    });
  }
}
```

## Cross-Chain Scenarios

### Complete Transfer Scenarios

#### Scenario 1: Mint on Solana and Transfer to Ethereum

```typescript
// examples/solana-to-ethereum.ts
import { UniversalNftClient, Network, UniversalNftUtils } from '../sdk/client';
import { Keypair, PublicKey } from '@solana/web3.js';

async function mintAndTransferToEthereum() {
  // Initialize client
  const client = await UniversalNftClient.create(
    { network: Network.DEVNET },
    { keypair: Keypair.generate() }
  );

  try {
    // Step 1: Initialize collection
    const collectionResult = await client.initializeCollection(
      "Cross-Chain Collection",
      "XCC",
      "https://example.com/collection.json",
      UniversalNftUtils.ethAddressToBytes("0x1234567890123456789012345678901234567890")
    );

    console.log(`Collection created: ${collectionResult.collection.toString()}`);

    // Step 2: Mint NFT on Solana
    const mintResult = await client.mintNft(
      collectionResult.collection,
      "Cross-Chain NFT #1",
      "XC1",
      "https://example.com/nft1.json"
    );

    console.log(`NFT minted: ${mintResult.mint.toString()}`);
    console.log(`Token ID: ${mintResult.tokenId}`);

    // Step 3: Transfer to Ethereum
    const transferResult = await client.transferCrossChain(
      collectionResult.collection,
      mintResult.mint,
      11155111, // Ethereum Sepolia
      UniversalNftUtils.ethAddressToBytes("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
    );

    console.log(`Transfer initiated: ${transferResult.signature}`);
    console.log(`Destination: Ethereum Sepolia`);
    console.log(`Recipient: 0xabcdefabcdefabcdefabcdefabcdefabcdefabcd`);

    // Step 4: Monitor for completion
    await client.waitForConfirmation(transferResult.signature);
    console.log(`Transfer confirmed on Solana`);

    return {
      collection: collectionResult.collection,
      mint: mintResult.mint,
      tokenId: mintResult.tokenId,
      transferSignature: transferResult.signature,
    };

  } catch (error) {
    console.error('Transfer failed:', error);
    throw error;
  } finally {
    client.dispose();
  }
}

// Usage
mintAndTransferToEthereum()
  .then(result => console.log('Success:', result))
  .catch(error => console.error('Error:', error));
```

#### Scenario 2: Receive NFT from EVM Chain

```typescript
// examples/receive-from-evm.ts
import { UniversalNftClient, Network } from '../sdk/client';
import { Keypair, PublicKey } from '@solana/web3.js';

async function simulateReceiveFromEVM() {
  const client = await UniversalNftClient.create(
    { network: Network.DEVNET },
    { keypair: Keypair.generate() }
  );

  try {
    // Simulate incoming cross-chain message from Ethereum
    const incomingMessage = createEthereumMessage({
      tokenId: 12345n,
      name: "Ethereum NFT",
      symbol: "ETH",
      uri: "https://ethereum.example.com/nft12345.json",
      recipient: client.provider.wallet.publicKey,
      originChain: 1, // Ethereum mainnet
    });

    // This would normally be called by the ZetaChain gateway
    const receiveResult = await client.onCall(
      new PublicKey("YourCollectionAddress"), // Collection address
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20], // Sender
      1, // Source chain (Ethereum)
      incomingMessage,
      1 // Nonce
    );

    console.log(`NFT received from Ethereum:`);
    console.log(`New mint: ${receiveResult.mint.toString()}`);
    console.log(`Is returning: ${receiveResult.isReturning}`);

    return receiveResult;

  } catch (error) {
    console.error('Receive failed:', error);
    throw error;
  } finally {
    client.dispose();
  }
}

function createEthereumMessage(params: {
  tokenId: bigint;
  name: string;
  symbol: string;
  uri: string;
  recipient: PublicKey;
  originChain: number;
}): number[] {
  const message: number[] = [];
  
  // Token ID (8 bytes, little-endian)
  const tokenIdBuffer = Buffer.alloc(8);
  tokenIdBuffer.writeBigUInt64LE(params.tokenId);
  message.push(...Array.from(tokenIdBuffer));
  
  // Name length and data
  const nameBuffer = Buffer.from(params.name, 'utf8');
  const nameLengthBuffer = Buffer.alloc(4);
  nameLengthBuffer.writeUInt32LE(nameBuffer.length);
  message.push(...Array.from(nameLengthBuffer));
  message.push(...Array.from(nameBuffer));
  
  // Symbol length and data
  const symbolBuffer = Buffer.from(params.symbol, 'utf8');
  const symbolLengthBuffer = Buffer.alloc(4);
  symbolLengthBuffer.writeUInt32LE(symbolBuffer.length);
  message.push(...Array.from(symbolLengthBuffer));
  message.push(...Array.from(symbolBuffer));
  
  // URI length and data
  const uriBuffer = Buffer.from(params.uri, 'utf8');
  const uriLengthBuffer = Buffer.alloc(4);
  uriLengthBuffer.writeUInt32LE(uriBuffer.length);
  message.push(...Array.from(uriLengthBuffer));
  message.push(...Array.from(uriBuffer));
  
  // Recipient (32 bytes)
  message.push(...Array.from(params.recipient.toBuffer()));
  
  // Origin chain (8 bytes)
  const originChainBuffer = Buffer.alloc(8);
  originChainBuffer.writeBigUInt64LE(BigInt(params.originChain));
  message.push(...Array.from(originChainBuffer));
  
  return message;
}
```

#### Scenario 3: Round-Trip Transfer with Origin Preservation

```typescript
// examples/round-trip-transfer.ts
import { UniversalNftClient, Network } from '../sdk/client';
import { Keypair, PublicKey } from '@solana/web3.js';

async function demonstrateRoundTrip() {
  const client = await UniversalNftClient.create(
    { network: Network.DEVNET },
    { keypair: Keypair.generate() }
  );

  try {
    // Step 1: Mint original NFT on Solana
    const collection = new PublicKey("YourCollectionAddress");
    
    const originalMint = await client.mintNft(
      collection,
      "Round-Trip NFT",
      "RT",
      "https://example.com/round-trip.json"
    );

    console.log(`Original NFT minted on Solana: ${originalMint.mint.toString()}`);

    // Get origin data
    const originData = await client.getNftByTokenId(originalMint.tokenId);
    console.log(`Origin chain: ${originData.account.chainOfOrigin.toNumber()}`);
    console.log(`Is native: ${originData.account.isNative}`);

    // Step 2: Transfer to Ethereum
    await client.transferCrossChain(
      collection,
      originalMint.mint,
      11155111, // Ethereum Sepolia
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
    );

    console.log(`NFT transferred to Ethereum`);

    // Step 3: Simulate return from Ethereum
    const returnMessage = createReturnMessage({
      tokenId: originalMint.tokenId,
      originalUri: "https://example.com/round-trip.json",
      recipient: client.provider.wallet.publicKey,
    });

    const returnResult = await client.onCall(
      collection,
      [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
      11155111, // From Ethereum
      returnMessage,
      2
    );

    console.log(`NFT returned to Solana: ${returnResult.mint.toString()}`);
    console.log(`Is returning NFT: ${returnResult.isReturning}`);

    // Step 4: Verify origin data preserved
    const finalOriginData = await client.getNftByTokenId(originalMint.tokenId);
    console.log(`Origin preserved - Chain: ${finalOriginData.account.chainOfOrigin.toNumber()}`);
    console.log(`Original mint preserved: ${finalOriginData.account.originalMint.toString()}`);

    return {
      originalMint: originalMint.mint,
      returnedMint: returnResult.mint,
      originPreserved: finalOriginData.account.originalMint.equals(originalMint.mint),
    };

  } catch (error) {
    console.error('Round-trip failed:', error);
    throw error;
  } finally {
    client.dispose();
  }
}

function createReturnMessage(params: {
  tokenId: BN;
  originalUri: string;
  recipient: PublicKey;
}): number[] {
  // Create message indicating this is a returning NFT
  const message: number[] = [];
  
  // Token ID
  const tokenIdBuffer = Buffer.alloc(8);
  tokenIdBuffer.writeBigUInt64LE(BigInt(params.tokenId.toString()));
  message.push(...Array.from(tokenIdBuffer));
  
  // URI
  const uriBuffer = Buffer.from(params.originalUri, 'utf8');
  const uriLengthBuffer = Buffer.alloc(4);
  uriLengthBuffer.writeUInt32LE(uriBuffer.length);
  message.push(...Array.from(uriLengthBuffer));
  message.push(...Array.from(uriBuffer));
  
  // Recipient
  message.push(...Array.from(params.recipient.toBuffer()));
  
  // Return flag (1 byte indicating this is a return)
  message.push(1);
  
  return message;
}
```

### Error Handling and Recovery

```typescript
// examples/error-handling.ts
import { UniversalNftClient, UniversalNftError, TransactionError } from '../sdk/client';

class CrossChainErrorHandler {
  constructor(private client: UniversalNftClient) {}

  async safeTransfer(
    collection: PublicKey,
    nftMint: PublicKey,
    destinationChain: number,
    recipient: number[],
    retries: number = 3
  ) {
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= retries; attempt++) {
      try {
        console.log(`Transfer attempt ${attempt}/${retries}`);

        // Pre-flight checks
        await this.validateTransfer(collection, nftMint, destinationChain, recipient);

        // Execute transfer
        const result = await this.client.transferCrossChain(
          collection,
          nftMint,
          destinationChain,
          recipient
        );

        // Wait for confirmation
        await this.client.waitForConfirmation(result.signature, 60000);

        console.log(`Transfer successful on attempt ${attempt}`);
        return result;

      } catch (error) {
        lastError = error as Error;
        console.error(`Transfer attempt ${attempt} failed:`, error.message);

        if (error instanceof UniversalNftError) {
          // Handle specific Universal NFT errors
          switch (error.code) {
            case 'ACCOUNT_NOT_FOUND':
              throw new Error('NFT or collection not found');
            case 'INVALID_PARAMETER':
              throw new Error('Invalid transfer parameters');
            case 'TRANSACTION_ERROR':
              // Retry transaction errors
              if (attempt === retries) {
                throw new Error(`Transaction failed after ${retries} attempts`);
              }
              break;
            default:
              throw error;
          }
        }

        // Wait before retry
        if (attempt < retries) {
          await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
        }
      }
    }

    throw lastError || new Error('Transfer failed');
  }

  private async validateTransfer(
    collection: PublicKey,
    nftMint: PublicKey,
    destinationChain: number,
    recipient: number[]
  ) {
    // Validate collection exists
    try {
      await this.client.getCollection(collection);
    } catch {
      throw new Error('Collection not found or not accessible');
    }

    // Validate NFT ownership
    const tokenAccount = await getAssociatedTokenAddress(
      nftMint,
      this.client.provider.wallet.publicKey
    );

    try {
      const tokenAccountInfo = await getAccount(this.client.connection, tokenAccount);
      if (tokenAccountInfo.amount === 0n) {
        throw new Error('You do not own this NFT');
      }
    } catch {
      throw new Error('NFT token account not found');
    }

    // Validate destination chain
    const supportedChains = [1, 56, 8453, 11155111, 97, 84532]; // Add your supported chains
    if (!supportedChains.includes(destinationChain)) {
      throw new Error(`Unsupported destination chain: ${destinationChain}`);
    }

    // Validate recipient address
    if (!recipient || recipient.length !== 20) {
      throw new Error('Invalid recipient address');
    }
  }

  async handleFailedTransfer(signature: string) {
    try {
      const status = await this.client.getTransactionStatus(signature);
      
      if (status.value?.err) {
        console.error('Transaction failed:', status.value.err);
        
        // Analyze error and provide recovery options
        const errorAnalysis = this.analyzeTransactionError(status.value.err);
        return {
          canRetry: errorAnalysis.canRetry,
          suggestedAction: errorAnalysis.suggestedAction,
          error: status.value.err,
        };
      }

      return { canRetry: false, suggestedAction: 'Transaction succeeded' };
    } catch (error) {
      console.error('Failed to analyze transaction:', error);
      return {
        canRetry: true,
        suggestedAction: 'Check network connection and retry',
        error: error.message,
      };
    }
  }

  private analyzeTransactionError(error: any) {
    // Analyze common Solana transaction errors
    const errorString = JSON.stringify(error);

    if (errorString.includes('InsufficientFunds')) {
      return {
        canRetry: false,
        suggestedAction: 'Add more SOL to your wallet for transaction fees',
      };
    }

    if (errorString.includes('BlockhashNotFound')) {
      return {
        canRetry: true,
        suggestedAction: 'Transaction expired, safe to retry',
      };
    }

    if (errorString.includes('AccountNotFound')) {
      return {
        canRetry: false,
        suggestedAction: 'NFT or account not found, check addresses',
      };
    }

    return {
      canRetry: true,
      suggestedAction: 'Unknown error, safe to retry with caution',
    };
  }
}
```

## Testing Examples

### Unit Test Examples

```typescript
// tests/unit/client.test.ts
import { expect } from 'chai';
import { Keypair, PublicKey } from '@solana/web3.js';
import { UniversalNftClient, UniversalNftUtils, Network } from '../../sdk/client';

describe('UniversalNftClient Unit Tests', () => {
  let client: UniversalNftClient;
  let testKeypair: Keypair;

  beforeEach(async () => {
    testKeypair = Keypair.generate();
    client = await UniversalNftClient.create(
      { network: Network.DEVNET },
      { keypair: testKeypair }
    );
  });

  afterEach(() => {
    client?.dispose();
  });

  describe('PDA Derivation', () => {
    it('should derive collection PDA correctly', () => {
      const authority = testKeypair.publicKey;
      const name = "Test Collection";
      
      const [pda, bump] = client.deriveCollectionPda(authority, name);
      
      expect(pda).to.be.instanceOf(PublicKey);
      expect(bump).to.be.a('number');
      expect(bump).to.be.at.least(0);
      expect(bump).to.be.at.most(255);
    });

    it('should derive NFT Origin PDA correctly', () => {
      const tokenId = new BN(12345);
      
      const [pda, bump] = client.deriveNftOriginPda(tokenId);
      
      expect(pda).to.be.instanceOf(PublicKey);
      expect(bump).to.be.a('number');
    });

    it('should derive different PDAs for different inputs', () => {
      const [pda1] = client.deriveNftOriginPda(new BN(1));
      const [pda2] = client.deriveNftOriginPda(new BN(2));
      
      expect(pda1.toString()).to.not.equal(pda2.toString());
    });
  });

  describe('Utility Functions', () => {
    it('should convert Ethereum address to bytes correctly', () => {
      const ethAddress = "0x1234567890123456789012345678901234567890";
      const bytes = UniversalNftUtils.ethAddressToBytes(ethAddress);
      
      expect(bytes).to.have.length(20);
      expect(bytes[0]).to.equal(0x12);
      expect(bytes[1]).to.equal(0x34);
    });

    it('should convert bytes to Ethereum address correctly', () => {
      const bytes = [0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90, 
                    0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90];
      const address = UniversalNftUtils.bytesToEthAddress(bytes);
      
      expect(address).to.equal("0x1234567890123456789012345678901234567890");
    });

    it('should validate Solana public keys', () => {
      const validKey = testKeypair.publicKey.toString();
      const invalidKey = "invalid-key";
      
      expect(UniversalNftUtils.isValidPublicKey(validKey)).to.be.true;
      expect(UniversalNftUtils.isValidPublicKey(invalidKey)).to.be.false;
    });

    it('should validate Ethereum addresses', () => {
      const validAddress = "0x1234567890123456789012345678901234567890";
      const invalidAddress = "0x123"; // Too short
      
      expect(UniversalNftUtils.isValidEthAddress(validAddress)).to.be.true;
      expect(UniversalNftUtils.isValidEthAddress(invalidAddress)).to.be.false;
    });
  });

  describe('Token ID Generation', () => {
    it('should generate deterministic token IDs', () => {
      const mint = testKeypair.publicKey;
      const authority = Keypair.generate().publicKey;
      
      const tokenId1 = client.generateTokenId(mint, authority);
      const tokenId2 = client.generateTokenId(mint, authority);
      
      expect(tokenId1.toString()).to.equal(tokenId2.toString());
    });

    it('should generate different token IDs for different inputs', () => {
      const mint1 = testKeypair.publicKey;
      const mint2 = Keypair.generate().publicKey;
      const authority = Keypair.generate().publicKey;
      
      const tokenId1 = client.generateTokenId(mint1, authority);
      const tokenId2 = client.generateTokenId(mint2, authority);
      
      expect(tokenId1.toString()).to.not.equal(tokenId2.toString());
    });
  });

  describe('Message Parsing', () => {
    it('should parse valid cross-chain messages', () => {
      const tokenId = new BN(12345);
      const name = "Test NFT";
      const symbol = "TEST";
      const uri = "https://example.com/test.json";
      const recipient = testKeypair.publicKey;
      
      // Create a mock message
      const message = createMockMessage(tokenId, name, symbol, uri, recipient);
      
      const parsed = client.parseMessage(message);
      
      expect(parsed.tokenId.toString()).to.equal(tokenId.toString());
      expect(parsed.recipient.toString()).to.equal(recipient.toString());
    });

    it('should handle invalid messages gracefully', () => {
      const invalidMessage = [1, 2, 3]; // Too short
      
      expect(() => client.parseMessage(invalidMessage)).to.throw();
    });
  });
});

function createMockMessage(
  tokenId: BN,
  name: string,
  symbol: string,
  uri: string,
  recipient: PublicKey
): number[] {
  const message: number[] = [];
  
  // Token ID (8 bytes)
  const tokenIdBuffer = Buffer.alloc(8);
  tokenId.toArrayLike(Buffer, 'le', 8).copy(tokenIdBuffer);
  message.push(...Array.from(tokenIdBuffer));
  
  // For simplicity, just add recipient
  message.push(...Array.from(recipient.toBuffer()));
  
  return message;
}
```

### Integration Test Patterns

```typescript
// tests/integration/cross-chain.test.ts
import { expect } from 'chai';
import { Keypair, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { UniversalNftClient, Network } from '../../sdk/client';

describe('Cross-Chain Integration Tests', () => {
  let client: UniversalNftClient;
  let authority: Keypair;
  let collection: PublicKey;

  before(async () => {
    authority = Keypair.generate();
    
    // Airdrop SOL for testing
    const connection = new Connection('https://api.devnet.solana.com');
    const signature = await connection.requestAirdrop(
      authority.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);

    client = await UniversalNftClient.create(
      { network: Network.DEVNET },
      { keypair: authority }
    );
  });

  after(() => {
    client?.dispose();
  });

  describe('End-to-End NFT Lifecycle', () => {
    it('should complete full NFT lifecycle', async () => {
      // Step 1: Initialize collection
      const collectionResult = await client.initializeCollection(
        "Integration Test Collection",
        "ITC",
        "https://example.com/collection.json",
        Array.from(Buffer.alloc(20, 1))
      );

      collection = collectionResult.collection;
      expect(collection).to.be.instanceOf(PublicKey);

      // Step 2: Mint NFT
      const mintResult = await client.mintNft(
        collection,
        "Test NFT",
        "TEST",
        "https://example.com/test.json"
      );

      expect(mintResult.mint).to.be.instanceOf(PublicKey);
      expect(mintResult.signature).to.be.a('string');

      // Step 3: Verify NFT was minted
      const originData = await client.getNftByTokenId(mintResult.tokenId);
      expect(originData.account.tokenId.toString()).to.equal(mintResult.tokenId.toString());

      // Step 4: Transfer cross-chain
      const transferResult = await client.transferCrossChain(
        collection,
        mintResult.mint,
        11155111, // Ethereum Sepolia
        Array.from(Buffer.alloc(20, 2))
      );

      expect(transferResult.signature).to.be.a('string');

      // Step 5: Verify transfer completed
      await client.waitForConfirmation(transferResult.signature);
      
      // Origin data should still exist
      const postTransferOrigin = await client.getNftByTokenId(mintResult.tokenId);
      expect(postTransferOrigin.account.tokenId.toString()).to.equal(mintResult.tokenId.toString());
    });
  });

  describe('Error Scenarios', () => {
    it('should handle invalid collection gracefully', async () => {
      const invalidCollection = Keypair.generate().publicKey;
      
      try {
        await client.mintNft(
          invalidCollection,
          "Test NFT",
          "TEST",
          "https://example.com/test.json"
        );
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.message).to.include('Collection not found');
      }
    });

    it('should handle invalid transfer parameters', async () => {
      const invalidMint = Keypair.generate().publicKey;
      
      try {
        await client.transferCrossChain(
          collection,
          invalidMint,
          11155111,
          Array.from(Buffer.alloc(20, 1))
        );
        expect.fail('Should have thrown error');
      } catch (error) {
        expect(error.message).to.include('Token account not found');
      }
    });
  });
});
```

### Mock Data and Test Utilities

```typescript
// tests/utils/mock-data.ts
import { Keypair, PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

export class MockDataGenerator {
  static generateKeypair(): Keypair {
    return Keypair.generate();
  }

  static generateCollectionData() {
    return {
      name: `Test Collection ${Date.now()}`,
      symbol: "TC",
      uri: "https://example.com/collection.json",
      tssAddress: Array.from(Buffer.alloc(20, 1)),
    };
  }

  static generateNFTData() {
    const id = Date.now();
    return {
      name: `Test NFT #${id}`,
      symbol: "TN",
      uri: `https://example.com/nft${id}.json`,
    };
  }

  static generateCrossChainMessage(tokenId: BN, recipient: PublicKey): number[] {
    const message: number[] = [];
    
    // Token ID
    const tokenIdBuffer = Buffer.alloc(8);
    tokenId.toArrayLike(Buffer, 'le', 8).copy(tokenIdBuffer);
    message.push(...Array.from(tokenIdBuffer));
    
    // URI
    const uri = `https://example.com/nft${tokenId.toString()}.json`;
    const uriBuffer = Buffer.from(uri, 'utf8');
    const uriLengthBuffer = Buffer.alloc(4);
    uriLengthBuffer.writeUInt32LE(uriBuffer.length);
    message.push(...Array.from(uriLengthBuffer));
    message.push(...Array.from(uriBuffer));
    
    // Recipient
    message.push(...Array.from(recipient.toBuffer()));
    
    return message;
  }

  static generateEthereumAddress(): string {
    const bytes = Array.from(Buffer.alloc(20)).map(() => 
      Math.floor(Math.random() * 256)
    );
    return '0x' + Buffer.from(bytes).toString('hex');
  }

  static generateChainId(): number {
    const supportedChains = [1, 56, 8453, 11155111, 97, 84532];
    return supportedChains[Math.floor(Math.random() * supportedChains.length)];
  }
}

export class TestHelpers {
  static async airdropSol(
    connection: Connection,
    publicKey: PublicKey,
    amount: number = 2 * LAMPORTS_PER_SOL
  ): Promise<void> {
    const signature = await connection.requestAirdrop(publicKey, amount);
    await connection.confirmTransaction(signature);
  }

  static async waitForTransaction(
    connection: Connection,
    signature: string,
    timeout: number = 30000
  ): Promise<void> {
    const start = Date.now();
    
    while (Date.now() - start < timeout) {
      const status = await connection.getSignatureStatus(signature);
      
      if (status.value?.confirmationStatus === 'confirmed') {
        return;
      }
      
      if (status.value?.err) {
        throw new Error(`Transaction failed: ${JSON.stringify(status.value.err)}`);
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new Error('Transaction confirmation timeout');
  }

  static expectPublicKey(value: any): void {
    expect(value).to.be.instanceOf(PublicKey);
  }

  static expectValidSignature(signature: any): void {
    expect(signature).to.be.a('string');
    expect(signature).to.have.length.greaterThan(80);
  }

  static expectValidTokenId(tokenId: any): void {
    expect(tokenId).to.be.instanceOf(BN);
    expect(tokenId.toNumber()).to.be.greaterThan(0);
  }
}
```

### Performance Testing Guidelines

```typescript
// tests/performance/load-test.ts
import { performance } from 'perf_hooks';
import { UniversalNftClient, Network } from '../../sdk/client';
import { MockDataGenerator, TestHelpers } from '../utils/mock-data';

describe('Performance Tests', () => {
  let client: UniversalNftClient;
  let collection: PublicKey;

  before(async () => {
    const authority = MockDataGenerator.generateKeypair();
    await TestHelpers.airdropSol(connection, authority.publicKey);
    
    client = await UniversalNftClient.create(
      { network: Network.DEVNET },
      { keypair: authority }
    );

    // Initialize test collection
    const collectionData = MockDataGenerator.generateCollectionData();
    const result = await client.initializeCollection(
      collectionData.name,
      collectionData.symbol,
      collectionData.uri,
      collectionData.tssAddress
    );
    collection = result.collection;
  });

  after(() => {
    client?.dispose();
  });

  describe('Minting Performance', () => {
    it('should mint NFTs within acceptable time limits', async () => {
      const iterations = 10;
      const times: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const nftData = MockDataGenerator.generateNFTData();
        
        const start = performance.now();
        
        await client.mintNft(
          collection,
          nftData.name,
          nftData.symbol,
          nftData.uri
        );
        
        const end = performance.now();
        times.push(end - start);
      }

      const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
      const maxTime = Math.max(...times);
      
      console.log(`Average mint time: ${avgTime.toFixed(2)}ms`);
      console.log(`Max mint time: ${maxTime.toFixed(2)}ms`);
      
      // Assert performance requirements
      expect(avgTime).to.be.lessThan(5000); // 5 seconds average
      expect(maxTime).to.be.lessThan(10000); // 10 seconds max
    });
  });

  describe('Concurrent Operations', () => {
    it('should handle concurrent mints efficiently', async () => {
      const concurrentCount = 5;
      const promises: Promise<any>[] = [];

      const start = performance.now();

      for (let i = 0; i < concurrentCount; i++) {
        const nftData = MockDataGenerator.generateNFTData();
        
        const promise = client.mintNft(
          collection,
          nftData.name,
          nftData.symbol,
          nftData.uri
        );
        
        promises.push(promise);
      }

      const results = await Promise.all(promises);
      const end = performance.now();

      const totalTime = end - start;
      const avgTimePerMint = totalTime / concurrentCount;

      console.log(`Concurrent mints completed in: ${totalTime.toFixed(2)}ms`);
      console.log(`Average time per mint: ${avgTimePerMint.toFixed(2)}ms`);

      expect(results).to.have.length(concurrentCount);
      expect(totalTime).to.be.lessThan(15000); // 15 seconds total
    });
  });

  describe('Memory Usage', () => {
    it('should not leak memory during extended operations', async () => {
      const initialMemory = process.memoryUsage();
      
      // Perform many operations
      for (let i = 0; i < 50; i++) {
        const nftData = MockDataGenerator.generateNFTData();
        await client.mintNft(collection, nftData.name, nftData.symbol, nftData.uri);
        
        // Force garbage collection if available
        if (global.gc) {
          global.gc();
        }
      }

      const finalMemory = process.memoryUsage();
      const memoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;
      
      console.log(`Memory increase: ${(memoryIncrease / 1024 / 1024).toFixed(2)}MB`);
      
      // Memory increase should be reasonable
      expect(memoryIncrease).to.be.lessThan(100 * 1024 * 1024); // 100MB
    });
  });
});
```

## Production Examples

### Monitoring Setup and Dashboard Configuration

```typescript
// monitoring/production-monitor.ts
import { UniversalNftClient, Network } from '../sdk/client';
import { Connection, PublicKey } from '@solana/web3.js';
import { EventEmitter } from 'events';

interface MonitoringConfig {
  collections: string[];
  alertThresholds: {
    errorRate: number;
    latency: number;
    failedTransactions: number;
  };
  webhookUrl?: string;
  dashboardPort?: number;
}

export class ProductionMonitor extends EventEmitter {
  private client: UniversalNftClient;
  private metrics: Map<string, any> = new Map();
  private isRunning: boolean = false;

  constructor(
    private config: MonitoringConfig,
    client: UniversalNftClient
  ) {
    super();
    this.client = client;
  }

  async start(): Promise<void> {
    if (this.isRunning) return;

    console.log('🚀 Starting production monitoring...');
    
    this.isRunning = true;
    
    // Start monitoring each collection
    for (const collectionAddress of this.config.collections) {
      await this.monitorCollection(new PublicKey(collectionAddress));
    }

    // Start health checks
    this.startHealthChecks();
    
    // Start metrics collection
    this.startMetricsCollection();

    console.log('✅ Production monitoring started');
  }

  async stop(): Promise<void> {
    this.isRunning = false;
    this.client.unsubscribeAll();
    console.log('🛑 Production monitoring stopped');
  }

  private async monitorCollection(collection: PublicKey): Promise<void> {
    // Monitor NFT mints
    await this.client.onNftMinted(async (event) => {
      this.recordMetric('nft_minted', {
        collection: event.collection.toString(),
        timestamp: Date.now(),
        tokenId: event.tokenId.toString(),
      });

      await this.checkAlerts('mint', event);
    }, collection);

    // Monitor cross-chain transfers
    await this.client.onCrossChainTransfer(async (event) => {
      this.recordMetric('cross_chain_transfer', {
        collection: event.collection.toString(),
        destinationChain: event.destinationChain,
        timestamp: Date.now(),
        tokenId: event.tokenId.toString(),
      });

      await this.checkAlerts('transfer', event);
    }, collection);

    // Monitor cross-chain receives
    await this.client.onCrossChainReceive(async (event) => {
      this.recordMetric('cross_chain_receive', {
        collection: event.collection.toString(),
        sourceChain: event.sourceChain,
        timestamp: Date.now(),
        tokenId: event.tokenId.toString(),
        isReturning: event.isReturning,
      });

      await this.checkAlerts('receive', event);
    }, collection);
  }

  private startHealthChecks(): void {
    setInterval(async () => {
      if (!this.isRunning) return;

      try {
        // Check Solana connection
        const slot = await this.client.connection.getSlot();
        this.recordMetric('health_check', {
          type: 'solana_connection',
          status: 'healthy',
          slot,
          timestamp: Date.now(),
        });

        // Check program account
        const programAccount = await this.client.connection.getAccountInfo(
          this.client.program.programId
        );
        
        if (!programAccount) {
          throw new Error('Program account not found');
        }

        this.recordMetric('health_check', {
          type: 'program_account',
          status: 'healthy',
          timestamp: Date.now(),
        });

      } catch (error) {
        this.recordMetric('health_check', {
          type: 'error',
          status: 'unhealthy',
          error: error.message,
          timestamp: Date.now(),
        });

        await this.sendAlert('health_check_failed', { error: error.message });
      }
    }, 30000); // Every 30 seconds
  }

  private startMetricsCollection(): void {
    setInterval(() => {
      if (!this.isRunning) return;

      const now = Date.now();
      const oneHour = 60 * 60 * 1000;

      // Calculate metrics for the last hour
      const recentMetrics = Array.from(this.metrics.values())
        .filter((metric: any) => now - metric.timestamp < oneHour);

      const stats = {
        totalEvents: recentMetrics.length,
        mintCount: recentMetrics.filter((m: any) => m.type === 'nft_minted').length,
        transferCount: recentMetrics.filter((m: any) => m.type === 'cross_chain_transfer').length,
        receiveCount: recentMetrics.filter((m: any) => m.type === 'cross_chain_receive').length,
        errorCount: recentMetrics.filter((m: any) => m.type === 'error').length,
        timestamp: now,
      };

      this.emit('metrics', stats);

      // Check alert thresholds
      const errorRate = stats.errorCount / Math.max(stats.totalEvents, 1);
      if (errorRate > this.config.alertThresholds.errorRate) {
        this.sendAlert('high_error_rate', { errorRate, threshold: this.config.alertThresholds.errorRate });
      }

    }, 60000); // Every minute
  }

  private recordMetric(type: string, data: any): void {
    const id = `${type}_${Date.now()}_${Math.random()}`;
    this.metrics.set(id, { type, ...data });

    // Clean up old metrics (keep last 24 hours)
    const oneDayAgo = Date.now() - (24 * 60 * 60 * 1000);
    for (const [key, metric] of this.metrics.entries()) {
      if ((metric as any).timestamp < oneDayAgo) {
        this.metrics.delete(key);
      }
    }
  }

  private async checkAlerts(eventType: string, event: any): Promise<void> {
    // Implement custom alert logic based on event patterns
    // This is a simplified example
    
    if (eventType === 'transfer') {
      // Check for unusual transfer patterns
      const recentTransfers = Array.from(this.metrics.values())
        .filter((m: any) => 
          m.type === 'cross_chain_transfer' && 
          Date.now() - m.timestamp < 300000 // Last 5 minutes
        );

      if (recentTransfers.length > 10) {
        await this.sendAlert('high_transfer_volume', {
          count: recentTransfers.length,
          timeframe: '5 minutes',
        });
      }
    }
  }

  private async sendAlert(type: string, data: any): Promise<void> {
    const alert = {
      type,
      data,
      timestamp: new Date().toISOString(),
      severity: this.getAlertSeverity(type),
    };

    console.warn(`🚨 ALERT [${alert.severity}]: ${type}`, data);

    // Send to webhook if configured
    if (this.config.webhookUrl) {
      try {
        await fetch(this.config.webhookUrl, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(alert),
        });
      } catch (error) {
        console.error('Failed to send webhook alert:', error);
      }
    }

    this.emit('alert', alert);
  }

  private getAlertSeverity(type: string): 'low' | 'medium' | 'high' | 'critical' {
    const severityMap: Record<string, 'low' | 'medium' | 'high' | 'critical'> = {
      'health_check_failed': 'critical',
      'high_error_rate': 'high',
      'high_transfer_volume': 'medium',
      'unusual_pattern': 'low',
    };

    return severityMap[type] || 'medium';
  }

  getMetrics(): any {
    return {
      totalMetrics: this.metrics.size,
      isRunning: this.isRunning,
      collections: this.config.collections,
      recentActivity: Array.from(this.metrics.values())
        .slice(-10)
        .sort((a: any, b: any) => b.timestamp - a.timestamp),
    };
  }
}
```

### Deployment Automation Scripts

```typescript
// scripts/deploy-production.ts
import { Keypair, PublicKey, Connection } from '@solana/web3.js';
import { UniversalNftClient, Network } from '../sdk/client';
import { ProductionMonitor } from '../monitoring/production-monitor';
import * as fs from 'fs';
import * as path from 'path';

interface DeploymentConfig {
  network: Network;
  collections: Array<{
    name: string;
    symbol: string;
    uri: string;
    tssAddress: string;
  }>;
  monitoring: {
    enabled: boolean;
    webhookUrl?: string;
    alertThresholds: {
      errorRate: number;
      latency: number;
      failedTransactions: number;
    };
  };
}

class ProductionDeployer {
  private client: UniversalNftClient;
  private config: DeploymentConfig;
  private deploymentLog: any[] = [];

  constructor(config: DeploymentConfig) {
    this.config = config;
  }

  async deploy(): Promise<void> {
    console.log('🚀 Starting production deployment...');
    
    try {
      // Step 1: Initialize client
      await this.initializeClient();
      
      // Step 2: Pre-deployment validation
      await this.validateEnvironment();
      
      // Step 3: Deploy collections
      const collections = await this.deployCollections();
      
      // Step 4: Setup monitoring
      if (this.config.monitoring.enabled) {
        await this.setupMonitoring(collections);
      }
      
      // Step 5: Post-deployment validation
      await this.validateDeployment(collections);
      
      // Step 6: Generate deployment report
      await this.generateDeploymentReport(collections);
      
      console.log('✅ Production deployment completed successfully');
      
    } catch (error) {
      console.error('❌ Deployment failed:', error);
      await this.rollback();
      throw error;
    }
  }

  private async initializeClient(): Promise<void> {
    this.log('Initializing client...');
    
    const privateKey = process.env.DEPLOYMENT_PRIVATE_KEY;
    if (!privateKey) {
      throw new Error('DEPLOYMENT_PRIVATE_KEY environment variable required');
    }

    const keypair = Keypair.fromSecretKey(
      new Uint8Array(JSON.parse(privateKey))
    );

    this.client = await UniversalNftClient.create(
      {
        network: this.config.network,
        commitment: 'confirmed',
      },
      {
        keypair,
      }
    );

    this.log('Client initialized successfully');
  }

  private async validateEnvironment(): Promise<void> {
    this.log('Validating environment...');
    
    // Check Solana connection
    const slot = await this.client.connection.getSlot();
    if (!slot) {
      throw new Error('Failed to connect to Solana network');
    }

    // Check wallet balance
    const balance = await this.client.connection.getBalance(
      this.client.provider.wallet.publicKey
    );
    
    const requiredBalance = 1000000000; // 1 SOL
    if (balance < requiredBalance) {
      throw new Error(`Insufficient balance. Required: ${requiredBalance}, Available: ${balance}`);
    }

    // Check program deployment
    const programAccount = await this.client.connection.getAccountInfo(
      this.client.program.programId
    );
    
    if (!programAccount) {
      throw new Error('Universal NFT program not found on network');
    }

    this.log('Environment validation passed');
  }

  private async deployCollections(): Promise<PublicKey[]> {
    this.log('Deploying collections...');
    
    const deployedCollections: PublicKey[] = [];

    for (const collectionConfig of this.config.collections) {
      try {
        this.log(`Deploying collection: ${collectionConfig.name}`);
        
        const tssBytes = this.hexToBytes(collectionConfig.tssAddress);
        
        const result = await this.client.initializeCollection(
          collectionConfig.name,
          collectionConfig.symbol,
          collectionConfig.uri,
          tssBytes
        );

        deployedCollections.push(result.collection);
        
        this.log(`Collection deployed: ${result.collection.toString()}`);
        
        // Wait for confirmation
        await this.client.waitForConfirmation(result.signature);
        
      } catch (error) {
        this.log(`Failed to deploy collection ${collectionConfig.name}: ${error.message}`);
        throw error;
      }
    }

    return deployedCollections;
  }

  private async setupMonitoring(collections: PublicKey[]): Promise<void> {
    this.log('Setting up monitoring...');
    
    const monitor = new ProductionMonitor(
      {
        collections: collections.map(c => c.toString()),
        alertThresholds: this.config.monitoring.alertThresholds,
        webhookUrl: this.config.monitoring.webhookUrl,
      },
      this.client
    );

    await monitor.start();
    
    this.log('Monitoring setup completed');
  }

  private async validateDeployment(collections: PublicKey[]): Promise<void> {
    this.log('Validating deployment...');
    
    for (const collection of collections) {
      // Verify collection exists and is properly configured
      const collectionData = await this.client.getCollection(collection);
      
      if (!collectionData) {
        throw new Error(`Collection validation failed: ${collection.toString()}`);
      }

      // Test basic functionality
      try {
        const testMint = await this.client.mintNft(
          collection,
          "Deployment Test NFT",
          "TEST",
          "https://example.com/test.json"
        );

        await this.client.waitForConfirmation(testMint.signature);
        
        this.log(`Collection ${collection.toString()} validated successfully`);
        
      } catch (error) {
        throw new Error(`Collection functionality test failed: ${error.message}`);
      }
    }

    this.log('Deployment validation completed');
  }

  private async generateDeploymentReport(collections: PublicKey[]): Promise<void> {
    const report = {
      timestamp: new Date().toISOString(),
      network: this.config.network,
      deployer: this.client.provider.wallet.publicKey.toString(),
      collections: collections.map(c => c.toString()),
      deploymentLog: this.deploymentLog,
      status: 'success',
    };

    const reportPath = path.join(__dirname, `../deployment-reports/deployment-${Date.now()}.json`);
    
    // Ensure directory exists
    const dir = path.dirname(reportPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    
    console.log(`📄 Deployment report saved: ${reportPath}`);
  }

  private async rollback(): Promise<void> {
    this.log('Initiating rollback...');
    
    // Implement rollback logic
    // This might involve:
    // - Stopping monitoring
    // - Reverting configuration changes
    // - Notifying stakeholders
    
    this.log('Rollback completed');
  }

  private hexToBytes(hex: string): number[] {
    if (hex.startsWith('0x')) {
      hex = hex.slice(2);
    }
    
    const bytes = [];
    for (let i = 0; i < hex.length; i += 2) {
      bytes.push(parseInt(hex.substr(i, 2), 16));
    }
    
    return bytes;
  }

  private log(message: string): void {
    const logEntry = {
      timestamp: new Date().toISOString(),
      message,
    };
    
    this.deploymentLog.push(logEntry);
    console.log(`[${logEntry.timestamp}] ${message}`);
  }
}

// Usage
async function main() {
  const config: DeploymentConfig = {
    network: Network.MAINNET,
    collections: [
      {
        name: "Universal Art Collection",
        symbol: "UAC",
        uri: "https://api.example.com/collections/art",
        tssAddress: "0x1234567890123456789012345678901234567890",
      },
    ],
    monitoring: {
      enabled: true,
      webhookUrl: process.env.MONITORING_WEBHOOK_URL,
      alertThresholds: {
        errorRate: 0.05,
        latency: 5000,
        failedTransactions: 10,
      },
    },
  };

  const deployer = new ProductionDeployer(config);
  await deployer.deploy();
}

if (require.main === module) {
  main().catch(console.error);
}
```

### Backup and Disaster Recovery

```typescript
// scripts/backup-recovery.ts
import { Connection, PublicKey } from '@solana/web3.js';
import { UniversalNftClient, Network } from '../sdk/client';
import * as fs from 'fs';
import * as path from 'path';

interface BackupData {
  timestamp: string;
  network: string;
  collections: Array<{
    address: string;
    data: any;
    nfts: Array<{
      tokenId: string;
      originData: any;
    }>;
  }>;
}

export class BackupManager {
  private client: UniversalNftClient;
  private backupDir: string;

  constructor(client: UniversalNftClient, backupDir: string = './backups') {
    this.client = client;
    this.backupDir = backupDir;
    
    // Ensure backup directory exists
    if (!fs.existsSync(backupDir)) {
      fs.mkdirSync(backupDir, { recursive: true });
    }
  }

  async createBackup(collections: PublicKey[]): Promise<string> {
    console.log('🔄 Creating backup...');
    
    const backupData: BackupData = {
      timestamp: new Date().toISOString(),
      network: this.client.config.network,
      collections: [],
    };

    for (const collection of collections) {
      console.log(`Backing up collection: ${collection.toString()}`);
      
      try {
        // Get collection data
        const collectionData = await this.client.getCollection(collection);
        
        // Get all NFTs for this collection
        const nfts = await this.getAllNFTsForCollection(collection);
        
        backupData.collections.push({
          address: collection.toString(),
          data: collectionData,
          nfts,
        });
        
      } catch (error) {
        console.error(`Failed to backup collection ${collection.toString()}:`, error);
        throw error;
      }
    }

    // Save backup to file
    const filename = `backup-${Date.now()}.json`;
    const filepath = path.join(this.backupDir, filename);
    
    fs.writeFileSync(filepath, JSON.stringify(backupData, null, 2));
    
    console.log(`✅ Backup created: ${filepath}`);
    return filepath;
  }

  async restoreFromBackup(backupPath: string): Promise<void> {
    console.log(`🔄 Restoring from backup: ${backupPath}`);
    
    if (!fs.existsSync(backupPath)) {
      throw new Error(`Backup file not found: ${backupPath}`);
    }

    const backupData: BackupData = JSON.parse(fs.readFileSync(backupPath, 'utf8'));
    
    console.log(`Restoring backup from ${backupData.timestamp}`);
    console.log(`Network: ${backupData.network}`);
    console.log(`Collections: ${backupData.collections.length}`);

    for (const collectionBackup of backupData.collections) {
      await this.restoreCollection(collectionBackup);
    }

    console.log('✅ Restore completed');
  }

  private async getAllNFTsForCollection(collection: PublicKey): Promise<Array<{ tokenId: string; originData: any }>> {
    // This is a simplified implementation
    // In practice, you'd need to scan all NFT Origin accounts for this collection
    
    const nfts: Array<{ tokenId: string; originData: any }> = [];
    
    try {
      // Get all NFT Origin accounts (this is a simplified approach)
      const originAccounts = await this.client.program.account.nftOrigin.all();
      
      for (const account of originAccounts) {
        if (account.account.collection.equals(collection)) {
          nfts.push({
            tokenId: account.account.tokenId.toString(),
            originData: account.account,
          });
        }
      }
      
    } catch (error) {
      console.warn(`Failed to get NFTs for collection ${collection.toString()}:`, error);
    }

    return nfts;
  }

  private async restoreCollection(collectionBackup: any): Promise<void> {
    console.log(`Restoring collection: ${collectionBackup.address}`);
    
    try {
      // Check if collection already exists
      const existingCollection = await this.client.getCollection(
        new PublicKey(collectionBackup.address)
      );
      
      if (existingCollection) {
        console.log(`Collection ${collectionBackup.address} already exists, skipping...`);
        return;
      }
      
    } catch (error) {
      // Collection doesn't exist, proceed with restoration
    }

    // Restore collection
    const result = await this.client.initializeCollection(
      collectionBackup.data.name,
      collectionBackup.data.symbol,
      collectionBackup.data.uri,
      Array.from(collectionBackup.data.tssAddress)
    );

    console.log(`Collection restored: ${result.collection.toString()}`);

    // Note: NFT restoration would require more complex logic
    // as you can't directly restore NFT Origin accounts
    // You'd need to re-mint NFTs and recreate the origin data
  }

  async scheduleBackups(collections: PublicKey[], intervalHours: number = 24): Promise<void> {
    console.log(`📅 Scheduling backups every ${intervalHours} hours`);
    
    const intervalMs = intervalHours * 60 * 60 * 1000;
    
    setInterval(async () => {
      try {
        await this.createBackup(collections);
        await this.cleanupOldBackups();
      } catch (error) {
        console.error('Scheduled backup failed:', error);
      }
    }, intervalMs);
  }

  private async cleanupOldBackups(maxAge: number = 30): Promise<void> {
    const maxAgeMs = maxAge * 24 * 60 * 60 * 1000; // Convert days to milliseconds
    const now = Date.now();

    const files = fs.readdirSync(this.backupDir);
    
    for (const file of files) {
      if (file.startsWith('backup-') && file.endsWith('.json')) {
        const filepath = path.join(this.backupDir, file);
        const stats = fs.statSync(filepath);
        
        if (now - stats.mtime.getTime() > maxAgeMs) {
          fs.unlinkSync(filepath);
          console.log(`🗑️  Deleted old backup: ${file}`);
        }
      }
    }
  }

  async verifyBackup(backupPath: string): Promise<boolean> {
    try {
      const backupData: BackupData = JSON.parse(fs.readFileSync(backupPath, 'utf8'));
      
      // Verify backup structure
      if (!backupData.timestamp || !backupData.network || !backupData.collections) {
        return false;
      }

      // Verify each collection backup
      for (const collection of backupData.collections) {
        if (!collection.address || !collection.data) {
          return false;
        }
      }

      return true;
    } catch {
      return false;
    }
  }
}
```

This comprehensive integration guide provides developers with practical, runnable examples for integrating the Solana Universal NFT protocol into their applications. The examples cover frontend React components, backend Node.js services, complete cross-chain scenarios, testing patterns, and production deployment strategies.

Each code example is designed to be a starting point that developers can customize for their specific use cases while following best practices for security, performance, and maintainability.