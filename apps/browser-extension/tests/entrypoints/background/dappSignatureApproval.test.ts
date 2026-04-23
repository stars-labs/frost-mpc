/**
 * Ext-4-confirm regression tests for RpcHandler.approveDappSignature.
 *
 * The re-keying dance between placeholder requestId and real
 * session_id is the non-obvious core of the gated dApp flow:
 * when the RPC arrives, we register a pending Promise keyed by
 * `dapp_req_<ts>_<rand>`. On user approval, we create the session
 * via SessionManager and re-key the pending Promise to the
 * session_id so `stateManager.signingComplete` can find it later.
 * Break either half and the RPC hangs until timeout.
 *
 * Test surface: approveDappSignature is a pure TS method; we can
 * exercise it directly with a mocked SessionManager + mocked chrome
 * global without spinning up the service worker. Verifies:
 *   - Approve path: creates session, re-keys pending promise
 *   - Reject path: rejects pending promise with the standard
 *     "User rejected" error
 *   - Unknown requestId: returns error without touching the
 *     pending map
 *   - SessionManager not set: emits signature error, returns error
 *   - createSigningSession returns failure: propagates upstream
 *     error via handleSignatureError
 *   - Duplicate approve: second call is a no-op (context already
 *     consumed)
 */
import { describe, it, expect, beforeEach, jest, mock } from "bun:test";

// setup-bun.ts installs `#imports` as a mock module via its
// preload. Bun's mock registry has per-file isolation quirks:
// when this test file runs AFTER another that doesn't itself
// re-register the mock (e.g. signingDecline.test.ts), the
// subsequent transitive import chain `RpcHandler → permissionService
// → #imports` fails to resolve. Re-install the mock locally at
// file scope so the import below always sees it regardless of
// test-file ordering.
import * as mockImports from "../../wxt-imports-mock";
mock.module("#imports", () => mockImports);

import { RpcHandler } from "../../../src/entrypoints/background/rpcHandler";

// setup-bun.ts already installs a global chrome with storage +
// runtime. Here we only augment it with a notifications stub —
// approveDappSignature touches chrome.notifications.clear +
// handleSignMessageRequest touches chrome.notifications.create.
// beforeEach in setup-bun.ts resets storage but doesn't touch
// notifications, so we re-augment per test to keep jest.fn call
// counts clean.
function augmentChromeNotifications() {
    const chromeGlobal = (global as any).chrome ?? {};
    chromeGlobal.notifications = {
        clear: jest.fn(),
        create: jest.fn(),
    };
    (global as any).chrome = chromeGlobal;
}

function makeRpcHandler(opts: {
    sessionManagerCreates?: (args: any) => Promise<any>;
} = {}) {
    augmentChromeNotifications();
    const rpc = new RpcHandler();

    // Inject a fake sessionManager that records calls.
    const createCalls: any[] = [];
    const createSigningSession = jest.fn(async (args: any) => {
        createCalls.push(args);
        if (opts.sessionManagerCreates) {
            return await opts.sessionManagerCreates(args);
        }
        return { success: true, sessionId: "sign_abc123" };
    });
    (rpc as any).setSessionManager({ createSigningSession });

    // Seed a pending dApp request + pending promise so
    // approveDappSignature has something to consume. This mirrors
    // what handleSignMessageRequest does before the user interacts.
    const requestId = "dapp_req_1_aaa";
    let resolveFn: ((v: string) => void) | null = null;
    let rejectFn: ((e: any) => void) | null = null;
    const pendingPromise = new Promise<string>((resolve, reject) => {
        resolveFn = resolve;
        rejectFn = reject;
    });
    (rpc as any).pendingDappRequests.set(requestId, {
        walletId: "w1",
        walletName: "Treasury",
        groupPublicKey: "02cafe",
        blockchain: "ethereum" as const,
        threshold: 2,
        total: 3,
        messageHex: "deadbeef",
        originalMessage: "hello",
        address: "0xabc",
        origin: "https://dapp.example",
    });
    (rpc as any).pendingSignatures.set(requestId, {
        resolve: resolveFn!,
        reject: rejectFn!,
    });

    return {
        rpc,
        requestId,
        pendingPromise,
        createSigningSession,
        createCalls,
    };
}

