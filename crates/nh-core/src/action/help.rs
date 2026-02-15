//! Help system functions translated from NetHack pager.c
//!
//! Provides help content and display functions for:
//! - Main help menu
//! - Command reference
//! - Game history
//! - License information
//! - Options documentation

/// Main help content sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(feature = "std"))]
use crate::compat::*;

pub enum HelpSection {
    /// Main help/tutorial
    Help,
    /// Short help
    ShortHelp,
    /// Debug help (wizard mode)
    DebugHelp,
    /// License file
    License,
    /// Options documentation
    OptionsHelp,
    /// Monsters reference
    Monsters,
    /// Artifacts reference
    Artifacts,
    /// Spells reference
    Spells,
    /// Conduct information
    Conduct,
    /// Game time/calendar
    TimeInfo,
}

/// Get help content for a section
pub fn get_help_content(section: HelpSection) -> String {
    match section {
        HelpSection::Help => HELP_CONTENT.to_string(),
        HelpSection::ShortHelp => SHORT_HELP_CONTENT.to_string(),
        HelpSection::DebugHelp => DEBUG_HELP_CONTENT.to_string(),
        HelpSection::License => LICENSE_CONTENT.to_string(),
        HelpSection::OptionsHelp => OPTIONS_HELP_CONTENT.to_string(),
        HelpSection::Monsters => MONSTERS_CONTENT.to_string(),
        HelpSection::Artifacts => ARTIFACTS_CONTENT.to_string(),
        HelpSection::Spells => SPELLS_CONTENT.to_string(),
        HelpSection::Conduct => CONDUCT_CONTENT.to_string(),
        HelpSection::TimeInfo => TIME_INFO_CONTENT.to_string(),
    }
}

/// Display help menu
/// Returns the help content as a string
pub fn dohelp() -> String {
    HELP_CONTENT.to_string()
}

/// Display game history
pub fn dohistory() -> String {
    HISTORY_CONTENT.to_string()
}

/// Display main help file
pub fn dispfile_help() -> String {
    HELP_CONTENT.to_string()
}

/// Display short help
pub fn dispfile_shelp() -> String {
    SHORT_HELP_CONTENT.to_string()
}

/// Display debug help (wizard mode)
pub fn dispfile_debughelp() -> String {
    DEBUG_HELP_CONTENT.to_string()
}

/// Display license file
pub fn dispfile_license() -> String {
    LICENSE_CONTENT.to_string()
}

/// Display option file documentation
pub fn dispfile_optionfile() -> String {
    OPTIONS_HELP_CONTENT.to_string()
}

/// Display monsters reference
pub fn dispfile_monsters() -> String {
    MONSTERS_CONTENT.to_string()
}

/// Display artifacts reference
pub fn dispfile_artifacts() -> String {
    ARTIFACTS_CONTENT.to_string()
}

/// Display spells reference
pub fn dispfile_spells() -> String {
    SPELLS_CONTENT.to_string()
}

/// Display conduct information
pub fn dispfile_conduct() -> String {
    CONDUCT_CONTENT.to_string()
}

/// Display time information
pub fn dispfile_timeinfo() -> String {
    TIME_INFO_CONTENT.to_string()
}

const HELP_CONTENT: &str = r#"
NetHack Help
============

Getting Started
---------------
Welcome to NetHack, a roguelike dungeon exploration game!

Movement
--------
Use arrow keys or hjkl (vi-keys) to move around:
  y k u    (diagonal + up)
  h . l    (left, wait, right)
  b j n    (diagonal + down)

Or use 1-9 on numeric keypad for movement and resting.

Basic Commands
--------------
? - Help/command browser
/ - Identify objects on screen
: - Look around
~ - Repeat last command
& - Select prefixes for commands

Objects
-------
, or g - Pick up object
d - Drop item
e - Eat food/corpse
q - Drink potion
r - Read scroll/spell
a - Apply tool
w - Wear armor
t - Take off armor
P - Put on ring
R - Remove ring

Combat
------
f or Ctrl-Direction - Attack in direction
m - Move without attacking
z - Zap wand
t - Throw item at target

Special Actions
---------------
< - Go up stairs
> - Go down stairs
s - Search for hidden things
p - Pay toll
. - Rest one turn (or just space)
> - Descend
< - Ascend

Meta
----
S - Save game
q - Quit
# - Extended commands
? - Help

For more information, press ? in game.
"#;

