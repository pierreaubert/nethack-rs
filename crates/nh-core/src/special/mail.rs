//! Mail system (mail.c)
//!
//! Implements the mail daemon delivery system from NetHack.
//! In the original game, this would check for actual system mail,
//! but in this implementation it provides a framework for in-game
//! mail delivery events.
//!
//! The mail daemon is a special monster that appears to deliver
//! a "scroll of mail" to the player, then disappears.

use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::{Object, ObjectClass, ObjectId};
use crate::rng::GameRng;

/// Monster type index for mail daemon (from nh-data)
const PM_MAIL_DAEMON: i16 = 332; // MonsterType::MailDaemon equivalent

/// Object type index for scroll of mail (from nh-data)
const SCR_MAIL: i16 = 241; // Approximate index for scroll of mail

/// Mail delivery state
#[derive(Debug, Clone, Default)]
pub struct MailState {
    /// Whether mail delivery is enabled
    pub enabled: bool,
    /// Pending mail messages to deliver
    pub pending_mail: Vec<MailMessage>,
    /// Turn when last mail was delivered
    pub last_delivery_turn: u64,
    /// Minimum turns between deliveries
    pub delivery_cooldown: u64,
}

/// A mail message to be delivered
#[derive(Debug, Clone)]
pub struct MailMessage {
    /// Sender of the mail (displayed on scroll)
    pub from: String,
    /// Content of the mail
    pub content: String,
    /// Priority (higher = delivered sooner)
    pub priority: u8,
}

impl MailMessage {
    /// Create a new mail message
    pub fn new(from: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            content: content.into(),
            priority: 0,
        }
    }

    /// Create a high-priority mail message
    pub fn urgent(from: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            content: content.into(),
            priority: 10,
        }
    }
}

impl MailState {
    /// Create a new mail state with default settings
    pub fn new() -> Self {
        Self {
            enabled: true,
            pending_mail: Vec::new(),
            last_delivery_turn: 0,
            delivery_cooldown: 100, // At least 100 turns between deliveries
        }
    }

