//! Password Prompt Component — captures the wallet-encryption password
//! before DKG starts.
//!
//! **Substep 1.3**: real two-field input with confirm + validation.
//! - Password + confirm (both masked with `•`)
//! - Tab / BackTab to switch fields
//! - Enter to submit (runs validation; emits [`Message::SubmitPassword`]
//!   only on success)
//! - Esc to cancel ([`Message::NavigateBack`])
//!
//! Validation (inline error displayed on the screen, no modal):
//! - Password must be at least [`MIN_PASSWORD_LEN`] characters
//! - Confirm must exactly match password
//!
//! The password only encrypts **this device's** key share — every
//! participant picks their own. The copy on the screen says so, so a new
//! user doesn't assume they have to coordinate a shared secret out of
//! band.

use crate::elm::components::{Id, MpcWalletComponent, UserEvent};
use crate::elm::message::Message;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent};
use tuirealm::props::Props;
use tuirealm::ratatui::Frame;
use tuirealm::state::State;

const MIN_PASSWORD_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Password,
    Confirm,
}

impl Field {
    fn next(self) -> Self {
        match self {
            Field::Password => Field::Confirm,
            Field::Confirm => Field::Password,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordPromptComponent {
    props: Props,
    password: String,
    confirm: String,
    focused_field: Field,
    /// Set when the most recent Enter failed validation. Cleared as soon
    /// as the user types anything (stale errors are worse than none).
    error: Option<String>,
    focused: bool,
}

impl Default for PasswordPromptComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordPromptComponent {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            password: String::new(),
            confirm: String::new(),
            focused_field: Field::Password,
            error: None,
            focused: false,
        }
    }

    /// Validation rules. Returns `Err` with the user-visible message so
    /// the `on` handler can stash it in `self.error` verbatim. Centralised
    /// here so the rules stay in one place and are easily unit-tested.
    fn validate(&self) -> Result<(), String> {
        if self.password.len() < MIN_PASSWORD_LEN {
            return Err(format!(
                "Password must be at least {MIN_PASSWORD_LEN} characters"
            ));
        }
        if self.password != self.confirm {
            return Err("Confirm does not match password".to_string());
        }
        Ok(())
    }

    fn focused_buf_mut(&mut self) -> &mut String {
        match self.focused_field {
            Field::Password => &mut self.password,
            Field::Confirm => &mut self.confirm,
        }
    }

    /// Render a single field row: `label` on the left, masked content
    /// on the right. The focused field gets a yellow border, unfocused
    /// is gray. An underscore caret is appended to the masked text so
    /// the user can see where typing goes next.
    fn render_field_row(
        &self,
        frame: &mut Frame,
        area: Rect,
        label: &str,
        value: &str,
        is_focused: bool,
    ) {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

        let masked: String = value.chars().map(|_| '•').collect();
        let caret = if is_focused { "_" } else { "" };
        let content = format!("{masked}{caret}");

        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let widget = Paragraph::new(content).block(
            Block::default()
                .title(label)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        );
        frame.render_widget(widget, area);
    }
}

impl Component for PasswordPromptComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

        // Outer frame — a container box with the screen title so the user
        // knows which screen they're on even after switching focus.
        let outer = Block::default()
            .title(" Set Wallet Password ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        // Vertical layout: explainer (3 rows) / password (3) / confirm (3)
        // / error (2) / hints (1). Using concrete Lengths so the layout is
        // the same at any terminal size as long as it's taller than ~12
        // rows — smaller than that and the widgets clip, which is fine for
        // a password screen (not expected on sub-laptop terminals).
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // explainer
                Constraint::Length(3), // password field
                Constraint::Length(3), // confirm field
                Constraint::Length(2), // error line (if any)
                Constraint::Min(1),    // hints (bottom)
            ])
            .split(inner);

        // Explainer
        let explainer = Paragraph::new(
            "This password encrypts this device's key share in the local keystore.\n\
             Each participant picks their own — no coordination required.",
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
        frame.render_widget(explainer, rows[0]);

        self.render_field_row(
            frame,
            rows[1],
            " Password ",
            &self.password,
            self.focused_field == Field::Password,
        );
        self.render_field_row(
            frame,
            rows[2],
            " Confirm ",
            &self.confirm,
            self.focused_field == Field::Confirm,
        );

        // Error line — only renders when present.
        if let Some(ref msg) = self.error {
            let error_para = Paragraph::new(msg.as_str())
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
            frame.render_widget(error_para, rows[3]);
        }

        // Bottom hints — keep them discoverable since there's no menu bar
        // on this screen.
        let hints = Paragraph::new("Enter = Submit    Tab = Next field    Esc = Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hints, rows[4]);
    }

    fn query<'a>(
        &'a self,
        attr: tuirealm::props::Attribute,
    ) -> Option<tuirealm::props::QueryResult<'a>> {
        self.props.get_for_query(attr)
    }

    fn attr(&mut self, attr: tuirealm::props::Attribute, value: tuirealm::props::AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::NoChange
    }
}

