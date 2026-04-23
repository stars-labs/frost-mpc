<script lang="ts">
    /**
     * Ext-1b: minimal "Create Wallet" form. Fires
     * CREATE_DKG_WALLET over chrome.runtime. Background announces via
     * `announce_session` — any TUI or extension peer on the same
     * signal server can discover and join.
     *
     * This is the DKG *initiator* flow only. Joiner side is Ext-1e.
     */
    import { createEventDispatcher } from "svelte";
    import { MESSAGE_TYPES } from "@mpc-wallet/types/messages";

    export let deviceId: string = "";
    export let wsConnected: boolean = false;

    const dispatch = createEventDispatcher<{
        created: { sessionId: string };
        cancel: void;
    }>();

    // Defaults match TUI's ThresholdConfig screen starting values.
    let total = 3;
    let threshold = 2;
    let curve: "secp256k1" | "ed25519" = "secp256k1";
    let walletName = "";
    let submitting = false;
    let errorMessage = "";

    // Keep threshold clamped to [2, total] — threshold < 2 defeats the
    // purpose of multisig, and > total is nonsense. Mirror TUI.
    $: if (threshold > total) threshold = total;
    $: if (threshold < 2) threshold = 2;
    $: if (total < 2) total = 2;
    $: if (total > 10) total = 10;

    async function handleSubmit() {
        errorMessage = "";
        if (!wsConnected) {
            errorMessage =
                "Signal server not connected. Check Settings → Signal Server.";
            return;
        }
        submitting = true;
        try {
            const response = await chrome.runtime.sendMessage({
                type: MESSAGE_TYPES.CREATE_DKG_WALLET,
                name: walletName.trim() || undefined,
                total,
                threshold,
                curve,
            });
            if (response?.success && response.sessionId) {
                dispatch("created", { sessionId: response.sessionId });
            } else {
                errorMessage =
                    response?.error ?? "Failed to create wallet (no error returned)";
            }
        } catch (e) {
            errorMessage = (e as Error).message ?? String(e);
        } finally {
            submitting = false;
        }
    }
</script>

<div class="rounded border border-gray-200 bg-white p-4">
    <h2 class="mb-3 text-lg font-semibold">Create MPC Wallet</h2>
    <p class="mb-4 text-xs text-gray-600">
        Your device ({deviceId || "unregistered"}) will initiate a DKG ceremony.
        Other participants (TUI nodes or other extensions) join by discovering
        the session over the signal server.
    </p>

    <label class="mb-3 block">
        <span class="mb-1 block text-sm font-medium">Wallet name (optional)</span>
        <input
            type="text"
            bind:value={walletName}
            placeholder="e.g. Treasury 2-of-3"
            class="w-full rounded border px-2 py-1 text-sm"
            disabled={submitting}
        />
    </label>

    <div class="mb-3 grid grid-cols-2 gap-3">
        <label class="block">
            <span class="mb-1 block text-sm font-medium">Total participants (N)</span>
            <input
                type="number"
                min="2"
                max="10"
                bind:value={total}
                class="w-full rounded border px-2 py-1 text-sm"
                disabled={submitting}
            />
        </label>
        <label class="block">
            <span class="mb-1 block text-sm font-medium">Threshold (K)</span>
            <input
                type="number"
                min="2"
                max={total}
                bind:value={threshold}
                class="w-full rounded border px-2 py-1 text-sm"
                disabled={submitting}
            />
        </label>
    </div>

    <label class="mb-4 block">
        <span class="mb-1 block text-sm font-medium">Curve</span>
        <select
            bind:value={curve}
            class="w-full rounded border px-2 py-1 text-sm"
            disabled={submitting}
        >
            <option value="secp256k1">secp256k1 (Ethereum / EVM)</option>
            <option value="ed25519">ed25519 (Solana)</option>
        </select>
    </label>

    <p class="mb-3 text-xs text-gray-500">
        This announces <code>{threshold}-of-{total}</code>
        {curve} to the signal server. Joiners will see this session in
        their own "Join Session" tab.
    </p>

    {#if errorMessage}
        <div class="mb-3 rounded border border-red-200 bg-red-50 px-2 py-1 text-xs text-red-700">
            {errorMessage}
        </div>
    {/if}

    <div class="flex gap-2">
        <button
            type="button"
            class="flex-1 rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:bg-blue-300"
            on:click={handleSubmit}
            disabled={submitting || !wsConnected}
        >
            {submitting ? "Announcing…" : "Announce DKG Session"}
        </button>
        <button
            type="button"
            class="rounded border border-gray-300 px-3 py-2 text-sm hover:bg-gray-50"
            on:click={() => dispatch("cancel")}
            disabled={submitting}
        >
            Cancel
        </button>
    </div>
</div>