const SHORT_HELP_CONTENT: &str = r#"
Quick Start
-----------
hjkl - Move around (or arrow keys)
. or space - Rest
, - Pick up
d - Drop
e - Eat
w - Wear
S - Save
q - Quit
? - Help
"#;

const DEBUG_HELP_CONTENT: &str = r#"
Wizard Mode Commands
====================

These commands are only available in wizard mode.

Debug Navigation
----------------
X - Level teleport
Z - Spell power level

Debug Inspection
----------------
^ - Identify item
] - List dungeon features

Debug Modification
-------------------
E - Edit dungeon
O - Change object
G - Give item to player

Debug Statistics
----------------
# stats - Show game statistics
# time - Show time information

For detailed information, consult the source code or documentation.
"#;

const LICENSE_CONTENT: &str = r#"
NetHack License
===============

NetHack is released under the NetHack General Public License.

This implementation (nethack-rs) is released under the MIT License.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

Full license text available in the source distribution.
"#;

const OPTIONS_HELP_CONTENT: &str = r#"
Game Options
============

Options can be set in ~/.nethackrc or via the options menu.

Display Options
---------------
color - Enable color display (yes/no)
hilite_pet - Highlight your pet (yes/no)
showexp - Show experience points (yes/no)
time - Show time of day (yes/no)

Gameplay Options
----------------
autopickup - Automatically pick up items (yes/no)
safe_pet - Don't attack your pet (yes/no)
confirm - Confirm before dangerous actions (yes/no)
verbose - Verbose mode messages (yes/no)

Movement Options
----------------
number_pad - Use numeric keypad for movement (yes/no)
runmode - Running mode (teleport/run/walk)
travel - Enable travel command (yes/no)

Interface Options
-----------------
msg_window - Message window type (single/combo/full)
msghistory - Number of messages to keep (1-100)

Key Bindings
------------
bind:<key>=<command> - Bind key to command

Example:
  bind:z=cast
  bind:^w=wield

For complete option documentation, see the help menu.
"#;

const HISTORY_CONTENT: &str = r#"
NetHack History
===============

NetHack is a descendant of the Rogue computer game, which was
invented around 1980 at UC Berkeley. The original was expanded
by many contributors into what is now known as Hack, which was
originally ported to Unix by Don G. Kneller.

NetHack evolved from Hack in 1987 and has been continuously
developed and improved by a community of contributors for
over three decades.

The name "NetHack" refers to the original network distribution
system used for sharing the game among early players.

This Rust implementation (nethack-rs) is a modernization effort
to bring the classic dungeon exploration experience to modern
systems while preserving the spirit of the original game.

Key versions:
- 1987: NetHack 1.0
- 1989: NetHack 2.0
- 1992: NetHack 3.0
- 2007: NetHack 3.6
- 2024: nethack-rs (Rust implementation)

For more historical information, visit the official NetHack website.
"#;

const MONSTERS_CONTENT: &str = r#"
Monster Reference
=================

Common Monsters
---------------
Goblin (difficulty: 1) - Weak melee fighter
Orc (difficulty: 2) - Stronger melee attacker
Troll (difficulty: 4) - Regenerates health
Giant Spider (difficulty: 3) - Venomous attacks
Zombie (difficulty: 2) - Undead servant
Ghost (difficulty: 3) - Can pass through walls
Wraith (difficulty: 4) - Level drain on hit

Dragons (difficulty: 6+)
------------------------
Red Dragon - Fire breath
White Dragon - Cold breath
Blue Dragon - Lightning breath
Green Dragon - Poison breath
Black Dragon - Acid breath

Special Monsters
----------------
Unicorn (difficulty: 5) - Non-hostile, valuable
Nymph (difficulty: 2) - Steals items and flees
Floating Eye (difficulty: 1) - Paralyzing gaze
Cockatrice (difficulty: 2) - Petrification

Unique Monsters (Bosses)
------------------------
Medusa (difficulty: 8)
Vlad the Impaler (difficulty: 10)
The Wizard of Yendor (difficulty: 12)

Tip: Learn monster types to predict attacks!
"#;

const ARTIFACTS_CONTENT: &str = r#"
Artifacts Reference
====================

The Quest Artifacts
-------------------
Excalibur - Sword of righteousness
Grayswandir - Elven blade
Cleaver - Dwarvish battle axe
Kusanagi - Katana
Firewall - Quarterstaff
Frost Brand - Ice sword
Flaming Sword - Fire sword

Neutral Artifacts
-----------------
Wand of Wish - Extremely powerful
Amulet of Yendor - Quest goal
Orb of Zot - Final objective
Ring of Wish - Limited wishes
Robe of the Archmagi - Spell power

