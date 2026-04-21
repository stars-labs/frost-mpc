//! Snapshot-style rendering tests for Elm components.
//!
//! Uses `ratatui::backend::TestBackend` — an in-memory backend that renders
//! into a `Buffer` instead of a real terminal. The component's `view` method
//! paints into a `Frame`, we then flatten the buffer's cells back into a
//! newline-separated string and do `contains`-style assertions.
//!
//! Why substring rather than exact buffer equality: exact snapshot assertions
//! are extremely brittle for ratatui layouts (one spacing tweak invalidates
//! every test). Substring checks target the **invariants we care about** —
//! "when `dkg_round` is Round 1, the string 'Round 1' appears somewhere on
//! screen" — and stay stable across cosmetic changes.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

use tui_node::elm::components::DKGProgressComponent;
use tui_node::elm::message::DKGRound;
use tuirealm::component::Component;

/// Flatten every cell on every row into a single string, one row per line.
/// Ratatui cells can be wider than one grapheme in theory but for our
/// ASCII+emoji UI `symbol()` always gives the visible text.
fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut out = String::with_capacity((area.width as usize + 1) * area.height as usize);
    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                out.push_str(cell.symbol());
            }
        }
        out.push('\n');
    }
    out
}

/// Render the component into a fresh `TestBackend` and return the flattened
/// text. 120×40 is big enough for the DKG Progress layout (header +
/// participants list + progress bar + action row) without clipping.
fn render_dkg_progress_with_round(round: DKGRound) -> String {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("TestBackend::Terminal");

    // Realistic-ish session: 2-of-3, short session id so it fits on one line.
    let mut component = DKGProgressComponent::new("dkg-smoke-01".to_string(), 3, 2);
    component.set_round(round);
    // Make websocket look connected so the render path doesn't show the
    // "WebSocket disconnected" red banner instead of the round-specific
    // status line we want to assert on.
    component.set_websocket_connected(true);

    terminal
        .draw(|frame| {
            let area = frame.area();
            component.view(frame, area);
        })
        .expect("TestBackend draw must succeed");

    buffer_to_string(terminal.backend().buffer())
}

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context}: expected rendered UI to contain {needle:?}\n\
         --- rendered (first 800 chars) ---\n{}",
        &haystack[..haystack.len().min(800)]
    );
}

// -----------------------------------------------------------------
// Round-label invariants across the DKG lifecycle
// -----------------------------------------------------------------
#[test]
fn renders_initialization_round_label() {
    let rendered = render_dkg_progress_with_round(DKGRound::Initialization);
    assert_contains(
        &rendered,
        "Initialization",
        "Initialization round should render its header label",
    );
}

#[test]
fn renders_round1_label_and_progress() {
    let rendered = render_dkg_progress_with_round(DKGRound::Round1);
    assert_contains(&rendered, "Round1", "Round 1 header label (enum Debug form)");
    // Progress bar uses a different label style (`Generating commitments...`).
    assert_contains(
        &rendered,
        "Generating commitments",
        "Round 1 should render the round-1-specific progress caption",
    );
}

#[test]
fn renders_round2_label_and_progress() {
    let rendered = render_dkg_progress_with_round(DKGRound::Round2);
    assert_contains(&rendered, "Round2", "Round 2 header label");
    assert_contains(
        &rendered,
        "Exchanging shares",
        "Round 2 should render the round-2-specific progress caption",
    );
}

#[test]
fn renders_complete_at_100_percent() {
    // This is the exact regression we hit: Finalization capped at 95% and
    // read "Finalizing DKG..." forever. Complete must read 100% with a
    // "done" caption so the user knows the protocol actually finished.
    let rendered = render_dkg_progress_with_round(DKGRound::Complete);
    assert_contains(&rendered, "Complete", "terminal round label");
    assert_contains(&rendered, "100%", "Complete must render 100% in the progress bar");
    assert_contains(
        &rendered,
        "DKG complete",
        "Complete must render a user-visible 'done' caption",
    );
}

#[test]
fn renders_finalization_at_95_percent() {
    // Finalization is an intermediate state (part3 running). Keep it
    // distinct from Complete so a stuck part3 doesn't masquerade as done.
    let rendered = render_dkg_progress_with_round(DKGRound::Finalization);
    assert_contains(&rendered, "Finalization", "Finalization header label");
    assert_contains(
        &rendered,
        "95%",
        "Finalization must render 95%, not 100% — 100% is reserved for Complete",
    );
}

// -----------------------------------------------------------------
// PasswordPromptComponent (Substep 1.3: real two-field input)
// -----------------------------------------------------------------
fn render_password_prompt() -> String {
    use tui_node::elm::components::PasswordPromptComponent;

    let backend = TestBackend::new(120, 20);
    let mut terminal = Terminal::new(backend).expect("TestBackend::Terminal");
    let mut component = PasswordPromptComponent::new();

    terminal
        .draw(|frame| {
            let area = frame.area();
            component.view(frame, area);
        })
        .expect("TestBackend draw must succeed");

    buffer_to_string(terminal.backend().buffer())
}

#[test]
fn password_prompt_renders_title_and_both_fields() {
    let rendered = render_password_prompt();
    assert_contains(
        &rendered,
        "Set Wallet Password",
        "screen title must render so users know which screen they're on",
    );
    assert_contains(&rendered, "Password", "password field label must render");
    assert_contains(&rendered, "Confirm", "confirm field label must render");
}

#[test]
fn password_prompt_renders_keybinding_hints() {
    let rendered = render_password_prompt();
    // The bottom hint line lists the three interactions. Assert all three —
    // a user landing here with no history should see their options.
    assert_contains(&rendered, "Enter", "submit hint must render");
    assert_contains(&rendered, "Tab", "field-switch hint must render");
    assert_contains(&rendered, "Esc", "cancel hint must render");
}

#[test]
fn password_prompt_explains_password_is_local_only() {
    // Critical UX: the password is *not* a shared secret. If this copy
    // drifts or disappears, users may attempt to coordinate a shared
    // password out-of-band, which is both unnecessary and a security
    // anti-pattern (shared secrets leak faster). Pin the copy here.
    let rendered = render_password_prompt();
    assert_contains(
        &rendered,
        "device",
        "explainer should mention 'device' so the local-only semantics are visible",
    );
    assert_contains(
        &rendered,
        "their own",
        "explainer should make clear each participant picks their own password",
    );
}