impl AppComponent<Message, UserEvent> for PasswordPromptComponent {
    fn on(&mut self, event: &Event<UserEvent>) -> Option<Message> {
        let Event::Keyboard(KeyEvent { code, .. }) = event else {
            return None;
        };

        match code {
            Key::Esc => Some(Message::NavigateBack),

            Key::Tab | Key::BackTab => {
                // Shift+Tab in tuirealm is its own code (BackTab), but for a
                // two-field form next vs previous are identical — just flip.
                self.focused_field = self.focused_field.next();
                // Don't emit a Message — focus change is component-local.
                None
            }

            Key::Char(c) => {
                // Any typing clears the stale error; otherwise fixing the
                // password still shows the old complaint until Enter.
                self.error = None;
                self.focused_buf_mut().push(*c);
                None
            }

            Key::Backspace => {
                self.error = None;
                self.focused_buf_mut().pop();
                None
            }

            Key::Enter => match self.validate() {
                Ok(()) => Some(Message::SubmitPassword {
                    value: self.password.clone(),
                }),
                Err(msg) => {
                    self.error = Some(msg);
                    None
                }
            },

            _ => None,
        }
    }
}

impl MpcWalletComponent for PasswordPromptComponent {
    fn id(&self) -> Id {
        Id::PasswordPrompt
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

// -----------------------------------------------------------------
// Unit tests — event handling + validation, no rendering.
// -----------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::event::KeyModifiers;

    fn key(code: Key) -> Event<UserEvent> {
        Event::Keyboard(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn type_str(c: &mut PasswordPromptComponent, s: &str) {
        for ch in s.chars() {
            let _ = c.on(&key(Key::Char(ch)));
        }
    }

    #[test]
    fn typing_in_password_field_accumulates_chars() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "hunter2!");
        assert_eq!(c.password, "hunter2!");
        assert_eq!(c.confirm, "");
    }

    #[test]
    fn tab_switches_to_confirm_field() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "abc");
        c.on(&key(Key::Tab));
        type_str(&mut c, "xyz");
        assert_eq!(c.password, "abc", "Tab must not mutate the password field");
        assert_eq!(c.confirm, "xyz", "after Tab, characters should flow into confirm");
    }

    #[test]
    fn backtab_also_switches_fields() {
        // Two-field form: next and prev are the same flip, so BackTab
        // must behave identically to Tab.
        let mut c = PasswordPromptComponent::new();
        assert_eq!(c.focused_field, Field::Password);
        c.on(&key(Key::BackTab));
        assert_eq!(c.focused_field, Field::Confirm);
    }

    #[test]
    fn backspace_deletes_from_focused_field() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "abcd");
        c.on(&key(Key::Backspace));
        assert_eq!(c.password, "abc");
    }

    #[test]
    fn enter_with_short_password_sets_inline_error_and_does_not_submit() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "short"); // 5 chars < 8
        let out = c.on(&key(Key::Enter));
        assert!(out.is_none(), "short password must not emit SubmitPassword");
        assert!(
            c.error.as_deref().unwrap_or("").contains("at least"),
            "short-password error should mention the length requirement; got {:?}",
            c.error
        );
    }

    #[test]
    fn enter_with_mismatched_confirm_sets_error_and_does_not_submit() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "longenoughpw");
        c.on(&key(Key::Tab));
        type_str(&mut c, "different");
        let out = c.on(&key(Key::Enter));
        assert!(out.is_none(), "mismatched confirm must not emit SubmitPassword");
        assert!(
            c.error.as_deref().unwrap_or("").contains("match"),
            "mismatch error should mention matching; got {:?}",
            c.error
        );
    }

    #[test]
    fn enter_with_valid_matching_password_emits_submit_with_cleartext() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "correcthorsebatterystaple");
        c.on(&key(Key::Tab));
        type_str(&mut c, "correcthorsebatterystaple");
        match c.on(&key(Key::Enter)) {
            Some(Message::SubmitPassword { value }) => {
                assert_eq!(value, "correcthorsebatterystaple");
            }
            other => panic!("expected Some(SubmitPassword), got {:?}", other),
        }
        assert!(
            c.error.is_none(),
            "error must be cleared on successful submit; got {:?}",
            c.error
        );
    }

    #[test]
    fn typing_clears_stale_error() {
        let mut c = PasswordPromptComponent::new();
        type_str(&mut c, "short");
        c.on(&key(Key::Enter)); // triggers error
        assert!(c.error.is_some());
        c.on(&key(Key::Char('x')));
        assert!(
            c.error.is_none(),
            "typing after an error should clear it — stale errors are worse than none"
        );
    }

    #[test]
    fn esc_emits_navigate_back() {
        let mut c = PasswordPromptComponent::new();
        assert_eq!(c.on(&key(Key::Esc)), Some(Message::NavigateBack));
    }
}
