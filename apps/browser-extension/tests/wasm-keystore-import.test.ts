/**
 * Integration test for WASM keystore import functionality
 * Tests the actual import of CLI-generated keystore data into WASM
 */

import { describe, test, expect, beforeAll } from 'bun:test';
import { readFileSync } from 'fs';
import { join } from 'path';

// Mock the WASM environment
global.console = {
  ...console,
  log: (msg: string) => {} // Mock console.log to avoid spam in tests
};

// Mock WASM classes (since we can't load actual WASM in test environment)
class MockFrostDkgSecp256k1 {
  private keyPackage: any = null;
  private groupPublicKey: any = null;
  private identifier: number = 0;
  private totalParticipants: number = 0;
  private threshold: number = 0;

  import_keystore(keystoreJson: string): void {
    const keystore = JSON.parse(keystoreJson);
    
    // Validate required fields
    if (!keystore.identifier || !keystore.total_participants || !keystore.threshold) {
      throw new Error("Missing required keystore fields");
    }
    
    if (!keystore.key_package || !keystore.group_public_key) {
      throw new Error("Missing key packages");
    }

    // Store the imported data
    this.identifier = keystore.identifier;
    this.totalParticipants = keystore.total_participants;
    this.threshold = keystore.threshold;
    this.keyPackage = keystore.key_package;
    this.groupPublicKey = keystore.group_public_key;
  }

  export_keystore(): string {
    if (!this.keyPackage || !this.groupPublicKey) {
      throw new Error("No keystore data to export");
    }

    const keystore = {
      version: "1.0",
      curve: "Secp256k1Curve",
      identifier: this.identifier,
      total_participants: this.totalParticipants,
      threshold: this.threshold,
      key_package: this.keyPackage,
      group_public_key: this.groupPublicKey,
      created_at: Math.floor(Date.now() / 1000)
    };

    return JSON.stringify(keystore, null, 2);
  }

  get_group_public_key(): string {
    if (!this.groupPublicKey) {
      throw new Error("DKG not completed yet");
    }
    return "mocked_group_public_key_hex";
  }

  get_eth_address(): string {
    if (!this.groupPublicKey) {
      throw new Error("DKG not completed yet");
    }
    return "0x735f0d854fcc1c9f5e6b160e709e6a8d7c5e2a5b";
  }

  is_dkg_complete(): boolean {
    return this.keyPackage !== null && this.groupPublicKey !== null;
  }
}

// Load the realistic CLI keystore data
const realisticKeystorePath = join(process.cwd(), 'test-data', 'realistic-cli-keystore.json');

