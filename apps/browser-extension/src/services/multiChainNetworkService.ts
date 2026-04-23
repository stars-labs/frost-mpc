/**
 * Multi-Chain Network Service
 * 
 * Enhanced network service that supports multiple blockchain networks
 * organized by signature algorithm (secp256k1 vs ed25519).
 */

import type { Chain } from "@mpc-wallet/types/network";
import { 
  ChainInfo, 
  SignatureAlgorithm, 
  ChainCategory,
  ALL_CHAINS,
  DEFAULT_CHAINS,
  getChainById,
  getChainsByAlgorithm,
  getChainsByCategory,
  getTestnetForMainnet
} from '../config/chains';

type NetworkChangeCallback = (network: ChainInfo | undefined) => void;

export class MultiChainNetworkService {
  private static instance: MultiChainNetworkService;
  
  // Networks organized by algorithm
  private networksByAlgorithm: Record<SignatureAlgorithm, ChainInfo[]> = {
    secp256k1: [],
    ed25519: []
  };
  
  // Current network for each algorithm
  private currentNetworks: Record<SignatureAlgorithm, ChainInfo | undefined> = {
    secp256k1: undefined,
    ed25519: undefined
  };
  
  // Current algorithm focus
  private currentAlgorithm: SignatureAlgorithm = 'secp256k1';
  
  // Storage keys
  private readonly STORAGE_KEY = 'mpc_wallet_networks_v2';
  private readonly CURRENT_NETWORKS_KEY = 'mpc_wallet_current_networks_v2';
  private readonly CURRENT_ALGORITHM_KEY = 'mpc_wallet_current_algorithm';
  
  // Callbacks
  private changeCallbacks: NetworkChangeCallback[] = [];
  private initialized: boolean = false;

  private constructor() {
    this.initializeAsync();
  }

  public static getInstance(): MultiChainNetworkService {
    if (!MultiChainNetworkService.instance) {
      MultiChainNetworkService.instance = new MultiChainNetworkService();
    }
    return MultiChainNetworkService.instance;
  }

  private async initializeAsync(): Promise<void> {
    if (!this.initialized) {
      await this.loadNetworks();
      this.initialized = true;
    }
  }

  public async ensureInitialized(): Promise<void> {
    if (!this.initialized) {
      await this.initializeAsync();
    }
  }

  private async loadNetworks(): Promise<void> {
    try {
      if (typeof chrome !== 'undefined' && chrome.storage) {
        const result = await chrome.storage.local.get([
          this.STORAGE_KEY,
          this.CURRENT_NETWORKS_KEY,
          this.CURRENT_ALGORITHM_KEY
        ]);

        // Load networks
        const storedNetworks = result[this.STORAGE_KEY];
        if (storedNetworks && typeof storedNetworks === 'object') {
          this.networksByAlgorithm = {
            secp256k1: storedNetworks.secp256k1 || [],
            ed25519: storedNetworks.ed25519 || []
          };
        }

        // Load current algorithm
        this.currentAlgorithm = result[this.CURRENT_ALGORITHM_KEY] || 'secp256k1';

        // Load current networks
        const storedCurrentNetworks = result[this.CURRENT_NETWORKS_KEY];
        if (storedCurrentNetworks) {
          this.currentNetworks = storedCurrentNetworks;
        }

        // Initialize with default networks if empty
        if (this.networksByAlgorithm.secp256k1.length === 0) {
          this.networksByAlgorithm.secp256k1 = DEFAULT_CHAINS.secp256k1;
          await this.saveNetworks();
        }

        if (this.networksByAlgorithm.ed25519.length === 0) {
          this.networksByAlgorithm.ed25519 = DEFAULT_CHAINS.ed25519;
          await this.saveNetworks();
        }

        // Set default current networks if not set
        if (!this.currentNetworks.secp256k1) {
          const ethereum = this.networksByAlgorithm.secp256k1.find(c => c.id === 1);
          if (ethereum) {
            this.currentNetworks.secp256k1 = ethereum;
            await this.saveCurrentNetworks();
          }
        }

        if (!this.currentNetworks.ed25519) {
          const solana = this.networksByAlgorithm.ed25519.find(c => c.id === 101);
          if (solana) {
            this.currentNetworks.ed25519 = solana;
            await this.saveCurrentNetworks();
          }
        }
      }
    } catch (error) {
      console.error('Failed to load networks:', error);
      // Initialize with defaults on error
      this.networksByAlgorithm = {
        secp256k1: DEFAULT_CHAINS.secp256k1,
        ed25519: DEFAULT_CHAINS.ed25519
      };
    }
  }

