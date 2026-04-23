import { defineConfig } from 'wxt';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  srcDir: 'src',
  modules: ['@wxt-dev/module-svelte'],
  vite: () => ({
    plugins: [
      wasm(),
      topLevelAwait(),
      tailwindcss(),
    ],
  }),
  manifest: {
    name: 'Browser Wallet',
    description: 'A secure browser extension wallet for Ethereum',
    version: '1.0.0',
    permissions: ['storage', 'tabs', 'activeTab', 'offscreen'],
    host_permissions: [
      'https://*/*',
      // Allow both signal servers. `xiongchenyu.dpdns.org` is the
      // new default (matches TUI's `model.rs::WEBSOCKET_URL`);
      // `auto-life.tech` is the legacy endpoint left in place so
      // users who set a manual override via chrome.storage.local
      // ['signalServerUrl'] still have host permission for it.
      'wss://xiongchenyu.dpdns.org/*',
      'wss://auto-life.tech/*'
    ],
    icons: {
      "16": "assets/icon-16.png",
      "32": "assets/icon-32.png",
      "48": "assets/icon-48.png",
      "128": "assets/icon-128.png"
    },
    action: {
      default_popup: "popup.html",
      default_icon: {
        "16": "assets/icon-16.png",
        "32": "assets/icon-32.png"
      }
    },
    // options_ui: {
    //   page: "options.html",
    //   open_in_tab: true
    // },
    content_scripts: [
      {
        matches: ['<all_urls>'],
        js: ['content-scripts/content.js'],
        run_at: 'document_start'
      }
    ],
    background: {
      service_worker: "entrypoints/background/index.ts",
      type: "module"
    },
    content_security_policy: {
      "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self';"
    },
    web_accessible_resources: [
      {
        resources: ['injected.js'],
        matches: ['<all_urls>']
      }
    ],

    default_locale: 'en',
  },
});