import { defineUnlistedScript } from '#imports';
import { MESSAGE_PREFIX, MessageType, WALLET_INFO, DEFAULT_ADDRESSES } from '../../constants';
import { v4 as uuidv4 } from 'uuid';

// EIP6963 types
interface EIP6963ProviderInfo {
  uuid: string;
  name: string;
  icon: string;
  rdns: string;
  description?: string;
}

interface EIP6963ProviderDetail {
  info: EIP6963ProviderInfo;
  provider: any; // Provider following EIP-1193 standard
}

// Convert icon to data URI
const iconUrl = 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjEyOCIgdmlld0JveD0iMCAwIDEyOCAxMjgiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iMTI4IiByeD0iNjQiIGZpbGw9IiM2MzY2RjEiLz4KPHBhdGggZD0iTTY3LjQyIDE4LjAyNUM2OC4xNzk0IDE4LjAyNSA2OC43OTUgMTguNjQwOCA2OC43OTUgMTkuNFY0MS42QzY4Ljc5NSA0Mi4zNTkyIDY4LjE3OTQgNDIuOTc1IDY3LjQyIDQyLjk3NUg2MC41OEM2MC4yMDQgNDIuOTc1IDU5Ljg0MzggNDIuODIyIDU5LjU3NzQgNDIuNTU1NkM1OS4zMTEgNDIuMjg5NCA1OS4xNTggNDEuOTI5MiA1OS4xNTggNDEuNTUzVjE5LjQwMkM1OS4xNTggMTkuMDI1OCA1OS4zMTEgMTguNjY1NiA1OS41Nzc0IDE4LjM5OTJDNTkuODQzOCAxOC4xMzI4IDYwLjIwNCAxNy45OCA2MC41OCAxNy45OEw2Ny40MiAxOC4wMjVaTTQyLjk3NSAxOS40MDNDNDIuOTc1IDE4LjY0MzggNDMuNTkwOCAxOC4wMjgxIDQ0LjM1IDE4LjAyODFINTEuMTYzNEM1MS45MjI2IDE4LjAyODEgNTIuNTM4NCAxOC42NDM4IDUyLjUzODQgMTkuNDAzVjQ4LjQ0QzUyLjUzODQgNDguODE2MSA1Mi4zODUzIDQ5LjE3NjIgNTIuMTE4OSA0OS40NDI1QzUxLjg1MjUgNDkuNzA4OSA1MS40OTI0IDQ5Ljg2MiA1MS4xMTYzIDQ5Ljg2MkM1MC43NDAyIDQ5Ljg2MiA1MC4zODAxIDQ5LjcwODkgNTAuMTEzOCA0OS40NDI1QzQ5Ljg0NzQgNDkuMTc2MiA0OS42OTQzIDQ4LjgxNjEgNDkuNjk0MyA0OC40NFYyMC44NzJINDUuOTE5VjQ4LjQ0QzQ1LjkxOSA0OS4xOTkyIDQ1LjMwMzIgNDkuODE1IDQ0LjU0NCA0OS44MTVDNDMuNzg0OCA0OS44MTUgNDMuMTY5IDQ5LjE5OTIgNDMuMTY5IDQ4LjQ0VjE5LjQwM0g0Mi45NzVaTTg0Ljk0IDE5LjM0NzhDODQuOTQgMTguNTg4NiA4NS41NTU4IDE3Ljk3MjggODYuMzE1IDE3Ljk3MjhIOTMuMDlDOTMuODQ5MiAxNy45NzI4IDk0LjQ2NSAxOC41ODg2IDk0LjQ2NSAxOS4zNDc4VjI3Ljk3SDEwMy4wOUMxMDMuODQ5IDI3Ljk3IDEwNC40NjUgMjguNTg1OCAxMDQuNDY1IDI5LjM0NVYzNS45N0MxMDQuNDY1IDM2LjcyOTIgMTAzLjg0OSAzNy4zNDUgMTAzLjA5IDM3LjM0NUg5NC40NjVWNDEuNTUzQzk0LjQ2NSA0Mi4zMTIyIDkzLjg0OTIgNDIuOTI4IDkzLjA5IDQyLjkyOEg4Ni4zMTVDODUuNTU1OCA0Mi45MjggODQuOTQgNDIuMzEyMiA4NC45NCA0MS41NTNWMzcuMzQ1SDc1Ljk2QzQyLjk3NSAzNy4zNDUgNDMuMDM0MyAxMDQuMzYgNzUuOTYgMTA0LjM2SDg0LjY2NVY5OS4wOUg3NS45NkM0OS45OTU4IDk5LjA5IDQ4LjI0NSA0Mi42MTUgNzUuOTYgNDIuNjE1SDg0Ljk0VjQxLjU1M1YxOS4zNDc4WiIgZmlsbD0id2hpdGUiLz4KPC9zdmc+Cg==';
// Network information for Ethereum mainnet
const ETHEREUM_MAINNET = {
    chainId: '0x1',
    chainIdDecimal: 1,
    networkVersion: '1',
    name: 'Ethereum Mainnet',
    rpcUrl: 'https://mainnet.infura.io/v3/',
    blockExplorerUrl: 'https://etherscan.io',
    ticker: 'ETH',
    tickerName: 'Ethereum'
};

