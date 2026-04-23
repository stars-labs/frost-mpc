// Re-export domain types so existing imports from '@mpc-wallet/types/appstate' still work.
// Canonical definitions live in session.ts, dkg.ts, mesh.ts, and webrtc.ts.
export type { SessionInfo, SessionProposal, SessionResponse } from './session';
export { DkgState } from './dkg';
export { MeshStatusType } from './mesh';
export type { MeshStatus } from './mesh';
export type { WebRTCAppMessage } from './webrtc';

// AppState and related utilities are unique to this file.
import type { SessionInfo } from './session';
import type { MeshStatus } from './mesh';
import { MeshStatusType } from './mesh';
import { DkgState } from './dkg';

export interface AppState {
  deviceId: string;
  connecteddevices: string[];
  wsConnected: boolean;
  /** Last WebSocket error; cleared when connection re-establishes. */
  wsError?: string;
  sessionInfo: SessionInfo | null;
  invites: SessionInfo[];
  meshStatus: MeshStatus;
  dkgState: DkgState;
  webrtcConnections: Record<string, boolean>;
  blockchain?: "ethereum" | "solana";
  /**
   * FROST ciphersuite selection for the current wallet. Historically
   * tracked alongside `blockchain` for legacy code paths; setters
   * derive one from the other (secp256k1 ↔ ethereum, ed25519 ↔ solana).
   * Writers: StateManager.setBlockchain / setCurve.
   * Readers: StateManager.getCurve / getBlockchain.
   */
  curve?: "secp256k1" | "ed25519";
  /**
   * User-facing "chain" alias for blockchain. Some older code paths
   * persist + read this key; kept as an alias field so both work.
   * New code should prefer `blockchain`.
   */
  chain?: "ethereum" | "solana";
}

export const INITIAL_APP_STATE: AppState = {
  deviceId: '',
  connecteddevices: [],
  wsConnected: false,
  sessionInfo: null,
  invites: [],
  meshStatus: { type: MeshStatusType.Incomplete },
  dkgState: DkgState.Idle,
  webrtcConnections: {},
  blockchain: "ethereum"
};

export type SupportedChain = 'ethereum' | 'solana';

export const CURVE_COMPATIBLE_CHAINS: Record<string, SupportedChain[]> = {
  'secp256k1': ['ethereum'],
  'ed25519': ['solana']
};

export function getCompatibleChains(curveType: string): SupportedChain[] {
  return CURVE_COMPATIBLE_CHAINS[curveType] || [];
}

export function getRequiredCurve(chain: SupportedChain): string {
  for (const [curve, chains] of Object.entries(CURVE_COMPATIBLE_CHAINS)) {
    if (chains.includes(chain)) {
      return curve;
    }
  }
  return 'secp256k1';
}