Cursed Artifacts
----------------
Sting - Elven dagger (weak)
Magicbane - Powerful but drains power
Dragonbane - Anti-dragon weapon

Collection Tips
---------------
- Many artifacts are guarded by monsters
- Some artifacts grant special abilities
- Keep discovered artifacts in your discoveries list
- Some are cursed and need identification

Tip: Discovering all artifacts is a coveted achievement!
"#;

const SPELLS_CONTENT: &str = r#"
Spells Reference
================

Utility Spells
--------------
Light (level 1) - Illuminate area
Detect Monsters (level 1) - Sense nearby creatures
Invisibility (level 2) - Hide from enemies
Teleportation (level 3) - Jump to location
Levitation (level 4) - Float above ground

Combat Spells
-------------
Magic Missile (level 1) - Basic ranged attack
Shock Wave (level 2) - Blast nearby enemies
Fireball (level 3) - Area fire damage
Cone of Cold (level 4) - Freeze enemies
Finger of Death (level 5) - Instant kill attempt

Healing Spells
--------------
Healing (level 1) - Restore HP
Cure Wounds (level 2) - Remove injuries
Full Healing (level 3) - Complete restoration
Restoration (level 4) - Restore attributes

Enhancement Spells
------------------
Strength (level 1) - Boost strength
Haste (level 2) - Move faster
Protection (level 2) - Reduce damage taken
Polymorph (level 3) - Change form

Spell Resources
---------------
- Cast spells by memorizing scrolls or spell books
- Requires mana (magical energy)
- Improves with practice
- Some spells have level requirements

Tip: A diverse spell selection makes adventuring easier!
"#;

const CONDUCT_CONTENT: &str = r#"
Conduct Information
===================

What are Conducts?
------------------
Conducts are personal rules or achievements that track
your playstyle and can affect your final score.

Major Conducts
--------------
Vegetarianism - Never eat meat
Pacifism - Avoid killing monsters (use magic/allies)
Atheism - Don't use prayers or divine intervention
Illiteracy - Never read books or scrolls
Weaponless - Don't wield any weapons
Alchemy - Only use alchemy for healing
Genoside Prevention - Don't wipe out a monster race

Breaking Conducts
------------------
- Eating meat breaks vegetarianism
- Killing breaks pacifism
- Praying breaks atheism
- Reading breaks illiteracy
- Wielding breaks weaponless

Benefits
--------
- Maintains moral integrity
- Increases final score multiplier
- Unlocks special endings
- Achievement/bragging rights
- Unique playstyle challenges

Tip: Challenging conduct runs provide extra entertainment!
"#;

const TIME_INFO_CONTENT: &str = r#"
Game Time Information
=====================

Calendar System
---------------
The game takes place over multiple turns (actions).
Each turn represents a game-world day.

Time Tracking
-------------
Turn Counter - Number of actions taken
Game Day - Current date in game calendar
Game Year - Current year (usually year 1 of discovery)
Time of Day - Morning/Noon/Evening/Night

Seasonal Effects
----------------
Spring - Better growth, more water
Summer - Faster movement outdoors
Autumn - Resource gathering season
Winter - Harsh conditions, limited resources

Important Dates
---------------
Year 1 - Your arrival
Year 5 - Mid-point adventure
Year 10+ - Endgame challenges

Time Mechanics
--------------
- Resting advances time
- Combat takes more time
- Travel takes variable time
- Some effects are time-based

Tip: Track your playtime across turns for your statistics!
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dohelp() {
        let help = dohelp();
        assert!(help.contains("NetHack Help"));
        assert!(help.contains("Movement"));
    }

    #[test]
    fn test_dohistory() {
        let history = dohistory();
        assert!(history.contains("NetHack History"));
        assert!(history.contains("1987"));
    }

    #[test]
    fn test_get_help_content() {
        let help = get_help_content(HelpSection::Help);
        assert!(help.contains("NetHack Help"));

        let license = get_help_content(HelpSection::License);
        assert!(license.contains("License"));

        let options = get_help_content(HelpSection::OptionsHelp);
        assert!(options.contains("Options"));
    }

    #[test]
    fn test_dispfile_functions() {
        let short_help = dispfile_shelp();
        assert!(short_help.contains("Quick Start"));

        let debug_help = dispfile_debughelp();
        assert!(debug_help.contains("Wizard"));

        let license = dispfile_license();
        assert!(license.contains("License"));

        let options = dispfile_optionfile();
        assert!(options.contains("Options"));
    }
}