class PageProvider {
    private requestIdCounter = 0;
    private pendingRequests = new Map<number | string, (response: any) => void>();
    private eventHandlers: { [event: string]: Set<(params: any) => void> } = {};
    
    // Network properties
    public chainId: string = ETHEREUM_MAINNET.chainId;
    public networkVersion: string = ETHEREUM_MAINNET.networkVersion;
    public _chainId: string = ETHEREUM_MAINNET.chainId; // For legacy compatibility
    public _networkVersion: string = ETHEREUM_MAINNET.networkVersion; // For legacy compatibility
    public networkName: string = ETHEREUM_MAINNET.name;
    public ticker: string = ETHEREUM_MAINNET.ticker;
    public tickerName: string = ETHEREUM_MAINNET.tickerName;
    
    // Network info accessible as a property
    public readonly networkInfo = ETHEREUM_MAINNET;
    
    constructor() {
        // Add message listener to receive responses from content script
        window.addEventListener('message', this.handleContentScriptMessage);
        
        // Initialize event handlers storage
        this.eventHandlers = {
            'accountsChanged': new Set(),
            'chainChanged': new Set(),
            'connect': new Set(),
            'disconnect': new Set(),
            'message': new Set()
        };
        
        // Try to initialize cached accounts from sessionStorage
        try {
            if (window.sessionStorage) {
                const storedData = window.sessionStorage.getItem('starlab_wallet_accounts');
                if (storedData) {
                    try {
                        const data = JSON.parse(storedData);
                        if (Array.isArray(data) && data.length > 0) {
                            this.cachedAccounts = data;
                            console.log('[Injected Provider] Initialized with cached accounts from sessionStorage:', this.cachedAccounts);
                        }
                    } catch (parseError) {
                        console.error('[Injected Provider] Error parsing sessionStorage data:', parseError);
                    }
                }
            }
        } catch (e) {
            console.log('[Injected Provider] Error accessing sessionStorage during initialization:', e);
        }
    }

    // Handle messages from content script
    private handleContentScriptMessage = (event: MessageEvent) => {
        // Safety check: ensure message is from current window
        if (event.source !== window) return;

        const data = event.data;

        // Validate message format
        if (
            data &&
            typeof data === 'object' &&
            data.type === `${MESSAGE_PREFIX}${MessageType.RESPONSE}` &&
            data.payload
        ) {
            const response = data.payload;
            const callback = this.pendingRequests.get(response.id);

            if (callback) {
                callback(response);
                this.pendingRequests.delete(response.id);
            }
        }
    };