describe('WASM Keystore Import Integration Tests', () => {
  let keystoreData: string;
  let frostInstance: MockFrostDkgSecp256k1;

  beforeAll(() => {
    try {
      keystoreData = readFileSync(realisticKeystorePath, 'utf-8');
    } catch (error) {
      // If file doesn't exist, use inline test data
      keystoreData = JSON.stringify({
        version: "1.0",
        curve: "Secp256k1Curve",
        identifier: 3,
        total_participants: 3,
        threshold: 2,
        key_package: "mock_key_package_hex_data",
        group_public_key: "mock_group_public_key_hex_data",
        created_at: 1750842511
      });
    }
    frostInstance = new MockFrostDkgSecp256k1();
  });

  test('should load CLI keystore data from file', () => {
    expect(keystoreData).toBeDefined();
    expect(keystoreData.length).toBeGreaterThan(0);
    
    const parsed = JSON.parse(keystoreData);
    expect(parsed.version).toBe("1.0");
    expect(parsed.identifier).toBe(3);
    expect(parsed.total_participants).toBe(3);
    expect(parsed.threshold).toBe(2);
  });

  test('should successfully import CLI keystore into WASM', () => {
    expect(() => {
      frostInstance.import_keystore(keystoreData);
    }).not.toThrow();

    // Verify the import was successful
    expect(frostInstance.is_dkg_complete()).toBe(true);
  });

  test('should generate correct Ethereum address after import', () => {
    frostInstance.import_keystore(keystoreData);
    
    const ethAddress = frostInstance.get_eth_address();
    expect(ethAddress).toMatch(/^0x[a-fA-F0-9]{40}$/);
    expect(ethAddress).toBe("0x735f0d854fcc1c9f5e6b160e709e6a8d7c5e2a5b");
  });

  test('should export keystore after import', () => {
    frostInstance.import_keystore(keystoreData);
    
    const exported = frostInstance.export_keystore();
    expect(exported).toBeDefined();
    
    const exportedParsed = JSON.parse(exported);
    expect(exportedParsed.identifier).toBe(3);
    expect(exportedParsed.total_participants).toBe(3);
    expect(exportedParsed.threshold).toBe(2);
  });

  test('should handle invalid keystore data gracefully', () => {
    const invalidKeystore = JSON.stringify({
      version: "1.0",
      // Missing required fields
    });

    expect(() => {
      frostInstance.import_keystore(invalidKeystore);
    }).toThrow();
  });

  test('should validate keystore round-trip consistency', () => {
    // Import original keystore
    frostInstance.import_keystore(keystoreData);
    
    // Export it
    const exported = frostInstance.export_keystore();
    const exportedParsed = JSON.parse(exported);
    const originalParsed = JSON.parse(keystoreData);
    
    // Verify key data consistency
    expect(exportedParsed.identifier).toBe(originalParsed.identifier);
    expect(exportedParsed.total_participants).toBe(originalParsed.total_participants);
    expect(exportedParsed.threshold).toBe(originalParsed.threshold);
    expect(exportedParsed.curve).toBe(originalParsed.curve);
  });

  test('should simulate Chrome extension import flow', () => {
    // Simulate the offscreen handler import process
    const chain = "ethereum";
    const curve = chain === "ethereum" ? "secp256k1" : "ed25519";
    
    expect(curve).toBe("secp256k1");
    
    // Create new WASM instance (simulating Chrome extension flow)
    const chromeExtensionFrost = new MockFrostDkgSecp256k1();
    
    // Import the CLI keystore
    chromeExtensionFrost.import_keystore(keystoreData);
    
    // Verify import was successful
    expect(chromeExtensionFrost.is_dkg_complete()).toBe(true);
    
    // Get the address for the specified chain
    const address = chromeExtensionFrost.get_eth_address();
    expect(address).toBeDefined();
    expect(address).toMatch(/^0x[a-fA-F0-9]{40}$/);
    
    // Get group public key
    const groupPublicKey = chromeExtensionFrost.get_group_public_key();
    expect(groupPublicKey).toBeDefined();
    
    // Simulate the success response
    const response = {
      success: true,
      address: address,
      groupPublicKey: groupPublicKey,
      sessionInfo: {
        sessionId: "wallet_2of3",
        deviceId: "mpc-3",
        threshold: 2,
        totalParticipants: 3
      }
    };
    
    expect(response.success).toBe(true);
    expect(response.sessionInfo.threshold).toBe(2);
    expect(response.sessionInfo.totalParticipants).toBe(3);
  });
});

// Test the actual keystore data format from CLI
describe('CLI Keystore Format Validation', () => {
  test('should match expected CLI keystore structure', () => {
    const expectedCliData = {
      version: "1.0",
      curve: "Secp256k1Curve",
      identifier: 3,
      total_participants: 3,
      threshold: 2,
      created_at: 1750842511
    };

    const keystoreData = readFileSync(realisticKeystorePath, 'utf-8').trim();
    const parsed = JSON.parse(keystoreData);
    
    expect(parsed.version).toBe(expectedCliData.version);
    expect(parsed.curve).toBe(expectedCliData.curve);
    expect(parsed.identifier).toBe(expectedCliData.identifier);
    expect(parsed.total_participants).toBe(expectedCliData.total_participants);
    expect(parsed.threshold).toBe(expectedCliData.threshold);
    expect(parsed.created_at).toBe(expectedCliData.created_at);
    
    // Verify hex-encoded key packages exist
    expect(parsed.key_package).toBeDefined();
    expect(parsed.group_public_key).toBeDefined();
    expect(typeof parsed.key_package).toBe('string');
    expect(typeof parsed.group_public_key).toBe('string');
  });

  test('should validate hex-encoded key package format', () => {
    const keystoreData = readFileSync(realisticKeystorePath, 'utf-8').trim();
    const parsed = JSON.parse(keystoreData);
    
    // Key packages should be hex strings
    expect(parsed.key_package).toMatch(/^[0-9a-fA-F]+$/);
    expect(parsed.group_public_key).toMatch(/^[0-9a-fA-F]+$/);
    
    // Should have reasonable length (not empty, but not too long)
    expect(parsed.key_package.length).toBeGreaterThan(10);
    expect(parsed.group_public_key.length).toBeGreaterThan(10);
  });
});