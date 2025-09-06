import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  ConnectionProvider,
  WalletProvider,
  useConnection,
  useWallet,
  useAnchorWallet
} from '@solana/wallet-adapter-react';
import {
  WalletModalProvider,
  WalletMultiButton,
  WalletDisconnectButton
} from '@solana/wallet-adapter-react-ui';
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  TorusWalletAdapter,
  LedgerWalletAdapter,
  SolletWalletAdapter,
  SolletExtensionWalletAdapter,
  MathWalletAdapter,
  Coin98WalletAdapter,
  SlopeWalletAdapter,
  BackpackWalletAdapter
} from '@solana/wallet-adapter-wallets';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { clusterApiUrl, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { 
  UniversalNftClient, 
  Network, 
  CollectionData, 
  NftOriginData, 
  MintResult, 
  TransferResult,
  UniversalNftUtils,
  NftMintedEvent,
  CrossChainTransferEvent,
  CrossChainReceiveEvent,
  EventSubscription
} from '../sdk/client';

// CSS imports (would be in separate files in real implementation)
import '@solana/wallet-adapter-react-ui/styles.css';

// Types for UI components
interface NFTCardProps {
  mint: PublicKey;
  tokenId: BN;
  name: string;
  symbol: string;
  uri: string;
  isOwned: boolean;
  onTransfer?: (mint: PublicKey) => void;
  onView?: (mint: PublicKey) => void;
}

interface CollectionCardProps {
  collection: PublicKey;
  data: CollectionData;
  onSelect: (collection: PublicKey) => void;
  isSelected: boolean;
}

interface TransactionStatusProps {
  signature?: string;
  status: 'pending' | 'success' | 'error' | 'idle';
  message?: string;
  onClose: () => void;
}

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

// Error Boundary Component
class ErrorBoundary extends React.Component<
  React.PropsWithChildren<{}>,
  ErrorBoundaryState
> {
  constructor(props: React.PropsWithChildren<{}>) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Universal NFT Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
          <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-6">
            <div className="flex items-center justify-center w-12 h-12 mx-auto bg-red-100 rounded-full">
              <svg className="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <div className="mt-4 text-center">
              <h3 className="text-lg font-medium text-gray-900">Something went wrong</h3>
              <p className="mt-2 text-sm text-gray-500">
                {this.state.error?.message || 'An unexpected error occurred'}
              </p>
              <button
                onClick={() => window.location.reload()}
                className="mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                Reload Page
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

// Loading Spinner Component
const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ size = 'md', className = '' }) => {
  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-8 h-8',
    lg: 'w-12 h-12'
  };

  return (
    <div className={`animate-spin rounded-full border-2 border-gray-300 border-t-blue-600 ${sizeClasses[size]} ${className}`} />
  );
};