    // EIP-1193 required methods
    request = (args: { method: string, params?: unknown[] | object }): Promise<unknown> => {
        console.log(`[Injected Provider] Request called with method: ${args.method}`, args);
        
        return new Promise((resolve, reject) => {
            const id = this.requestIdCounter++;

            // Define proceedWithRequest at the top of the function body
            // to avoid hoisting issues
            const proceedWithRequest = () => {
                // Create JSON-RPC request
                const request = {
                    id,
                    jsonrpc: '2.0',
                    method: args.method,
                    params: args.params || []
                };

                // Store callback
                this.pendingRequests.set(id, (response: any) => {
                    console.log(`[Injected Provider] Received response for ${args.method}:`, response);
                    
                    if (response.error) {
                        // Handle different error formats that might be received
                        let errorMessage = 'Unknown error';
                        
                        if (typeof response.error === 'string') {
                            errorMessage = response.error;
                        } else if (typeof response.error === 'object' && response.error !== null) {
                            errorMessage = response.error.message || JSON.stringify(response.error);
                        }
                        
                        console.error(`[Injected Provider] Request failed: ${errorMessage}`);
                        reject(new Error(errorMessage));
                    } else {
                        // For eth_requestAccounts, emit events
                        if (args.method === 'eth_requestAccounts' && Array.isArray(response.result)) {
                            const chainId = this.chainId || '0x1';
                            this.emit('connect', { chainId });
                            this.emit('accountsChanged', response.result);
                        }
                        resolve(response.result);
                    }
                });

                // Send message to content script
                window.postMessage(
                    {
                        type: `${MESSAGE_PREFIX}${MessageType.REQUEST}`,
                        payload: request
                    },
                    '*'
                );
            };
            
            // Handle eth_requestAccounts specially - this is the connection request
            if (args.method === 'eth_requestAccounts') {
                console.log('[Injected Provider] Processing eth_requestAccounts connection request');
                
                // Auto-approve connections for now to prevent wallet getting stuck
                // In the future, this should trigger a popup for user approval
                this.getAccounts().then(accounts => {
                    console.log('[Injected Provider] Auto-approved connection with accounts:', accounts);
                    
                    // If we got accounts, resolve with them
                    if (accounts && accounts.length > 0) {
                        // Emit connection event
                        const chainId = this.chainId || '0x1';
                        this.emit('connect', { chainId });
                        this.emit('accountsChanged', accounts);
                        
                        resolve(accounts);
                        return;
                    }
                    
                    // If we don't have accounts yet, proceed with normal request flow
                    console.log('[Injected Provider] No accounts found, proceeding with RPC request');
                    proceedWithRequest();
                }).catch(error => {
                    console.error('[Injected Provider] Error in getAccounts during connection:', error);
                    proceedWithRequest();
                });
            } else {
                // For all other methods, proceed normally
                proceedWithRequest();
            }
        });
    };

    // EIP-1193 required event methods
    on(eventName: string, listener: (params: any) => void): void {
        if (!this.eventHandlers[eventName]) {
            this.eventHandlers[eventName] = new Set();
        }
        this.eventHandlers[eventName].add(listener);
    }

    removeListener(eventName: string, listener: (params: any) => void): void {
        if (this.eventHandlers[eventName]) {
            this.eventHandlers[eventName].delete(listener);
        }
    }

    // Helper to emit events to listeners
    emit(eventName: string, params: any): void {
        if (this.eventHandlers[eventName]) {
            this.eventHandlers[eventName].forEach(listener => {
                try {
                    listener(params);
                } catch (error) {
                    console.error(`Error in ${eventName} listener:`, error);
                }
            });
        }
    }

    // Clean up when PageProvider is destroyed

    // Cache property values. Not marked `private` because the
    // factory code below (line ~466) intentionally seeds
    // cachedAccounts before the async real-accounts fetch completes,
    // and the same factory clears it to force re-fetch after
    // initial delay. That external seeding is the documented
    // initialization path — marking these `private` was a type-
    // surface bug, not an API contract.
    cachedAccounts: string[] | null = null;
    cachedChainId: string | null = null;
    
    // Network-related methods
    getChainId = async (): Promise<string> => {
        if (this.cachedChainId) {
            return this.cachedChainId;
        }
        
        try {
            console.log('[Injected Provider] Requesting chain ID from background');
            const result = await this.request({ method: 'eth_chainId' });
            if (typeof result === 'string') {
                this.cachedChainId = result;
                this.chainId = result;
                this._chainId = result;
                console.log('[Injected Provider] Received chain ID:', result);
                return result;
            }
        } catch (error) {
            console.error('[Injected Provider] Error getting chain ID:', error);
        }
        
        return this.chainId; // Return default if we couldn't get it
    };
    