    /// Queue a mail message for delivery
    pub fn queue_mail(&mut self, message: MailMessage) {
        self.pending_mail.push(message);
        // Sort by priority (highest first)
        self.pending_mail.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Check if mail can be delivered this turn
    pub fn can_deliver(&self, current_turn: u64) -> bool {
        self.enabled
            && !self.pending_mail.is_empty()
            && current_turn >= self.last_delivery_turn + self.delivery_cooldown
    }

    /// Get the next mail message to deliver (without removing it)
    pub fn peek_next_mail(&self) -> Option<&MailMessage> {
        self.pending_mail.first()
    }

    /// Take the next mail message for delivery
    pub fn take_next_mail(&mut self) -> Option<MailMessage> {
        if self.pending_mail.is_empty() {
            None
        } else {
            Some(self.pending_mail.remove(0))
        }
    }

    /// Record that a delivery was made
    pub fn record_delivery(&mut self, turn: u64) {
        self.last_delivery_turn = turn;
    }
}

/// Result of attempting mail delivery
#[derive(Debug, Clone)]
pub enum MailDeliveryResult {
    /// Mail was successfully delivered
    Delivered {
        /// Message that was delivered
        message: MailMessage,
        /// Position where daemon appeared
        daemon_pos: (i8, i8),
    },
    /// No mail to deliver
    NoMail,
    /// Delivery on cooldown
    OnCooldown,
    /// Could not find valid position for daemon
    NoValidPosition,
    /// Mail delivery is disabled
    Disabled,
}

/// Attempt to deliver mail to the player
///
/// This will:
/// 1. Check if there's mail to deliver
/// 2. Find a valid position near the player for the mail daemon
/// 3. Create the mail daemon monster
/// 4. Create the scroll of mail object
/// 5. Return the delivery result
///
/// The caller is responsible for:
/// - Adding the daemon to the level
/// - Displaying appropriate messages
/// - Handling the daemon's disappearance after delivery
pub fn attempt_mail_delivery(
    mail_state: &mut MailState,
    level: &Level,
    player_x: i8,
    player_y: i8,
    current_turn: u64,
    rng: &mut GameRng,
) -> MailDeliveryResult {
    if !mail_state.enabled {
        return MailDeliveryResult::Disabled;
    }

    if mail_state.pending_mail.is_empty() {
        return MailDeliveryResult::NoMail;
    }

    if current_turn < mail_state.last_delivery_turn + mail_state.delivery_cooldown {
        return MailDeliveryResult::OnCooldown;
    }

    // Find a valid position near the player for the daemon
    let daemon_pos = find_daemon_position(level, player_x, player_y, rng);
    let Some((dx, dy)) = daemon_pos else {
        return MailDeliveryResult::NoValidPosition;
    };

    // Take the mail message
    let message = mail_state.take_next_mail().unwrap();
    mail_state.record_delivery(current_turn);

    MailDeliveryResult::Delivered {
        message,
        daemon_pos: (dx, dy),
    }
}

/// Find a valid position near the player for the mail daemon to appear
fn find_daemon_position(
    level: &Level,
    player_x: i8,
    player_y: i8,
    rng: &mut GameRng,
) -> Option<(i8, i8)> {
    // Try positions adjacent to the player
    let mut candidates = Vec::new();

    for dx in -2..=2 {
        for dy in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let x = player_x + dx;
            let y = player_y + dy;

            if level.is_valid_pos(x, y)
                && level.is_walkable(x, y)
                && level.monster_at(x, y).is_none()
            {
                candidates.push((x, y));
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Pick a random valid position
    let idx = rng.rn2(candidates.len() as u32) as usize;
    Some(candidates[idx])
}

/// Create a mail daemon monster
pub fn create_mail_daemon(x: i8, y: i8) -> Monster {
    let mut daemon = Monster::new(MonsterId::NONE, PM_MAIL_DAEMON, x, y);
    daemon.name = "mail daemon".to_string();
    daemon.state = MonsterState::peaceful();
    daemon.state.can_move = true;
    // Mail daemon has special properties
    daemon.hp = 1;
    daemon.hp_max = 1;
    daemon.level = 25; // High level so it's not easily killed
    daemon
}

/// Create a scroll of mail object
pub fn create_scroll_of_mail(message: &MailMessage) -> Object {
    let mut scroll = Object::new(ObjectId::NONE, SCR_MAIL, ObjectClass::Scroll);
    scroll.name = Some(format!("mail from {}", message.from));
    scroll.known = true;
    scroll.quantity = 1;
    scroll
}

/// Messages displayed during mail delivery
pub mod messages {
    /// Message when mail daemon appears
    pub const DAEMON_APPEARS: &str = "A strident voice sounds in your ear:";

    /// Message the daemon says
    pub const DAEMON_ANNOUNCEMENT: &str = "\"You have mail!\"";

    /// Message when daemon delivers mail
    pub fn delivery_message(from: &str) -> String {
        format!("The mail daemon hands you a scroll from {}.", from)
    }

    /// Message when daemon disappears
    pub const DAEMON_DISAPPEARS: &str = "The mail daemon disappears in a puff of smoke.";

    /// Message when trying to attack the daemon
    pub const DAEMON_IMMUNE: &str = "The mail daemon is magically protected!";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_state_new() {
        let state = MailState::new();
        assert!(state.enabled);
        assert!(state.pending_mail.is_empty());
        assert_eq!(state.delivery_cooldown, 100);
    }

    #[test]
    fn test_queue_mail() {
        let mut state = MailState::new();

        state.queue_mail(MailMessage::new("Wizard", "Hello!"));
        assert_eq!(state.pending_mail.len(), 1);

        state.queue_mail(MailMessage::urgent("Oracle", "Important!"));
        assert_eq!(state.pending_mail.len(), 2);

        // Urgent message should be first due to priority sorting
        assert_eq!(state.pending_mail[0].from, "Oracle");
    }

    #[test]
    fn test_can_deliver() {
        let mut state = MailState::new();

        // No mail - can't deliver
        assert!(!state.can_deliver(0));

        state.queue_mail(MailMessage::new("Test", "Test"));

        // Has mail, but need to wait for initial cooldown (last_delivery=0, cooldown=100)
        // So we need current_turn >= 100
        assert!(!state.can_deliver(0));
        assert!(!state.can_deliver(99));
        assert!(state.can_deliver(100));

        // Record delivery at turn 150
        state.record_delivery(150);

        // On cooldown - can't deliver (need turn >= 250)
        assert!(!state.can_deliver(200));
        assert!(!state.can_deliver(249));

        // After cooldown - can deliver
        assert!(state.can_deliver(250));
    }

    #[test]
    fn test_take_next_mail() {
        let mut state = MailState::new();
        state.queue_mail(MailMessage::new("Sender1", "Content1"));
        state.queue_mail(MailMessage::new("Sender2", "Content2"));

        let mail = state.take_next_mail();
        assert!(mail.is_some());
        assert_eq!(state.pending_mail.len(), 1);

        let mail = state.take_next_mail();
        assert!(mail.is_some());
        assert!(state.pending_mail.is_empty());

        let mail = state.take_next_mail();
        assert!(mail.is_none());
    }

    #[test]
    fn test_mail_message() {
        let msg = MailMessage::new("Wizard", "Greetings!");
        assert_eq!(msg.from, "Wizard");
        assert_eq!(msg.content, "Greetings!");
        assert_eq!(msg.priority, 0);

        let urgent = MailMessage::urgent("Oracle", "Urgent!");
        assert_eq!(urgent.priority, 10);
    }

    #[test]
    fn test_create_mail_daemon() {
        let daemon = create_mail_daemon(10, 10);
        assert_eq!(daemon.name, "mail daemon");
        assert!(daemon.state.peaceful);
        assert_eq!(daemon.x, 10);
        assert_eq!(daemon.y, 10);
    }

    #[test]
    fn test_create_scroll_of_mail() {
        let msg = MailMessage::new("Test Sender", "Test content");
        let scroll = create_scroll_of_mail(&msg);

        assert_eq!(scroll.class, ObjectClass::Scroll);
        assert!(scroll.known);
        assert!(scroll.name.as_ref().unwrap().contains("Test Sender"));
    }

    #[test]
    fn test_delivery_disabled() {
        let mut state = MailState::new();
        state.enabled = false;
        state.queue_mail(MailMessage::new("Test", "Test"));

        let level = crate::dungeon::Level::default();
        let mut rng = GameRng::new(42);

        let result = attempt_mail_delivery(&mut state, &level, 40, 10, 0, &mut rng);
        assert!(matches!(result, MailDeliveryResult::Disabled));
    }

    #[test]
    fn test_delivery_no_mail() {
        let mut state = MailState::new();
        let level = crate::dungeon::Level::default();
        let mut rng = GameRng::new(42);

        let result = attempt_mail_delivery(&mut state, &level, 40, 10, 0, &mut rng);
        assert!(matches!(result, MailDeliveryResult::NoMail));
    }

    #[test]
    fn test_delivery_on_cooldown() {
        let mut state = MailState::new();
        state.queue_mail(MailMessage::new("Test", "Test"));
        state.record_delivery(50);

        let level = crate::dungeon::Level::default();
        let mut rng = GameRng::new(42);

        let result = attempt_mail_delivery(&mut state, &level, 40, 10, 100, &mut rng);
        assert!(matches!(result, MailDeliveryResult::OnCooldown));
    }
}
