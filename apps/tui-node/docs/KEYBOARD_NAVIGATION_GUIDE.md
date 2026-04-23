# MPC Wallet TUI - Keyboard Navigation & Shortcuts Guide

## Quick Reference Card

```
╔═══════════════════════════════════════════════════════════════╗
║                    MPC WALLET TUI CONTROLS                       ║
╠═══════════════════╤═══════════════════════════════════════════╣
║ Navigation        │ ↑↓←→ or hjkl  Navigate menus/fields         ║
║                   │ Enter         Select/Confirm                 ║
║                   │ Esc           Go back/Cancel                 ║
║                   │ Tab           Next field                     ║
║                   │ Shift+Tab     Previous field                 ║
╠═══════════════════╪═══════════════════════════════════════════╣
║ Global            │ Ctrl+Q        Quit application               ║
║                   │ Ctrl+R        Refresh screen                 ║
║                   │ Ctrl+H        Go to home                     ║
║                   │ ?             Show help                      ║
╠═══════════════════╪═══════════════════════════════════════════╣
║ Quick Actions     │ n             New wallet                     ║
║                   │ j             Join session                   ║
║                   │ w             Wallets list                   ║
║                   │ s             Sign transaction               ║
║                   │ /             Search                         ║
╚═══════════════════╧═══════════════════════════════════════════╝
```

## Detailed Navigation Guide

### 1. Main Menu Navigation

The main menu is your starting point. Navigate with arrow keys or vim-style keys.

```
Main Menu
├─[1] Create New Wallet     (Enter or 1)
├─[2] Join Session          (Enter or 2)
├─[3] Manage Wallets        (Enter or 3)
├─[4] Sign Transaction      (Enter or 4)
├─[5] Settings              (Enter or 5)
└─[6] Help & About          (Enter or 6)

Quick Keys: n=new, j=join, w=wallets, s=sign
```

**Tips:**
- Use number keys (1-6) for direct selection
- Arrow keys wrap around (down from bottom goes to top)
- Quick keys work from any screen

### 2. List Navigation

When viewing lists (wallets, sessions, etc.):

| Key | Action | Description |
|-----|--------|-------------|
| `↑/k` | Previous item | Move selection up |
| `↓/j` | Next item | Move selection down |
| `Home/g` | First item | Jump to top |
| `End/G` | Last item | Jump to bottom |
| `Page Up` | Previous page | Scroll up one page |
| `Page Down` | Next page | Scroll down one page |
| `Enter` | Select | Open selected item |
| `d` | Delete | Delete selected (with confirmation) |
| `e` | Edit | Edit selected item |
| `r` | Rename | Rename selected item |
| `/` | Search | Filter list by search term |
| `Esc` | Clear/Back | Clear search or go back |

### 3. Form Navigation

When filling out forms (wallet creation, settings, etc.):

```
┌─ Create Wallet ──────────────────────┐
│ Name:     [________________]   ←Tab→ │
│ Mode:     [▼ Online        ]   Space │
│ Curve:    [▼ Secp256k1     ]   Space │
│                                       │
│ Participants: [3] ←→ to adjust       │
│ Threshold:    [2] ←→ to adjust       │
│                                       │
│ [Cancel] Esc    [Next] Enter         │
└───────────────────────────────────────┘
```

**Form Controls:**
- `Tab` / `Shift+Tab`: Navigate between fields
- `Space`: Open dropdown or toggle checkbox
- `←/→`: Adjust numeric values or select options
- `Enter`: Submit form or proceed to next step
- `Esc`: Cancel and return to previous screen

### 4. Modal Dialogs

Modals appear for confirmations, errors, and inputs:

```
┌─ Confirm ────────────────────────┐
│                                  │
│  Delete wallet "Alice's Wallet"? │
│                                  │
│  This action cannot be undone.   │
│                                  │
│    [No] ←→ [Yes]                 │
│     Esc     Enter                │
└──────────────────────────────────┘
```

**Modal Controls:**
- `←/→` or `Tab`: Switch between buttons
- `Enter`: Confirm selected option
- `Esc`: Cancel/close modal
- `y/n`: Quick confirm/deny for yes/no dialogs

### 5. Progress Screens

During operations like DKG or signing:

```
┌─ DKG Progress ───────────────────┐
│                                  │
│  Round 2 of 3                    │
│  ████████████░░░░░░  65%         │
│                                  │
│  Participants:                   │
│  • Alice    ✓ Ready              │
│  • Bob      ⟳ Processing...      │
│  • Charlie  ✓ Ready              │
│                                  │
│  [Cancel] Esc  [Details] d       │
└──────────────────────────────────┘
```

**Progress Controls:**
- `d`: Show detailed logs
- `p`: Pause operation (if supported)
- `r`: Retry failed operation
- `Esc`: Cancel operation (with confirmation)

## Screen-Specific Shortcuts

### Wallet List Screen

| Key | Action |
|-----|--------|
| `n` | Create new wallet |
| `i` | Import wallet |
| `e` | Export selected wallet |
| `d` | Delete selected wallet |
| `Enter` | View wallet details |
| `r` | Refresh wallet list |
| `s` | Sort wallets (cycle through: name/date/balance) |
| `/` | Search wallets |

### Wallet Details Screen

| Key | Action |
|-----|--------|
| `s` | Sign transaction |
| `e` | Export wallet |
| `b` | Show balance details |
| `h` | Show transaction history |
| `a` | Show addresses |
| `c` | Copy address to clipboard |
| `q` | Generate QR code |
| `Esc` | Back to wallet list |

### Session Discovery Screen