    // Legacy method for compatibility
    getNetworkVersion = async (): Promise<string> => {
        try {
            const result = await this.request({ method: 'net_version' });
            if (typeof result === 'string') {
                this.networkVersion = result;
                this._networkVersion = result;
                return result;
            }
        } catch (error) {
            console.error('[Injected Provider] Error getting network version:', error);
        }
        
        return this.networkVersion; // Return default
    };
    
    // Get accounts method - required by many dapps
    getAccounts = async (): Promise<string[]> => {
        // If we have cached accounts, return them immediately to prevent loops
        if (this.cachedAccounts && this.cachedAccounts.length > 0) {
            console.log('[Injected Provider] Returning cached accounts:', this.cachedAccounts);
            return this.cachedAccounts;
        }
        
        // Always have a fallback ready
        const fallbackAddress = DEFAULT_ADDRESSES.ethereum();
        
        try {
            console.log('[Injected Provider] Requesting accounts from background');
            // Use direct messaging to content script for better stability
            const result = await this.getAccountsDirectly();
            
            if (Array.isArray(result) && result.length > 0) {
                this.cachedAccounts = result as string[];
                console.log('[Injected Provider] Received and cached accounts:', this.cachedAccounts);
                return this.cachedAccounts;
            } else {
                console.log('[Injected Provider] No accounts received from direct method');
                // Fallback to the default address
                this.cachedAccounts = [fallbackAddress];
                return this.cachedAccounts;
            }
        } catch (error) {
            console.error('[Injected Provider] Error in getAccounts:', error);
            
            // For testing and development - return a placeholder address if we can't get a real one
            console.log('[Injected Provider] Using fallback address due to error:', fallbackAddress);
            this.cachedAccounts = [fallbackAddress];
            return this.cachedAccounts;
        }
    };
    
