# Keyboard Navigation Guide

Per-screen keybinding reference for the MPC Wallet TUI. All
keybindings are hardcoded in the tui-realm `Component::on` handlers
under `apps/tui-node/src/elm/components/`; there is no runtime
config file or remap mechanism.

Earlier drafts of this guide described ~40 different shortcuts:
`hjkl` vim navigation, `Ctrl+Q`/`Ctrl+R`/`Ctrl+H`/`Ctrl+L` globals,
single-letter quick keys (`n`/`j`/`w`/`s`/`i`/`e`/`d`/`r`), command
mode (`:`), search mode (`/`), vim-style macros (`q`, `@`),
bookmarks (`m`, `'`), number-key menu selection (`1`-`6`), and a
`?` help overlay. **None of those are wired up** (verified by
grepping `KeyCode::` / `Key::Char(...)` patterns across
`src/elm/components/` and `src/elm/update.rs`). This doc now
reflects what actually works.

## Global keys

These work everywhere inside the TUI:

| Key     | Action                  |
|---------|-------------------------|
| `↑` `↓` | Move selection / focus  |
| `Enter` | Confirm selection       |
| `Esc`   | Back / cancel           |
| `Tab`   | Move focus within screen |

Quit the app with `Ctrl+C` from the terminal — there's no dedicated
in-app quit key beyond the system-level interrupt.

## Per-screen behaviours

### Main menu (`main_menu.rs`)

- `↑` / `↓` — move between menu items.
- `Enter` — open the selected item.

Menu items depend on state — see the [User Guide](./guides/USER_GUIDE.md)
for the full list.

### Mode selection (`mode_selection.rs`)

- `↑` / `↓` — toggle between Online / Offline.
- `Enter` — confirm.

### Curve selection

- `↑` / `↓` — toggle between secp256k1 (Ethereum) / ed25519 (Solana).
- `Enter` — confirm.

### Threshold config (`threshold_config.rs`)

- `↑` / `↓` — increment / decrement the currently-focused field
  (total participants or threshold).
- `Tab` — switch focus between the two fields.
- `Enter` — submit and start DKG.

### Wallet list (`wallet_list.rs`)

- `↑` / `↓` — move selection between wallets.
- `Enter` — open wallet detail for the selected wallet.
- `Esc` — back to main menu.

### Wallet detail (`wallet_detail.rs`)

- `Enter` on the "Sign" row — open the SignTransaction screen.
- `Esc` — back to wallet list.

### Password prompt (`password_prompt.rs`)

- Any printable character — appends to the current password draft.
- `Backspace` — delete the last character.
- `Tab` — toggle focus between the Password and Confirm fields
  (when both are required).
- `Enter` — submit.
- `Esc` — cancel and clear the draft.

The draft buffer lives on `Model.wallet_state` and is zeroed on
every exit path.

### Sign transaction / sign message (`sign_transaction.rs`)

Single-field message input (the scope note in the component
docstring: "Phase C scope: message-only field").

- Any printable character — appends to the message-to-sign draft.
- `Backspace` — delete the last character.
- `Enter` — submit and start the signing ceremony.
- `Esc` — cancel.

### DKG / signing progress (`dkg_progress.rs`)

- `Esc` — cancel the ceremony (where that's allowed by the state
  machine).
- Otherwise view-only — the screen self-advances as messages
  arrive over the mesh.

### Join session (`join_session.rs`)

- `↑` / `↓` — move between announced sessions.
- `Enter` — join the selected session.
- `Esc` — back to main menu.

### Modal dialogs (`modal.rs`)

- `Enter` — confirm / acknowledge.
- `Esc` — cancel / dismiss.

## Customisation

Keybindings are hardcoded in the Elm update layer (`src/elm/update.rs`)
and the per-screen components in `src/elm/components/`. To remap a
key, edit the corresponding `KeyCode` match arm and rebuild.

Proposed future work: load keybinding overrides from a TOML file at
startup. Until that lands, the code is the authoritative source.

## Related reference

- [`KEYBOARD_HANDLING_GUIDE.md`](./KEYBOARD_HANDLING_GUIDE.md) —
  for developers adding a new screen / component. Covers the
  `KeyModifiers::NONE` footgun and the message-routing patterns
  each component must follow.
