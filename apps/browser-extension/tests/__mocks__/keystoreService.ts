// Mock KeystoreService for tests
import { jest } from 'bun:test';
import type { KeyShareData, WalletMetadata, KeystoreBackup } from "@mpc-wallet/types/keystore";

export class KeystoreService {
  private static instance: KeystoreService | null = null;
  private isInitialized = false;
  private isUnlocked = false;
  private wallets: Map<string, WalletMetadata> = new Map();

  static getInstance(): KeystoreService {
    if (!KeystoreService.instance) {
      KeystoreService.instance = new KeystoreService();
    }
    return KeystoreService.instance;
  }

  async initialize(deviceId: string): Promise<void> {
    this.isInitialized = true;
  }

  async isSetup(): Promise<boolean> {
    return this.isInitialized;
  }

  async unlock(password: string): Promise<void> {
    this.isUnlocked = true;
  }

  async lock(): Promise<void> {
    this.isUnlocked = false;
  }

  isLocked(): boolean {
    return !this.isUnlocked;
  }

  async changePassword(oldPassword: string, newPassword: string): Promise<void> {
    // Mock implementation
  }

  async addWallet(walletId: string, keyShare: KeyShareData, metadata: WalletMetadata): Promise<void> {
    this.wallets.set(walletId, metadata);
  }

  async removeWallet(walletId: string): Promise<void> {
    this.wallets.delete(walletId);
  }

  async getKeyShare(walletId: string): Promise<KeyShareData | null> {
    return null;
  }

  getWallets(): WalletMetadata[] {
    return Array.from(this.wallets.values());
  }

  async exportWallet(walletId: string): Promise<KeystoreBackup> {
    return {
      version: '1.0.0',
      device_id: 'test-device',
      exportedAt: Date.now(),
      wallets: []
    };
  }

  async importWallet(backup: KeystoreBackup, password: string): Promise<string[]> {
    return [];
  }

  async clearAll(): Promise<void> {
    this.wallets.clear();
  }

  async markWalletBackedUp(walletId: string): Promise<void> {
    const wallet = this.wallets.get(walletId);
    if (wallet) {
      wallet.hasBackup = true;
    }
  }
}

export default KeystoreService;