    // Direct method to get accounts that doesn't rely on the request method
    private getAccountsDirectly = (): Promise<string[]> => {
        return new Promise((resolve) => {
            // Try to get saved address from sessionStorage, which is safer in injected context
            try {
                const storedData = window.sessionStorage.getItem('starlab_wallet_accounts');
                if (storedData) {
                    try {
                        const data = JSON.parse(storedData);
                        if (Array.isArray(data) && data.length > 0) {
                            console.log('[Injected Provider] Found accounts in sessionStorage:', data);
                            
                            // Verify the address is valid and not a default placeholder
                            const defaultAddress = DEFAULT_ADDRESSES.ethereum();
                            if (data[0] !== defaultAddress && 
                                !data[0].startsWith('0x000000') && 
                                data[0].match(/^0x[a-fA-F0-9]{40}$/)) {
                                
                                console.log('[Injected Provider] Using saved account address:', data[0]);
                                resolve(data);
                                return;
                            } else {
                                console.log('[Injected Provider] Found fallback address in storage, will try refresh');
                            }
                        }
                    } catch (parseError) {
                        console.error('[Injected Provider] Error parsing sessionStorage data:', parseError);
                    }
                }
            } catch (e) {
                console.log('[Injected Provider] Error accessing sessionStorage:', e);
            }
            
            // First try to get directly from chrome.storage.local if this is possible in this context
            if (typeof chrome !== 'undefined' && chrome.storage && chrome.storage.local) {
                try {
                    chrome.storage.local.get(['mpc_ethereum_address'], (result) => {
                        if (chrome.runtime.lastError) {
                            console.log('[Injected Provider] Error accessing chrome.storage:', chrome.runtime.lastError);
                            proceedWithPageRequest();
                            return;
                        }
                        
                        if (result && result.mpc_ethereum_address) {
                            const address = result.mpc_ethereum_address;
                            console.log('[Injected Provider] Got address directly from chrome.storage:', address);
                            
                            // Validate the address format
                            if (address.match(/^0x[a-fA-F0-9]{40}$/) && 
                                !address.startsWith('0x000000')) {
                                
                                // Save to sessionStorage for future use
                                try {
                                    if (window.sessionStorage) {
                                        window.sessionStorage.setItem('starlab_wallet_accounts', 
                                            JSON.stringify([address]));
                                    }
                                } catch (e) {
                                    console.log('[Injected Provider] Error saving to sessionStorage:', e);
                                }
                                
                                resolve([address]);
                                return;
                            }
                        }
                        
                        // If we get here, we need to proceed with page request
                        proceedWithPageRequest();
                    });
                } catch (e) {
                    console.log('[Injected Provider] Error with chrome.storage:', e);
                    proceedWithPageRequest();
                }
            } else {
                // No chrome.storage access, proceed with page request
                proceedWithPageRequest();
            }
            
            // Function to send page request and check sessionStorage after
            function proceedWithPageRequest() {
                console.log('[Injected Provider] Sending direct page request for accounts');
                window.postMessage(
                    {
                        type: `${MESSAGE_PREFIX}${MessageType.REQUEST}`,
                        payload: {
                            id: 'direct-accounts-req-' + Date.now(),
                            jsonrpc: '2.0',
                            method: 'eth_accounts',
                            params: []
                        }
                    },
                    '*'
                );
                
                // Give it some time to get processed, then check sessionStorage again
                setTimeout(() => {
                    try {
                        const refreshedData = window.sessionStorage.getItem('starlab_wallet_accounts');
                        if (refreshedData) {
                            try {
                                const data = JSON.parse(refreshedData);
                                if (Array.isArray(data) && data.length > 0 && 
                                    data[0].match(/^0x[a-fA-F0-9]{40}$/) && 
                                    !data[0].startsWith('0x000000')) {
                                    
                                    console.log('[Injected Provider] Found refreshed accounts in sessionStorage:', data);
                                    resolve(data);
                                    return;
                                }
                            } catch (parseError) {
                                console.error('[Injected Provider] Error parsing refreshed sessionStorage data:', parseError);
                            }
                        }
                        // If we still don't have valid accounts, use fallback with unique seed
                        // Use a specific string that's unique per domain for better UX
                        const seed = window.location.hostname || 'unknown-site';
                        resolve([DEFAULT_ADDRESSES.ethereum(seed)]);
                    } catch (e) {
                        console.log('[Injected Provider] Error accessing refreshed sessionStorage:', e);
                        resolve([DEFAULT_ADDRESSES.ethereum()]);
                    }
                }, 500); // Give it 500ms to process
            }
        });
    };
    
    // For compatibility with legacy web3.js
    isConnected = (): boolean => {
        return true; // Always return true as per most wallet providers
    };
    
    disconnect(): void {
        window.removeEventListener('message', this.handleContentScriptMessage);
        // Clear all event handlers
        for (const key in this.eventHandlers) {
            this.eventHandlers[key].clear();
        }
    }
}