describe("RpcHandler.approveDappSignature", () => {
    let env: ReturnType<typeof makeRpcHandler>;

    beforeEach(() => {
        env = makeRpcHandler();
    });

    it("approve: creates session via SessionManager", async () => {
        const result = await env.rpc.approveDappSignature(env.requestId, true);
        expect(result.success).toBe(true);
        expect(result.sessionId).toBe("sign_abc123");

        expect(env.createSigningSession).toHaveBeenCalledTimes(1);
        const args = env.createCalls[0];
        expect(args.walletId).toBe("w1");
        expect(args.walletName).toBe("Treasury");
        expect(args.groupPublicKey).toBe("02cafe");
        expect(args.blockchain).toBe("ethereum");
        expect(args.threshold).toBe(2);
        expect(args.total).toBe(3);
        expect(args.signingMessageHex).toBe("deadbeef");
    });

    it("approve: re-keys pending promise from placeholder to sessionId", async () => {
        await env.rpc.approveDappSignature(env.requestId, true);

        // Placeholder id should no longer have a pending entry.
        expect(
            (env.rpc as any).pendingSignatures.has(env.requestId),
        ).toBe(false);
        // Actual session id should now hold the pending entry.
        expect(
            (env.rpc as any).pendingSignatures.has("sign_abc123"),
        ).toBe(true);
    });

    it("approve: original RPC promise resolves when signature arrives under new key", async () => {
        await env.rpc.approveDappSignature(env.requestId, true);

        // Simulate stateManager.signingComplete firing.
        env.rpc.handleSignatureComplete("sign_abc123", "0xcafebabe");

        const sig = await env.pendingPromise;
        // handleSignatureComplete normalizes to 0x prefix already.
        expect(sig).toBe("0xcafebabe");
    });

    it("reject: rejects the pending promise with 'User rejected'", async () => {
        const result = await env.rpc.approveDappSignature(env.requestId, false);
        expect(result.success).toBe(true); // reject path returns success=true, the RPC itself failed

        await expect(env.pendingPromise).rejects.toThrow(
            "User rejected signature request",
        );
        // No session should have been created.
        expect(env.createSigningSession).not.toHaveBeenCalled();
    });

    it("reject: cleans up both pending maps", async () => {
        // Attach a catch handler so the rejected promise doesn't
        // trip bun's unhandled-rejection detection. We don't await
        // because the point of the test is the sync post-state of
        // the Maps, not the rejection value.
        env.pendingPromise.catch(() => {});
        await env.rpc.approveDappSignature(env.requestId, false);
        expect(
            (env.rpc as any).pendingDappRequests.has(env.requestId),
        ).toBe(false);
        expect(
            (env.rpc as any).pendingSignatures.has(env.requestId),
        ).toBe(false);
    });

    it("unknown requestId: returns error without side effects", async () => {
        const result = await env.rpc.approveDappSignature(
            "dapp_req_bogus",
            true,
        );
        expect(result.success).toBe(false);
        expect(result.error).toContain("No pending dApp signature request");
        expect(env.createSigningSession).not.toHaveBeenCalled();
    });

    it("sessionManager.createSigningSession returns failure: propagates error", async () => {
        const env2 = makeRpcHandler({
            sessionManagerCreates: async () => ({
                success: false,
                error: "Signal server not connected",
            }),
        });

        const result = await env2.rpc.approveDappSignature(env2.requestId, true);
        expect(result.success).toBe(false);
        expect(result.error).toContain("Signal server not connected");

        // Pending promise should reject with the propagated error.
        await expect(env2.pendingPromise).rejects.toThrow(
            "Signal server not connected",
        );
    });

    it("duplicate approve: second call is a safe no-op", async () => {
        await env.rpc.approveDappSignature(env.requestId, true);
        expect(env.createSigningSession).toHaveBeenCalledTimes(1);

        // Context consumed on first call; second finds nothing.
        const result2 = await env.rpc.approveDappSignature(
            env.requestId,
            true,
        );
        expect(result2.success).toBe(false);
        expect(env.createSigningSession).toHaveBeenCalledTimes(1);
    });

    it("approve after reject: is a no-op (pending context already deleted)", async () => {
        // Swallow the rejection so bun doesn't fail on unhandled.
        env.pendingPromise.catch(() => {});
        await env.rpc.approveDappSignature(env.requestId, false);
        const result = await env.rpc.approveDappSignature(env.requestId, true);
        expect(result.success).toBe(false);
        expect(env.createSigningSession).not.toHaveBeenCalled();
    });

    it("clears chrome.notifications for the requestId on either outcome", async () => {
        await env.rpc.approveDappSignature(env.requestId, true);
        const clearMock = (global as any).chrome.notifications.clear;
        expect(clearMock).toHaveBeenCalledWith(
            `mpc-dapp-sig:${env.requestId}`,
        );
    });
});
