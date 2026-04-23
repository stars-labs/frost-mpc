import { describe, it, expect } from 'bun:test';
import {
  ChainInfo,
  SignatureAlgorithm,
  ChainCategory,
  BITCOIN_CHAINS,
  ETHEREUM_CHAINS,
  SOLANA_CHAINS,
  APTOS_CHAINS,
  SUI_CHAINS,
  CHAINS_BY_ALGORITHM,
  CHAINS_BY_CATEGORY,
  ALL_CHAINS,
  DEFAULT_CHAINS,
  getChainById,
  getChainsByAlgorithm,
  getChainsByCategory,
  getMainnetChains,
  getTestnetChains,
  getTestnetForMainnet
} from '../../src/config/chains';

describe('Chain Configuration', () => {
  describe('Chain Structure Validation', () => {
    it('should have valid structure for all chains', () => {
      ALL_CHAINS.forEach(chain => {
        // Required fields
        expect(chain.id).toBeDefined();
        expect(typeof chain.id).toBe('number');
        expect(chain.name).toBeDefined();
        expect(typeof chain.name).toBe('string');
        expect(chain.network).toBeDefined();
        expect(typeof chain.network).toBe('string');
        expect(chain.algorithm).toBeDefined();
        expect(['secp256k1', 'ed25519']).toContain(chain.algorithm);
        expect(chain.category).toBeDefined();
        expect(['bitcoin', 'ethereum', 'evm', 'solana', 'aptos', 'sui']).toContain(chain.category);
        
        // Native currency
        expect(chain.nativeCurrency).toBeDefined();
        expect(chain.nativeCurrency!.name).toBeDefined();
        expect(chain.nativeCurrency!.symbol).toBeDefined();
        expect(chain.nativeCurrency!.decimals).toBeDefined();
        expect(typeof chain.nativeCurrency!.decimals).toBe('number');
        
        // RPC URLs
        expect(chain.rpcUrls).toBeDefined();
        expect(chain.rpcUrls!.default).toBeDefined();
        expect(chain.rpcUrls!.default.http).toBeDefined();
        expect(Array.isArray(chain.rpcUrls!.default.http)).toBe(true);
        expect(chain.rpcUrls!.default.http.length).toBeGreaterThan(0);
        
        // Block explorers
        expect(chain.blockExplorers).toBeDefined();
        expect(chain.blockExplorers!.default).toBeDefined();
        expect(chain.blockExplorers!.default.name).toBeDefined();
        expect(chain.blockExplorers!.default.url).toBeDefined();
        
        // Testnet validation
        if (chain.testnet) {
          expect(chain.mainnetId).toBeDefined();
          expect(typeof chain.mainnetId).toBe('number');
        }
      });
    });

    it('should have unique chain IDs', () => {
      const chainIds = ALL_CHAINS.map(chain => chain.id);
      const uniqueIds = new Set(chainIds);
      expect(chainIds.length).toBe(uniqueIds.size);
    });

    it('should have proper derivation paths', () => {
      ALL_CHAINS.forEach(chain => {
        if (chain.derivationPath) {
          // More flexible regex to handle various derivation path formats
          // Matches paths like m/44'/60'/0'/0/0 or m/44'/501'/0'/0'
          expect(chain.derivationPath).toMatch(/^m(\/\d+'?)+$/);
        }
      });
    });
  });

  describe('Algorithm-based Grouping', () => {
    it('should correctly group chains by algorithm', () => {
      const secp256k1Chains = CHAINS_BY_ALGORITHM.secp256k1;
      const ed25519Chains = CHAINS_BY_ALGORITHM.ed25519;
      
      // Check secp256k1 chains
      expect(secp256k1Chains).toContain(BITCOIN_CHAINS[0]);
      expect(secp256k1Chains).toContain(ETHEREUM_CHAINS[0]);
      secp256k1Chains.forEach(chain => {
        expect(chain.algorithm).toBe('secp256k1');
      });
      
      // Check ed25519 chains
      expect(ed25519Chains).toContain(SOLANA_CHAINS[0]);
      expect(ed25519Chains).toContain(APTOS_CHAINS[0]);
      expect(ed25519Chains).toContain(SUI_CHAINS[0]);
      ed25519Chains.forEach(chain => {
        expect(chain.algorithm).toBe('ed25519');
      });
    });

    it('should have no overlap between algorithm groups', () => {
      const secp256k1Ids = CHAINS_BY_ALGORITHM.secp256k1.map(c => c.id);
      const ed25519Ids = CHAINS_BY_ALGORITHM.ed25519.map(c => c.id);
      
      const intersection = secp256k1Ids.filter(id => ed25519Ids.includes(id));
      expect(intersection.length).toBe(0);
    });
  });

  describe('Category-based Grouping', () => {
    it('should correctly group chains by category', () => {
      expect(CHAINS_BY_CATEGORY.bitcoin.every(c => c.category === 'bitcoin')).toBe(true);
      expect(CHAINS_BY_CATEGORY.ethereum.every(c => c.category === 'ethereum')).toBe(true);
      expect(CHAINS_BY_CATEGORY.evm.every(c => c.category === 'evm')).toBe(true);
      expect(CHAINS_BY_CATEGORY.solana.every(c => c.category === 'solana')).toBe(true);
      expect(CHAINS_BY_CATEGORY.aptos.every(c => c.category === 'aptos')).toBe(true);
      expect(CHAINS_BY_CATEGORY.sui.every(c => c.category === 'sui')).toBe(true);
    });

    it('should have correct number of chains per category', () => {
      expect(CHAINS_BY_CATEGORY.bitcoin.length).toBe(2); // mainnet + testnet
      expect(CHAINS_BY_CATEGORY.ethereum.length).toBe(2); // mainnet + sepolia
      expect(CHAINS_BY_CATEGORY.evm.length).toBeGreaterThan(5); // multiple EVM chains
      expect(CHAINS_BY_CATEGORY.solana.length).toBe(3); // mainnet + devnet + testnet
      expect(CHAINS_BY_CATEGORY.aptos.length).toBe(2); // mainnet + testnet
      expect(CHAINS_BY_CATEGORY.sui.length).toBe(2); // mainnet + testnet
    });
  });

  describe('Helper Functions', () => {
    it('should get chain by ID correctly', () => {
      const ethereum = getChainById(1);
      expect(ethereum).toBeDefined();
      expect(ethereum?.name).toBe('Ethereum Mainnet');
      
      const bitcoin = getChainById(0);
      expect(bitcoin).toBeDefined();
      expect(bitcoin?.name).toBe('Bitcoin');
      
      const solana = getChainById(101);
      expect(solana).toBeDefined();
      expect(solana?.name).toBe('Solana Mainnet');
      
      const nonExistent = getChainById(999999);
      expect(nonExistent).toBeUndefined();
    });

    it('should get chains by algorithm', () => {
      const secp256k1Chains = getChainsByAlgorithm('secp256k1');
      expect(secp256k1Chains.length).toBeGreaterThan(10);
      expect(secp256k1Chains.every(c => c.algorithm === 'secp256k1')).toBe(true);
      
      const ed25519Chains = getChainsByAlgorithm('ed25519');
      expect(ed25519Chains.length).toBeGreaterThan(5);
      expect(ed25519Chains.every(c => c.algorithm === 'ed25519')).toBe(true);
    });

    it('should get chains by category', () => {
      const evmChains = getChainsByCategory('evm');
      expect(evmChains.length).toBeGreaterThan(5);
      expect(evmChains.every(c => c.category === 'evm')).toBe(true);
      
      const bitcoinChains = getChainsByCategory('bitcoin');
      expect(bitcoinChains.length).toBe(2);
      expect(bitcoinChains.every(c => c.category === 'bitcoin')).toBe(true);
    });

    it('should separate mainnet and testnet chains', () => {
      const mainnets = getMainnetChains();
      const testnets = getTestnetChains();
      
      expect(mainnets.every(c => !c.testnet)).toBe(true);
      expect(testnets.every(c => c.testnet)).toBe(true);
      
      // Should have roughly equal numbers (each mainnet has a testnet)
      expect(Math.abs(mainnets.length - testnets.length)).toBeLessThan(3);
    });

    it('should find testnet for mainnet', () => {
      const ethereumTestnet = getTestnetForMainnet(1);
      expect(ethereumTestnet).toBeDefined();
      expect(ethereumTestnet?.name).toBe('Sepolia');
      
      const bitcoinTestnet = getTestnetForMainnet(0);
      expect(bitcoinTestnet).toBeDefined();
      expect(bitcoinTestnet?.name).toBe('Bitcoin Testnet');
      
      const solanaTestnet = getTestnetForMainnet(101);
      expect(solanaTestnet).toBeDefined();
      expect(solanaTestnet?.name).toContain('Solana');
      expect(solanaTestnet?.testnet).toBe(true);
    });
  });

  describe('Default Chains', () => {
    it('should have sensible defaults for each algorithm', () => {
      expect(DEFAULT_CHAINS.secp256k1.length).toBeGreaterThan(3);
      expect(DEFAULT_CHAINS.ed25519.length).toBeGreaterThan(1);
      
      // Should include major chains
      const defaultSecp256k1Ids = DEFAULT_CHAINS.secp256k1.map(c => c.id);
      expect(defaultSecp256k1Ids).toContain(1); // Ethereum
      expect(defaultSecp256k1Ids).toContain(137); // Polygon
      expect(defaultSecp256k1Ids).toContain(56); // BSC
      
      const defaultEd25519Ids = DEFAULT_CHAINS.ed25519.map(c => c.id);
      expect(defaultEd25519Ids).toContain(101); // Solana mainnet
    });
  });

  describe('Chain-specific Validation', () => {
    it('should have correct Bitcoin configuration', () => {
      const bitcoin = getChainById(0);
      expect(bitcoin?.derivationPath).toBe("m/84'/0'/0'/0/0"); // BIP84
      expect(bitcoin?.addressPrefix).toBe('bc1');
      expect(bitcoin?.decimals).toBe(8);
      expect(bitcoin!.nativeCurrency!.decimals).toBe(8);
    });

    it('should have correct Ethereum configuration', () => {
      const ethereum = getChainById(1);
      expect(ethereum?.derivationPath).toBe("m/44'/60'/0'/0/0");
      expect(ethereum?.decimals).toBe(18);
      expect(ethereum!.nativeCurrency!.symbol).toBe('ETH');
    });

    it('should have correct Solana configuration', () => {
      const solana = getChainById(101);
      expect(solana?.derivationPath).toBe("m/44'/501'/0'/0'");
      expect(solana?.decimals).toBe(9);
      expect(solana!.nativeCurrency!.symbol).toBe('SOL');
    });

    it('should have correct Layer 2 configurations', () => {
      // Arbitrum
      const arbitrum = getChainById(42161);
      expect(arbitrum?.category).toBe('evm');
      expect(arbitrum?.algorithm).toBe('secp256k1');
      expect(arbitrum!.nativeCurrency!.symbol).toBe('ETH');
      
      // Polygon
      const polygon = getChainById(137);
      expect(polygon?.category).toBe('evm');
      expect(polygon!.nativeCurrency!.symbol).toBe('MATIC');
    });
  });

  describe('RPC and Explorer URLs', () => {
    it('should have valid RPC URLs', () => {
      ALL_CHAINS.forEach(chain => {
        chain.rpcUrls!.default.http.forEach(url => {
          expect(url).toMatch(/^https?:\/\//);
        });
      });
    });

    it('should have valid block explorer URLs', () => {
      ALL_CHAINS.forEach(chain => {
        expect(chain.blockExplorers!.default.url).toMatch(/^https?:\/\//);
      });
    });
  });
});
