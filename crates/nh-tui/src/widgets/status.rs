//! Status line widget

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use nh_core::player::{Attribute, You};

/// Widget for rendering the status line
pub struct StatusWidget<'a> {
    player: &'a You,
}

impl<'a> StatusWidget<'a> {
    pub fn new(player: &'a You) -> Self {
        Self { player }
    }
}

impl Widget for StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let p = self.player;

        // Line 1: Name, role, stats, HP, Pw
        let str_str = p.attr_current.strength_string();
        let line1 = format!(
            "{} the {} St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{} HP:{}/{} Pw:{}/{}",
            p.name,
            p.rank_title(),
            str_str,
            p.attr_current.get(Attribute::Dexterity),
            p.attr_current.get(Attribute::Constitution),
            p.attr_current.get(Attribute::Intelligence),
            p.attr_current.get(Attribute::Wisdom),
            p.attr_current.get(Attribute::Charisma),
            p.hp,
            p.hp_max,
            p.energy,
            p.energy_max,
        );

        // Line 2: Dlvl, $, AC, Exp, T, status
        let mut line2 = format!(
            "{} $:{} AC:{} Xp:{}/{} T:{}",
            p.level, p.gold, p.armor_class, p.exp_level, p.exp, p.turns_played,
        );

        // Add status conditions
        if let Some(hunger) = p.hunger_state.status_string() {
            line2.push_str(&format!(" {}", hunger));
        }
        if let Some(enc) = p.encumbrance().status_string() {
            line2.push_str(&format!(" {}", enc));
        }
        if p.is_confused() {
            line2.push_str(" Conf");
        }
        if p.is_stunned() {
            line2.push_str(" Stun");
        }
        if p.is_blind() {
            line2.push_str(" Blind");
        }

        // Render
        let style = Style::default().fg(Color::White);
        buf.set_string(area.x, area.y, &line1, style);
        if area.height > 1 {
            buf.set_string(area.x, area.y + 1, &line2, style);
        }
    }
}