  private async saveNetworks(): Promise<void> {
    try {
      if (typeof chrome !== 'undefined' && chrome.storage) {
        await chrome.storage.local.set({ 
          [this.STORAGE_KEY]: this.networksByAlgorithm 
        });
      }
    } catch (error) {
      console.error('Failed to save networks:', error);
    }
  }

  private async saveCurrentNetworks(): Promise<void> {
    try {
      if (typeof chrome !== 'undefined' && chrome.storage) {
        await chrome.storage.local.set({ 
          [this.CURRENT_NETWORKS_KEY]: this.currentNetworks 
        });
      }
    } catch (error) {
      console.error('Failed to save current networks:', error);
    }
  }

  private async saveCurrentAlgorithm(): Promise<void> {
    try {
      if (typeof chrome !== 'undefined' && chrome.storage) {
        await chrome.storage.local.set({ 
          [this.CURRENT_ALGORITHM_KEY]: this.currentAlgorithm 
        });
      }
    } catch (error) {
      console.error('Failed to save current algorithm:', error);
    }
  }

  // Get all networks for an algorithm
  public getNetworksByAlgorithm(algorithm: SignatureAlgorithm): ChainInfo[] {
    return this.networksByAlgorithm[algorithm] || [];
  }

  // Get all networks for a category
  public getNetworksByCategory(category: ChainCategory): ChainInfo[] {
    const allNetworks = [
      ...this.networksByAlgorithm.secp256k1,
      ...this.networksByAlgorithm.ed25519
    ];
    return allNetworks.filter(n => n.category === category);
  }

  // Get current network for an algorithm
  public getCurrentNetwork(algorithm?: SignatureAlgorithm): ChainInfo | undefined {
    const algo = algorithm || this.currentAlgorithm;
    return this.currentNetworks[algo];
  }

  // Get current algorithm
  public getCurrentAlgorithm(): SignatureAlgorithm {
    return this.currentAlgorithm;
  }

  // Set current algorithm
  public async setCurrentAlgorithm(algorithm: SignatureAlgorithm): Promise<void> {
    this.currentAlgorithm = algorithm;
    await this.saveCurrentAlgorithm();
    this.notifyNetworkChange(this.currentNetworks[algorithm]);
  }

  // Set current network for an algorithm
  public async setCurrentNetwork(chainId: number): Promise<void> {
    const network = getChainById(chainId);
    if (!network) {
      throw new Error(`Network with chain ID ${chainId} not found`);
    }

    // Ensure network is in our list
    const existingNetwork = this.networksByAlgorithm[network.algorithm].find(
      n => n.id === chainId
    );
    
    if (!existingNetwork) {
      // Add network if not exists
      this.networksByAlgorithm[network.algorithm].push(network);
      await this.saveNetworks();
    }

    this.currentNetworks[network.algorithm] = network;
    this.currentAlgorithm = network.algorithm;
    
    await this.saveCurrentNetworks();
    await this.saveCurrentAlgorithm();
    
    this.notifyNetworkChange(network);
  }

  // Add a custom network
  public async addCustomNetwork(network: ChainInfo): Promise<void> {
    // Check if network already exists
    const exists = this.networksByAlgorithm[network.algorithm].some(
      n => n.id === network.id
    );
    
    if (exists) {
      throw new Error(`Network with chain ID ${network.id} already exists`);
    }

    this.networksByAlgorithm[network.algorithm].push(network);
    await this.saveNetworks();
  }