| Key | Action |
|-----|--------|
| `Enter` | Join selected session |
| `i` | Show session info |
| `r` | Refresh session list |
| `f` | Filter by type (DKG/Signing) |
| `n` | Create new session |
| `/` | Search sessions |

### Settings Screen

| Key | Action |
|-----|--------|
| `Tab` | Next setting category |
| `Shift+Tab` | Previous category |
| `Enter` | Edit selected setting |
| `r` | Reset to defaults |
| `s` | Save settings |
| `Esc` | Discard changes and exit |

## Advanced Navigation

### Command Mode

Press `:` to enter command mode (similar to vim):

```
:w              Save current state
:q              Quit application
:wq             Save and quit
:new wallet     Create new wallet
:join <id>      Join session by ID
:connect <url>  Connect to WebSocket URL
:export <id>    Export wallet by ID
:help <topic>   Show help for topic
```

### Search Mode

Press `/` to enter search mode:

```
/alice          Search for "alice"
n               Next match
N               Previous match
Esc             Exit search mode
```

### Quick Jump

Use number keys for quick navigation:

- `1-9`: Jump to item 1-9 in current list
- `0`: Jump to item 10
- `gg`: Go to first item
- `G`: Go to last item
- `5j`: Move down 5 items
- `3k`: Move up 3 items

## Vim Mode (Optional)

Enable vim mode in settings for additional commands:

### Normal Mode
```
h j k l         Navigate (left, down, up, right)
w               Next word/field
b               Previous word/field
0               Beginning of line
$               End of line
gg              Top of screen
G               Bottom of screen
/               Search forward
?               Search backward
n               Next search result
N               Previous search result
```

### Visual Mode
```
v               Start visual selection
V               Select entire line
Ctrl+v          Block selection
y               Copy selection
d               Delete selection
```

## Accessibility Features

### Screen Reader Support

The TUI provides screen reader compatibility:

- All elements have descriptive labels
- Status changes are announced
- Navigation provides audio feedback (if enabled)

### High Contrast Mode

Toggle with `Ctrl+T`:
- Increases contrast for better visibility
- Larger text indicators
- Enhanced borders and separators

### Keyboard-Only Operation

Every feature is accessible via keyboard:
- No mouse required
- All actions have keyboard shortcuts
- Tab navigation through all elements

## Customization

Keybindings are hardcoded in the Elm update layer (`src/elm/update.rs`
and the per-screen components in `src/elm/components/`) — there is no
runtime configuration file. To remap a key, edit the corresponding
`KeyCode` match arms and rebuild.

Proposed future work: load keybinding overrides from a TOML file at
startup. See the existing hotkey definitions in code as the authoritative
source until that lands.

### Disable Shortcuts

To disable specific shortcuts:

```toml
[disabled]
shortcuts = ["Ctrl+c", "Ctrl+z"]
```

## Tips & Tricks

### 1. Efficient Navigation

- **Learn the quick keys**: `n`, `j`, `w`, `s` work from anywhere
- **Use number keys**: Direct selection in menus
- **Master Tab**: Fastest way through forms
- **Remember Esc**: Universal "go back" key

### 2. Power User Features

- **Command mode**: `:` for complex operations
- **Batch operations**: Select multiple items with `Space`
- **Macros**: Record with `q`, replay with `@`
- **Bookmarks**: Mark positions with `m`, jump with `'`

### 3. Troubleshooting Navigation

**Keys not working?**
- Check terminal emulator settings
- Verify TERM environment variable
- Try different terminal (recommend: Alacritty, iTerm2, Windows Terminal)

**Slow response?**
- Reduce terminal buffer size
- Disable animations in settings
- Check network latency for remote sessions

**Display issues?**
- Reset terminal: `reset` or `Ctrl+L`
- Check font supports box-drawing characters
- Ensure UTF-8 encoding

## Quick Start Tutorial

### First Time Setup

1. **Launch**: Start the application
2. **Navigate**: Use `↓` to highlight "Create New Wallet"
3. **Select**: Press `Enter`
4. **Configure**: Use `Tab` to move through options
5. **Confirm**: Press `Enter` to proceed
6. **Complete**: Follow on-screen prompts

### Daily Operations

**Morning Routine:**
1. `w` - Check wallets
2. `/` - Search for specific wallet
3. `Enter` - View details
4. `Esc` - Back to list

**Signing Transaction:**
1. `s` - Start signing
2. Select wallet with arrows
3. `Enter` - Confirm
4. Review details
5. `y` - Approve

## Reference Card (Print-Friendly)

```
┌─────────────────────────────────┬─────────────────────────────────┐
│ NAVIGATION                      │ ACTIONS                         │
├─────────────────────────────────┼─────────────────────────────────┤
│ ↑/k     Move up                 │ n       New wallet              │
│ ↓/j     Move down               │ j       Join session            │
│ ←/h     Move left               │ w       Wallet list             │
│ →/l     Move right              │ s       Sign transaction        │
│ Enter   Select/Confirm          │ i       Import                  │
│ Esc     Back/Cancel             │ e       Export                  │
│ Tab     Next field              │ d       Delete                  │
│ Space   Toggle/Select           │ r       Refresh/Rename          │
├─────────────────────────────────┼─────────────────────────────────┤
│ GLOBAL                          │ SEARCH & COMMAND                │
├─────────────────────────────────┼─────────────────────────────────┤
│ Ctrl+Q  Quit                    │ /       Search mode             │
│ Ctrl+R  Refresh                 │ :       Command mode            │
│ Ctrl+H  Home                    │ ?       Help                    │
│ Ctrl+L  Clear screen            │ n/N     Next/Previous match     │
└─────────────────────────────────┴─────────────────────────────────┘
```

---

*For more help, press `?` within the application or visit the documentation.*