export default defineUnlistedScript(() => {
    // Define the provider object with additional standard properties
    interface Window {
        ethereum: any;
    }
    const info = {
        uuid: uuidv4(),
        name: WALLET_INFO.name,
        icon: iconUrl,
        rdns: 'org.starlab.wallet',
        description: WALLET_INFO.description
    };

    // Initialize provider with window.ethereum properties
    const provider = new PageProvider();
    
    console.log('[Injected Provider] Initializing Starlab Wallet provider');
    
    // Immediately provide access to a placeholder account for smoother dapp experience
    // This will be replaced by the real address once available
    const initialAccounts = [DEFAULT_ADDRESSES.ethereum()];
    provider.cachedAccounts = initialAccounts;
    
    // Handle the initial chain ID and connection status
    provider.emit('connect', { chainId: provider.chainId });
    provider.emit('accountsChanged', initialAccounts);
    console.log('[Injected Provider] Emitted initial connect and accountsChanged events');
    
    // Try to fetch the real account details async
    setTimeout(() => {
        console.log('[Injected Provider] Attempting to fetch real accounts...');
        provider.cachedAccounts = null; // Clear cache to force fetch
        provider.getAccounts().then(accounts => {
            if (accounts && accounts.length > 0 && accounts[0] !== initialAccounts[0]) {
                console.log('[Injected Provider] Got real accounts, updating:', accounts);
                provider.emit('accountsChanged', accounts);
            }
        }).catch(error => {
            console.error('[Injected Provider] Failed to get real accounts:', error);
        });
    }, 1000);
    
    const injectedProvider = new Proxy(provider, {
        deleteProperty: (target, prop) => {
            if (typeof prop === 'string' && ['on'].includes(prop)) {
                // @ts-ignore
                delete target[prop];
            }
            return true;
        },
        get: (target, prop, receiver) => {
            const method = target[prop as keyof PageProvider];
            if (typeof method === 'function') {
                return (...args: any[]) => {
                    // @ts-ignore
                    return method.apply(target, args);
                };
            }

            return Reflect.get(target, prop, receiver);
        },
    });

    const createAndDispatchEvent = (e?: Event) => {
        try {
            // Create EIP6963ProviderDetail object
            const providerDetail: EIP6963ProviderDetail = {
                info: info,
                provider: injectedProvider
            };
            
            // Create and dispatch the event with frozen detail object
            const event = new CustomEvent(
                'eip6963:announceProvider',
                { detail: Object.freeze(providerDetail) }
            );
            window.dispatchEvent(event);
            console.log('EIP6963 provider announced:', info.name, info.rdns);
        } catch (error) {
            console.error('Failed to dispatch EIP6963 event:', error);
        }
    };

    // Add standard properties to the provider
    // These are common properties expected by many dapps
    injectedProvider.isStarLabWallet = true;
    
    // Add additional properties for compatibility with MetaMask test dapp
    injectedProvider.networkVersion = provider.networkVersion;
    injectedProvider._chainId = provider.chainId; // Some dapps check for this property
    injectedProvider.chainId = provider.chainId;
    injectedProvider._metamask = { isUnlocked: async () => true }; // For compatibility
    
    // For MetaMask test dapp specifically
    injectedProvider.sendAsync = (request: any, callback: any) => {
        console.log('[Injected Provider] sendAsync called with:', request);
        provider.request(request)
            .then(result => callback(null, { id: request.id, jsonrpc: '2.0', result }))
            .catch(error => callback(error, null));
    };
    
    // Legacy send method
    injectedProvider.send = (methodOrPayload: any, paramsOrCallback: any) => {
        console.log('[Injected Provider] send called with:', methodOrPayload, paramsOrCallback);
        // Handle as a synchronous RPC call
        if (typeof methodOrPayload === 'string' && Array.isArray(paramsOrCallback)) {
            const method = methodOrPayload;
            const params = paramsOrCallback;
            return provider.request({ method, params });
        }
        // Handle as a legacy async call with callback
        else if (typeof paramsOrCallback === 'function') {
            return injectedProvider.sendAsync(methodOrPayload, paramsOrCallback);
        }
        // Handle as a synchronous call with payload
        else {
            const payload = methodOrPayload;
            const response = { id: payload.id, jsonrpc: '2.0', result: null };
            
            // For simple synchronous methods
            if (payload.method === 'eth_accounts') {
                response.result = provider.cachedAccounts || [];
            } else if (payload.method === 'eth_coinbase') {
                response.result = (provider.cachedAccounts && provider.cachedAccounts[0]) || null;
            } else if (payload.method === 'eth_chainId') {
                response.result = provider.chainId;
            } else if (payload.method === 'net_version') {
                response.result = provider.networkVersion;
            } else {
                throw new Error(`Synchronous RPC method ${payload.method} not supported`);
            }
            
            return response;
        }
    };
    
    // ONLY make our provider accessible as window.starlabEthereum
    Object.defineProperty(window, 'starlabEthereum', {
        value: injectedProvider,
        writable: false,
        configurable: true
    });
    
    // Listen for EIP6963 provider request events
    window.addEventListener('eip6963:requestProvider', createAndDispatchEvent);
    
    // Announce the provider on page load
    setTimeout(createAndDispatchEvent, 0);
    
    return injectedProvider;
});