  // Remove a custom network
  public async removeCustomNetwork(chainId: number): Promise<void> {
    // Find the network in our managed lists
    let foundNetwork: ChainInfo | undefined;
    let algorithm: SignatureAlgorithm | undefined;
    
    for (const [alg, networks] of Object.entries(this.networksByAlgorithm)) {
      const network = networks.find(n => n.id === chainId);
      if (network) {
        foundNetwork = network;
        algorithm = alg as SignatureAlgorithm;
        break;
      }
    }
    
    if (!foundNetwork || !algorithm) {
      throw new Error(`Network with chain ID ${chainId} not found`);
    }

    // Check if it's a predefined network (from ALL_CHAINS)
    const isPredefined = ALL_CHAINS.some(c => c.id === chainId);
    if (isPredefined) {
      throw new Error('Cannot remove default networks');
    }

    // Remove from list
    this.networksByAlgorithm[algorithm] = 
      this.networksByAlgorithm[algorithm].filter(n => n.id !== chainId);

    // If it was the current network, switch to a default
    if (this.currentNetworks[algorithm]?.id === chainId) {
      const defaultNetwork = DEFAULT_CHAINS[algorithm][0];
      if (defaultNetwork) {
        await this.setCurrentNetwork(defaultNetwork.id);
      }
    }

    await this.saveNetworks();
  }

  // Toggle between mainnet and testnet
  public async toggleTestnet(): Promise<void> {
    const currentNetwork = this.getCurrentNetwork();
    if (!currentNetwork) return;

    let targetNetwork: ChainInfo | undefined;

    if (currentNetwork.testnet) {
      // Switch to mainnet
      targetNetwork = getChainById(currentNetwork.mainnetId || 0);
    } else {
      // Switch to testnet
      targetNetwork = getTestnetForMainnet(currentNetwork.id);

      // If testnet not in managed networks, add it. Alias to a
      // local const so the .some() closure sees the narrowed
      // (non-undefined) type — TS flow analysis doesn't narrow
      // through closure boundaries on the outer variable.
      if (targetNetwork) {
        const testnet = targetNetwork;
        if (!this.networksByAlgorithm[testnet.algorithm].some(n => n.id === testnet.id)) {
          this.networksByAlgorithm[testnet.algorithm].push(testnet);
          await this.saveNetworks();
        }
      }
    }

    if (targetNetwork) {
      await this.setCurrentNetwork(targetNetwork.id);
    }
  }

  // Get all networks grouped by category
  public getAllNetworksGrouped(): Record<ChainCategory, ChainInfo[]> {
    const allNetworks = [
      ...this.networksByAlgorithm.secp256k1,
      ...this.networksByAlgorithm.ed25519
    ];

    const grouped: Record<ChainCategory, ChainInfo[]> = {
      bitcoin: [],
      ethereum: [],
      evm: [],
      solana: [],
      aptos: [],
      sui: []
    };

    allNetworks.forEach(network => {
      grouped[network.category].push(network);
    });

    return grouped;
  }

  // Network change callbacks
  public onNetworkChange(callback: NetworkChangeCallback): void {
    this.changeCallbacks.push(callback);
  }

  public removeNetworkChangeListener(callback: NetworkChangeCallback): void {
    this.changeCallbacks = this.changeCallbacks.filter(cb => cb !== callback);
  }

  private notifyNetworkChange(network: ChainInfo | undefined): void {
    this.changeCallbacks.forEach(callback => callback(network));
  }

  // Legacy compatibility methods
  public getNetworks(blockchain?: 'ethereum' | 'solana'): Chain[] | Record<string, Chain[]> {
    if (blockchain === 'ethereum') {
      return this.networksByAlgorithm.secp256k1.filter(
        n => n.category === 'ethereum' || n.category === 'evm'
      );
    } else if (blockchain === 'solana') {
      return this.networksByAlgorithm.ed25519.filter(
        n => n.category === 'solana'
      );
    }

    return {
      ethereum: this.networksByAlgorithm.secp256k1.filter(
        n => n.category === 'ethereum' || n.category === 'evm'
      ),
      solana: this.networksByAlgorithm.ed25519.filter(
        n => n.category === 'solana'
      )
    };
  }

  public getCurrentBlockchain(): 'ethereum' | 'solana' {
    const currentNetwork = this.getCurrentNetwork();
    if (!currentNetwork) return 'ethereum';

    if (currentNetwork.category === 'solana') return 'solana';
    return 'ethereum'; // Default for all secp256k1 chains
  }

  public async setCurrentBlockchain(blockchain: 'ethereum' | 'solana'): Promise<void> {
    const targetAlgorithm = blockchain === 'solana' ? 'ed25519' : 'secp256k1';
    await this.setCurrentAlgorithm(targetAlgorithm);
  }
}