// Transaction Status Component
const TransactionStatus: React.FC<TransactionStatusProps> = ({ signature, status, message, onClose }) => {
  if (status === 'idle') return null;

  const statusConfig = {
    pending: {
      icon: <LoadingSpinner size="sm" />,
      bgColor: 'bg-yellow-50',
      borderColor: 'border-yellow-200',
      textColor: 'text-yellow-800',
      title: 'Transaction Pending'
    },
    success: {
      icon: (
        <svg className="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7" />
        </svg>
      ),
      bgColor: 'bg-green-50',
      borderColor: 'border-green-200',
      textColor: 'text-green-800',
      title: 'Transaction Successful'
    },
    error: {
      icon: (
        <svg className="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      ),
      bgColor: 'bg-red-50',
      borderColor: 'border-red-200',
      textColor: 'text-red-800',
      title: 'Transaction Failed'
    }
  };

  const config = statusConfig[status];

  return (
    <div className={`fixed top-4 right-4 max-w-sm w-full ${config.bgColor} border ${config.borderColor} rounded-lg shadow-lg p-4 z-50`}>
      <div className="flex items-start">
        <div className="flex-shrink-0">
          {config.icon}
        </div>
        <div className="ml-3 w-0 flex-1">
          <p className={`text-sm font-medium ${config.textColor}`}>
            {config.title}
          </p>
          {message && (
            <p className={`mt-1 text-sm ${config.textColor} opacity-75`}>
              {message}
            </p>
          )}
          {signature && (
            <a
              href={`https://explorer.solana.com/tx/${signature}?cluster=devnet`}
              target="_blank"
              rel="noopener noreferrer"
              className={`mt-1 text-sm ${config.textColor} underline hover:no-underline`}
            >
              View on Explorer
            </a>
          )}
        </div>
        <div className="ml-4 flex-shrink-0 flex">
          <button
            onClick={onClose}
            className={`inline-flex ${config.textColor} hover:opacity-75 focus:outline-none`}
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
};

// NFT Card Component
const NFTCard: React.FC<NFTCardProps> = ({ mint, tokenId, name, symbol, uri, isOwned, onTransfer, onView }) => {
  const [imageUrl, setImageUrl] = useState<string>('');
  const [imageLoading, setImageLoading] = useState(true);
  const [imageError, setImageError] = useState(false);

  useEffect(() => {
    const loadMetadata = async () => {
      try {
        const response = await fetch(uri);
        const metadata = await response.json();
        setImageUrl(metadata.image || '');
      } catch (error) {
        console.error('Failed to load metadata:', error);
        setImageError(true);
      } finally {
        setImageLoading(false);
      }
    };

    if (uri) {
      loadMetadata();
    } else {
      setImageLoading(false);
    }
  }, [uri]);

  return (
    <div className="bg-white rounded-lg shadow-md overflow-hidden hover:shadow-lg transition-shadow duration-200">
      <div className="aspect-square bg-gray-200 relative">
        {imageLoading ? (
          <div className="absolute inset-0 flex items-center justify-center">
            <LoadingSpinner />
          </div>
        ) : imageError || !imageUrl ? (
          <div className="absolute inset-0 flex items-center justify-center bg-gray-100">
            <svg className="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
        ) : (
          <img
            src={imageUrl}
            alt={name}
            className="w-full h-full object-cover"
            onError={() => setImageError(true)}
          />
        )}
        {isOwned && (
          <div className="absolute top-2 right-2 bg-green-500 text-white text-xs px-2 py-1 rounded-full">
            Owned
          </div>
        )}
      </div>
      
      <div className="p-4">
        <h3 className="font-semibold text-gray-900 truncate">{name}</h3>
        <p className="text-sm text-gray-500 truncate">{symbol}</p>
        <p className="text-xs text-gray-400 mt-1">
          Token ID: {tokenId.toString()}
        </p>
        <p className="text-xs text-gray-400 truncate">
          Mint: {mint.toString()}
        </p>
        
        <div className="mt-4 flex gap-2">
          {onView && (
            <button
              onClick={() => onView(mint)}
              className="flex-1 px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
            >
              View
            </button>
          )}
          {isOwned && onTransfer && (
            <button
              onClick={() => onTransfer(mint)}
              className="flex-1 px-3 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Transfer
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

// Collection Card Component
const CollectionCard: React.FC<CollectionCardProps> = ({ collection, data, onSelect, isSelected }) => {
  return (
    <div
      onClick={() => onSelect(collection)}
      className={`bg-white rounded-lg shadow-md p-6 cursor-pointer transition-all duration-200 ${
        isSelected ? 'ring-2 ring-blue-500 shadow-lg' : 'hover:shadow-lg'
      }`}
    >
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-900">{data.name}</h3>
          <p className="text-sm text-gray-500">{data.symbol}</p>
          <p className="text-xs text-gray-400 mt-1 truncate">
            {collection.toString()}
          </p>
        </div>
        <div className="text-right">
          <p className="text-sm font-medium text-gray-900">
            {data.totalSupply.toString()} NFTs
          </p>
          <p className="text-xs text-gray-500">Total Supply</p>
        </div>
      </div>
      
      <div className="mt-4 pt-4 border-t border-gray-200">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <span>Authority: {data.authority.toString().slice(0, 8)}...</span>
          {isSelected && (
            <span className="text-blue-600 font-medium">Selected</span>
          )}
        </div>
      </div>
    </div>
  );
};

// Main App Component
const UniversalNFTApp: React.FC = () => {
  const { connection } = useConnection();
  const wallet = useWallet();
  const anchorWallet = useAnchorWallet();
  
  // State management
  const [client, setClient] = useState<UniversalNftClient | null>(null);
  const [collections, setCollections] = useState<Array<{ pubkey: PublicKey; account: CollectionData }>>([]);
  const [selectedCollection, setSelectedCollection] = useState<PublicKey | null>(null);
  const [nfts, setNfts] = useState<Array<{ mint: PublicKey; tokenId: BN; name: string; symbol: string; uri: string; isOwned: boolean }>>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'collections' | 'mint' | 'transfer' | 'events'>('collections');
  
  // Transaction state
  const [transactionStatus, setTransactionStatus] = useState<{
    signature?: string;
    status: 'pending' | 'success' | 'error' | 'idle';
    message?: string;
  }>({ status: 'idle' });
  
  // Form state
  const [mintForm, setMintForm] = useState({
    name: '',
    symbol: '',
    uri: '',
    recipient: ''
  });
  
  const [transferForm, setTransferForm] = useState({
    selectedNft: '',
    destinationChain: '',
    recipient: ''
  });
  
  const [collectionForm, setCollectionForm] = useState({
    name: '',
    symbol: '',
    uri: '',
    tssAddress: ''
  });
  
  // Event subscriptions
  const [eventSubscriptions, setEventSubscriptions] = useState<EventSubscription[]>([]);
  const [events, setEvents] = useState<Array<{
    type: 'mint' | 'transfer' | 'receive';
    data: any;
    timestamp: number;
  }>>([]);

  // Initialize client when wallet connects
  useEffect(() => {
    const initializeClient = async () => {
      if (!anchorWallet || !connection) {
        setClient(null);
        return;
      }

      try {
        setLoading(true);
        const newClient = await UniversalNftClient.create(
          {
            network: Network.DEVNET,
            commitment: 'confirmed'
          },
          {
            adapter: wallet.adapter
          }
        );
        setClient(newClient);
        setError(null);
      } catch (err) {
        console.error('Failed to initialize client:', err);
        setError('Failed to initialize Universal NFT client');
      } finally {
        setLoading(false);
      }
    };

    initializeClient();
  }, [anchorWallet, connection, wallet.adapter]);

  // Load collections when client is ready
  useEffect(() => {
    const loadCollections = async () => {
      if (!client || !wallet.publicKey) return;

      try {
        setLoading(true);
        const userCollections = await client.getCollectionsByAuthority(wallet.publicKey);
        setCollections(userCollections);
      } catch (err) {
        console.error('Failed to load collections:', err);
        setError('Failed to load collections');
      } finally {
        setLoading(false);
      }
    };

    loadCollections();
  }, [client, wallet.publicKey]);

  // Load NFTs when collection is selected
  useEffect(() => {
    const loadNFTs = async () => {
      if (!client || !selectedCollection) return;

      try {
        setLoading(true);
        const collectionNfts = await client.getNftsByCollection(selectedCollection);
        
        // Transform data for UI
        const nftData = await Promise.all(
          collectionNfts.map(async ({ pubkey, account }) => {
            // This is simplified - in reality you'd need to derive the mint from the NFT Origin
            // and fetch metadata from the token metadata account
            return {
              mint: pubkey, // This would be the actual mint address
              tokenId: account.tokenId,
              name: `NFT #${account.tokenId.toString()}`,
              symbol: 'UNFT',
              uri: 'https://example.com/metadata.json',
              isOwned: true // This would be determined by checking token account ownership
            };
          })
        );
        
        setNfts(nftData);
      } catch (err) {
        console.error('Failed to load NFTs:', err);
        setError('Failed to load NFTs');
      } finally {
        setLoading(false);
      }
    };

    loadNFTs();
  }, [client, selectedCollection]);

  // Setup event listeners
  useEffect(() => {
    const setupEventListeners = async () => {
      if (!client || !selectedCollection) return;

      // Clear existing subscriptions
      eventSubscriptions.forEach(sub => sub.unsubscribe());
      setEventSubscriptions([]);

      try {
        const subscriptions: EventSubscription[] = [];

        // Subscribe to mint events
        const mintSub = await client.onNftMinted((event: NftMintedEvent) => {
          setEvents(prev => [...prev, {
            type: 'mint',
            data: event,
            timestamp: Date.now()
          }]);
        }, selectedCollection);
        subscriptions.push(mintSub);

        // Subscribe to transfer events
        const transferSub = await client.onCrossChainTransfer((event: CrossChainTransferEvent) => {
          setEvents(prev => [...prev, {
            type: 'transfer',
            data: event,
            timestamp: Date.now()
          }]);
        }, selectedCollection);
        subscriptions.push(transferSub);

        // Subscribe to receive events
        const receiveSub = await client.onCrossChainReceive((event: CrossChainReceiveEvent) => {
          setEvents(prev => [...prev, {
            type: 'receive',
            data: event,
            timestamp: Date.now()
          }]);
        }, selectedCollection);
        subscriptions.push(receiveSub);

        setEventSubscriptions(subscriptions);
      } catch (err) {
        console.error('Failed to setup event listeners:', err);
      }
    };

    setupEventListeners();

    return () => {
      eventSubscriptions.forEach(sub => sub.unsubscribe());
    };
  }, [client, selectedCollection]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (client) {
        client.dispose();
      }
    };
  }, [client]);

  // Handler functions
  const handleCreateCollection = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!client) return;

    try {
      setTransactionStatus({ status: 'pending', message: 'Creating collection...' });
      
      const tssBytes = UniversalNftUtils.ethAddressToBytes(collectionForm.tssAddress);
      const result = await client.initializeCollection(
        collectionForm.name,
        collectionForm.symbol,
        collectionForm.uri,
        tssBytes
      );

      setTransactionStatus({
        status: 'success',
        signature: result.signature,
        message: 'Collection created successfully!'
      });

      // Reset form
      setCollectionForm({ name: '', symbol: '', uri: '', tssAddress: '' });
      
      // Reload collections
      if (wallet.publicKey) {
        const userCollections = await client.getCollectionsByAuthority(wallet.publicKey);
        setCollections(userCollections);
      }
    } catch (err) {
      console.error('Failed to create collection:', err);
      setTransactionStatus({
        status: 'error',
        message: err instanceof Error ? err.message : 'Failed to create collection'
      });
    }
  };

  const handleMintNFT = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!client || !selectedCollection) return;

    try {
      setTransactionStatus({ status: 'pending', message: 'Minting NFT...' });
      
      const recipient = mintForm.recipient ? new PublicKey(mintForm.recipient) : undefined;
      const result = await client.mintNft(
        selectedCollection,
        mintForm.name,
        mintForm.symbol,
        mintForm.uri,
        recipient
      );

      setTransactionStatus({
        status: 'success',
        signature: result.signature,
        message: 'NFT minted successfully!'
      });

      // Reset form
      setMintForm({ name: '', symbol: '', uri: '', recipient: '' });
      
      // Reload NFTs
      const collectionNfts = await client.getNftsByCollection(selectedCollection);
      // Update NFTs list (simplified)
      
    } catch (err) {
      console.error('Failed to mint NFT:', err);
      setTransactionStatus({
        status: 'error',
        message: err instanceof Error ? err.message : 'Failed to mint NFT'
      });
    }
  };

  const handleTransferNFT = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!client || !selectedCollection) return;

    try {
      setTransactionStatus({ status: 'pending', message: 'Transferring NFT...' });
      
      const nftMint = new PublicKey(transferForm.selectedNft);
      const destinationChain = parseInt(transferForm.destinationChain);
      const recipient = UniversalNftUtils.ethAddressToBytes(transferForm.recipient);
      
      const result = await client.transferCrossChain(
        selectedCollection,
        nftMint,
        destinationChain,
        recipient
      );

      setTransactionStatus({
        status: 'success',
        signature: result.signature,
        message: 'NFT transferred successfully!'
      });

      // Reset form
      setTransferForm({ selectedNft: '', destinationChain: '', recipient: '' });
      
    } catch (err) {
      console.error('Failed to transfer NFT:', err);
      setTransactionStatus({
        status: 'error',
        message: err instanceof Error ? err.message : 'Failed to transfer NFT'
      });
    }
  };

  const handleNFTView = (mint: PublicKey) => {
    // Open NFT detail modal or navigate to detail page
    console.log('View NFT:', mint.toString());
  };

  const handleNFTTransfer = (mint: PublicKey) => {
    setTransferForm(prev => ({ ...prev, selectedNft: mint.toString() }));
    setActiveTab('transfer');
  };

  // Responsive design helpers
  const isMobile = typeof window !== 'undefined' && window.innerWidth < 768;

  // Render functions
  const renderCollections = () => (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <h2 className="text-2xl font-bold text-gray-900">Collections</h2>
        <button
          onClick={() => setActiveTab('collections')}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          Create Collection
        </button>
      </div>

      {/* Create Collection Form */}
      <div className="bg-white shadow rounded-lg p-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">Create New Collection</h3>
        <form onSubmit={handleCreateCollection} className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label htmlFor="collection-name" className="block text-sm font-medium text-gray-700">
                Name
              </label>
              <input
                type="text"
                id="collection-name"
                value={collectionForm.name}
                onChange={(e) => setCollectionForm(prev => ({ ...prev, name: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="My NFT Collection"
                required
                maxLength={32}
              />
            </div>
            <div>
              <label htmlFor="collection-symbol" className="block text-sm font-medium text-gray-700">
                Symbol
              </label>
              <input
                type="text"
                id="collection-symbol"
                value={collectionForm.symbol}
                onChange={(e) => setCollectionForm(prev => ({ ...prev, symbol: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="MNC"
                required
                maxLength={10}
              />
            </div>
          </div>
          <div>
            <label htmlFor="collection-uri" className="block text-sm font-medium text-gray-700">
              Metadata URI
            </label>
            <input
              type="url"
              id="collection-uri"
              value={collectionForm.uri}
              onChange={(e) => setCollectionForm(prev => ({ ...prev, uri: e.target.value }))}
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              placeholder="https://example.com/collection-metadata.json"
              required
            />
          </div>
          <div>
            <label htmlFor="tss-address" className="block text-sm font-medium text-gray-700">
              TSS Address (Ethereum format)
            </label>
            <input
              type="text"
              id="tss-address"
              value={collectionForm.tssAddress}
              onChange={(e) => setCollectionForm(prev => ({ ...prev, tssAddress: e.target.value }))}
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              placeholder="0x1234567890123456789012345678901234567890"
              required
              pattern="^0x[a-fA-F0-9]{40}$"
            />
          </div>
          <button
            type="submit"
            disabled={loading || !client}
            className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? <LoadingSpinner size="sm" /> : 'Create Collection'}
          </button>
        </form>
      </div>

      {/* Collections List */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {collections.map(({ pubkey, account }) => (
          <CollectionCard
            key={pubkey.toString()}
            collection={pubkey}
            data={account}
            onSelect={setSelectedCollection}
            isSelected={selectedCollection?.equals(pubkey) || false}
          />
        ))}
      </div>

      {collections.length === 0 && !loading && (
        <div className="text-center py-12">
          <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <h3 className="mt-2 text-sm font-medium text-gray-900">No collections</h3>
          <p className="mt-1 text-sm text-gray-500">Get started by creating your first collection.</p>
        </div>
      )}
    </div>
  );

  const renderMint = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Mint NFT</h2>
      
      {!selectedCollection ? (
        <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800">Select a collection first</h3>
              <p className="mt-1 text-sm text-yellow-700">
                You need to select a collection before you can mint NFTs.
              </p>
            </div>
          </div>
        </div>
      ) : (
        <div className="bg-white shadow rounded-lg p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">
            Mint NFT in {collections.find(c => c.pubkey.equals(selectedCollection))?.account.name}
          </h3>
          <form onSubmit={handleMintNFT} className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label htmlFor="nft-name" className="block text-sm font-medium text-gray-700">
                  Name
                </label>
                <input
                  type="text"
                  id="nft-name"
                  value={mintForm.name}
                  onChange={(e) => setMintForm(prev => ({ ...prev, name: e.target.value }))}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="My Awesome NFT"
                  required
                  maxLength={32}
                />
              </div>
              <div>
                <label htmlFor="nft-symbol" className="block text-sm font-medium text-gray-700">
                  Symbol
                </label>
                <input
                  type="text"
                  id="nft-symbol"
                  value={mintForm.symbol}
                  onChange={(e) => setMintForm(prev => ({ ...prev, symbol: e.target.value }))}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="MAN"
                  required
                  maxLength={10}
                />
              </div>
            </div>
            <div>
              <label htmlFor="nft-uri" className="block text-sm font-medium text-gray-700">
                Metadata URI
              </label>
              <input
                type="url"
                id="nft-uri"
                value={mintForm.uri}
                onChange={(e) => setMintForm(prev => ({ ...prev, uri: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="https://example.com/nft-metadata.json"
                required
              />
            </div>
            <div>
              <label htmlFor="nft-recipient" className="block text-sm font-medium text-gray-700">
                Recipient (optional)
              </label>
              <input
                type="text"
                id="nft-recipient"
                value={mintForm.recipient}
                onChange={(e) => setMintForm(prev => ({ ...prev, recipient: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="Leave empty to mint to yourself"
              />
            </div>
            <button
              type="submit"
              disabled={loading || !client}
              className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? <LoadingSpinner size="sm" /> : 'Mint NFT'}
            </button>
          </form>
        </div>
      )}

      {/* NFTs Grid */}
      {selectedCollection && (
        <div>
          <h3 className="text-lg font-medium text-gray-900 mb-4">Your NFTs</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
            {nfts.map((nft) => (
              <NFTCard
                key={nft.mint.toString()}
                mint={nft.mint}
                tokenId={nft.tokenId}
                name={nft.name}
                symbol={nft.symbol}
                uri={nft.uri}
                isOwned={nft.isOwned}
                onView={handleNFTView}
                onTransfer={handleNFTTransfer}
              />
            ))}
          </div>
          
          {nfts.length === 0 && !loading && (
            <div className="text-center py-12">
              <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              <h3 className="mt-2 text-sm font-medium text-gray-900">No NFTs</h3>
              <p className="mt-1 text-sm text-gray-500">Mint your first NFT to get started.</p>
            </div>
          )}
        </div>
      )}
    </div>
  );

  const renderTransfer = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Cross-Chain Transfer</h2>
      
      {!selectedCollection ? (
        <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800">Select a collection first</h3>
              <p className="mt-1 text-sm text-yellow-700">
                You need to select a collection before you can transfer NFTs.
              </p>
            </div>
          </div>
        </div>
      ) : (
        <div className="bg-white shadow rounded-lg p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Transfer NFT Cross-Chain</h3>
          <form onSubmit={handleTransferNFT} className="space-y-4">
            <div>
              <label htmlFor="transfer-nft" className="block text-sm font-medium text-gray-700">
                Select NFT
              </label>
              <select
                id="transfer-nft"
                value={transferForm.selectedNft}
                onChange={(e) => setTransferForm(prev => ({ ...prev, selectedNft: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                required
              >
                <option value="">Choose an NFT to transfer</option>
                {nfts.filter(nft => nft.isOwned).map((nft) => (
                  <option key={nft.mint.toString()} value={nft.mint.toString()}>
                    {nft.name} (Token ID: {nft.tokenId.toString()})
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label htmlFor="destination-chain" className="block text-sm font-medium text-gray-700">
                Destination Chain
              </label>
              <select
                id="destination-chain"
                value={transferForm.destinationChain}
                onChange={(e) => setTransferForm(prev => ({ ...prev, destinationChain: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                required
              >
                <option value="">Select destination chain</option>
                <option value="1">Ethereum Mainnet</option>
                <option value="5">Ethereum Goerli</option>
                <option value="137">Polygon</option>
                <option value="56">BSC</option>
                <option value="43114">Avalanche</option>
              </select>
            </div>
            <div>
              <label htmlFor="transfer-recipient" className="block text-sm font-medium text-gray-700">
                Recipient Address (Ethereum format)
              </label>
              <input
                type="text"
                id="transfer-recipient"
                value={transferForm.recipient}
                onChange={(e) => setTransferForm(prev => ({ ...prev, recipient: e.target.value }))}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="0x1234567890123456789012345678901234567890"
                required
                pattern="^0x[a-fA-F0-9]{40}$"
              />
            </div>
            <button
              type="submit"
              disabled={loading || !client || nfts.filter(nft => nft.isOwned).length === 0}
              className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? <LoadingSpinner size="sm" /> : 'Transfer NFT'}
            </button>
          </form>
        </div>
      )}
    </div>
  );

  const renderEvents = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">Event Monitor</h2>
      
      <div className="bg-white shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">Real-time Events</h3>
          <p className="mt-1 text-sm text-gray-500">
            {selectedCollection ? 
              `Monitoring events for ${collections.find(c => c.pubkey.equals(selectedCollection))?.account.name}` :
              'Select a collection to monitor events'
            }
          </p>
        </div>
        
        <div className="divide-y divide-gray-200 max-h-96 overflow-y-auto">
          {events.length === 0 ? (
            <div className="px-6 py-12 text-center">
              <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15 17h5l-5 5v-5zM4 19h6v-2H4v2zM16 3H4v2h12V3zM4 7h12v2H4V7zM4 11h12v2H4v-2z" />
              </svg>
              <h3 className="mt-2 text-sm font-medium text-gray-900">No events yet</h3>
              <p className="mt-1 text-sm text-gray-500">
                Events will appear here as they happen.
              </p>
            </div>
          ) : (
            events.slice().reverse().map((event, index) => (
              <div key={index} className="px-6 py-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center">
                    <div className={`flex-shrink-0 w-2 h-2 rounded-full ${
                      event.type === 'mint' ? 'bg-green-400' :
                      event.type === 'transfer' ? 'bg-blue-400' :
                      'bg-purple-400'
                    }`} />
                    <div className="ml-3">
                      <p className="text-sm font-medium text-gray-900">
                        {event.type === 'mint' ? 'NFT Minted' :
                         event.type === 'transfer' ? 'Cross-Chain Transfer' :
                         'Cross-Chain Receive'}
                      </p>
                      <p className="text-sm text-gray-500">
                        {new Date(event.timestamp).toLocaleString()}
                      </p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-gray-400 font-mono">
                      {event.data.mint?.toString().slice(0, 8)}...
                    </p>
                  </div>
                </div>
                <div className="mt-2 text-xs text-gray-600">
                  <pre className="whitespace-pre-wrap">
                    {JSON.stringify(event.data, null, 2)}
                  </pre>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );

  // Main render
  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <div className="flex items-center">
              <h1 className="text-3xl font-bold text-gray-900">Universal NFT</h1>
              <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                Solana
              </span>
            </div>
            <div className="flex items-center space-x-4">
              {wallet.connected && (
                <div className="text-sm text-gray-500">
                  {wallet.publicKey?.toString().slice(0, 8)}...
                </div>
              )}
              <WalletMultiButton className="!bg-blue-600 hover:!bg-blue-700" />
              {wallet.connected && <WalletDisconnectButton />}
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {!wallet.connected ? (
          <div className="text-center py-12">
            <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <h3 className="mt-2 text-lg font-medium text-gray-900">Connect your wallet</h3>
            <p className="mt-1 text-sm text-gray-500">
              Connect your Solana wallet to start using Universal NFT.
            </p>
            <div className="mt-6">
              <WalletMultiButton className="!bg-blue-600 hover:!bg-blue-700" />
            </div>
          </div>
        ) : (
          <>
            {/* Navigation Tabs */}
            <div className="border-b border-gray-200 mb-8">
              <nav className="-mb-px flex space-x-8 overflow-x-auto">
                {[
                  { id: 'collections', name: 'Collections', icon: '📁' },
                  { id: 'mint', name: 'Mint NFT', icon: '🎨' },
                  { id: 'transfer', name: 'Transfer', icon: '🔄' },
                  { id: 'events', name: 'Events', icon: '📊' }
                ].map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id as any)}
                    className={`whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm ${
                      activeTab === tab.id
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                    }`}
                  >
                    <span className="mr-2">{tab.icon}</span>
                    {tab.name}
                  </button>
                ))}
              </nav>
            </div>

            {/* Error Display */}
            {error && (
              <div className="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">Error</h3>
                    <p className="mt-1 text-sm text-red-700">{error}</p>
                  </div>
                  <div className="ml-auto pl-3">
                    <button
                      onClick={() => setError(null)}
                      className="inline-flex text-red-400 hover:text-red-600"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Tab Content */}
            {loading && !client ? (
              <div className="flex justify-center py-12">
                <LoadingSpinner size="lg" />
              </div>
            ) : (
              <>
                {activeTab === 'collections' && renderCollections()}
                {activeTab === 'mint' && renderMint()}
                {activeTab === 'transfer' && renderTransfer()}
                {activeTab === 'events' && renderEvents()}
              </>
            )}
          </>
        )}
      </main>

      {/* Transaction Status */}
      <TransactionStatus
        signature={transactionStatus.signature}
        status={transactionStatus.status}
        message={transactionStatus.message}
        onClose={() => setTransactionStatus({ status: 'idle' })}
      />
    </div>
  );
};

// Wallet Configuration
const WalletConnectionProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const network = WalletAdapterNetwork.Devnet;
  const endpoint = useMemo(() => clusterApiUrl(network), [network]);
  
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new BackpackWalletAdapter(),
      new TorusWalletAdapter(),
      new LedgerWalletAdapter(),
      new SolletWalletAdapter({ network }),
      new SolletExtensionWalletAdapter({ network }),
      new MathWalletAdapter(),
      new Coin98WalletAdapter(),
      new SlopeWalletAdapter(),
    ],
    [network]
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          {children}
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
};

// Main App with Providers
const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <WalletConnectionProvider>
        <div className="App">
          <UniversalNFTApp />
        </div>
      </WalletConnectionProvider>
    </ErrorBoundary>
  );
};

export default App;

// Additional utility components and hooks for advanced usage

// Custom hook for responsive design
export const useResponsive = () => {
  const [windowSize, setWindowSize] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });

  useEffect(() => {
    const handleResize = () => {
      setWindowSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return {
    ...windowSize,
    isMobile: windowSize.width < 768,
    isTablet: windowSize.width >= 768 && windowSize.width < 1024,
    isDesktop: windowSize.width >= 1024,
  };
};

// Custom hook for Universal NFT operations
export const useUniversalNft = () => {
  const wallet = useWallet();
  const { connection } = useConnection();
  const [client, setClient] = useState<UniversalNftClient | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initClient = async () => {
      if (!wallet.adapter || !wallet.connected) {
        setClient(null);
        return;
      }

      try {
        setLoading(true);
        const newClient = await UniversalNftClient.create(
          { network: Network.DEVNET },
          { adapter: wallet.adapter }
        );
        setClient(newClient);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to initialize client');
      } finally {
        setLoading(false);
      }
    };

    initClient();
  }, [wallet.adapter, wallet.connected]);

  return { client, loading, error };
};

// Accessibility helpers
export const announceToScreenReader = (message: string) => {
  const announcement = document.createElement('div');
  announcement.setAttribute('aria-live', 'polite');
  announcement.setAttribute('aria-atomic', 'true');
  announcement.setAttribute('class', 'sr-only');
  announcement.textContent = message;
  
  document.body.appendChild(announcement);
  
  setTimeout(() => {
    document.body.removeChild(announcement);
  }, 1000);
};

// Performance monitoring
export const usePerformanceMonitor = () => {
  const [metrics, setMetrics] = useState({
    renderTime: 0,
    componentCount: 0,
    memoryUsage: 0
  });

  useEffect(() => {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      entries.forEach((entry) => {
        if (entry.entryType === 'measure') {
          setMetrics(prev => ({
            ...prev,
            renderTime: entry.duration
          }));
        }
      });
    });

    observer.observe({ entryTypes: ['measure'] });

    return () => observer.disconnect();
  }, []);

  return metrics;
};