# NetHack-rs Bevy 3D Enhancement Roadmap

## Current State

The Bevy client has:
- 3D tile map rendering (36 cell types)
- Player/monster text billboards
- 4 camera modes with zoom/pan
- Vi-key input + basic commands
- Game loop integration with nh-core

## Phase 1: Core UI (Playable Game)

### 1.1 Status HUD
- [ ] HP bar with visual indicator (red gradient)
- [ ] Energy/Mana bar (blue)
- [ ] Hunger status indicator with icons (satiated → starving)
- [ ] Status condition icons (confused, blind, poisoned, etc.)
- [ ] Dungeon level display (Dlvl:X)
- [ ] Gold counter
- [ ] Experience level and progress bar
- [ ] Armor class display

### 1.2 Message System
- [ ] Message log panel (bottom of screen, last 5 messages)
- [ ] Scrollable message history (press 'P' for previous)
- [ ] Message fade animation
- [ ] Color-coded messages (combat=red, items=yellow, info=white)
- [ ] Floating combat text above entities

### 1.3 Object Rendering
- [ ] Render items on floor (currently only terrain)
- [ ] Different symbols/colors per object class
- [ ] Pile indicator for multiple items
- [ ] Corpse rendering with decay state
- [ ] Gold pile visualization

### 1.4 Inventory UI
- [ ] Inventory panel (press 'i')
- [ ] Item list grouped by class
- [ ] Item selection for actions (drop, eat, wear, etc.)
- [ ] Equipment slots display (weapon, armor, rings, etc.)
- [ ] Weight/burden indicator

### 1.5 Direction Selection
- [ ] Visual direction picker for open/close/kick
- [ ] Highlight adjacent tiles
- [ ] Keyboard (yuhjklbn) or click to select

---

## Phase 2: Interactions & Feedback

### 2.1 Door System
- [ ] Door open/close animation
- [ ] Locked door visual indicator
- [ ] Kicked door particles
- [ ] Secret door reveal effect

### 2.2 Stair Transitions
- [ ] Fade transition when changing levels
- [ ] Regenerate map geometry on level change
- [ ] Stair direction indicator (up vs down)

### 2.3 Combat Feedback
- [ ] Attack animation (player swing)
- [ ] Hit flash on monsters
- [ ] Floating damage numbers
- [ ] Death animation (monster fades/falls)
- [ ] Miss indicator

### 2.4 Movement Animation
- [ ] Smooth tile-to-tile movement (lerp over ~100ms)
- [ ] Monster movement animation
- [ ] Bump animation when blocked

### 2.5 Item Interactions
- [ ] Pickup animation (item floats to player)
- [ ] Drop animation
- [ ] Eat/drink visual feedback
- [ ] Equip item glow

---

## Phase 3: Menus & Audio

### 3.1 Main Menu
- [ ] New Game button
- [ ] Continue/Load button
- [ ] Settings button
- [ ] Quit button
- [ ] Background scene (dungeon render)

### 3.2 Pause Menu
- [ ] ESC to pause (currently quits)
- [ ] Resume
- [ ] Settings
- [ ] Save & Quit
- [ ] Quit without saving

### 3.3 Game Over Screen
- [ ] Death message display
- [ ] Character stats summary
- [ ] Restart button
- [ ] Main menu button

### 3.4 Settings Menu
- [ ] Camera sensitivity slider
- [ ] Zoom speed slider
- [ ] Music volume
- [ ] SFX volume
- [ ] Keybinding display

### 3.5 Sound Effects
- [ ] Footstep sounds (vary by terrain)
- [ ] Combat hit/miss sounds
- [ ] Door open/close sounds
- [ ] Item pickup sound
- [ ] Level up fanfare
- [ ] Death sound

### 3.6 Music
- [ ] Ambient dungeon music (looping)
- [ ] Combat music trigger
- [ ] Menu music

---

## Phase 4: Visual Polish

### 4.1 Fog of War
- [ ] Only render explored cells
- [ ] Dim unexplored but seen cells
- [ ] Line-of-sight calculation for visibility
- [ ] Blindness effect (all dark)

### 4.2 Lighting System
- [ ] Lit rooms brighter than corridors
- [ ] Player light radius
- [ ] Torch/lamp item effects
- [ ] Lava glow
- [ ] Fountain shimmer

### 4.3 Particle Effects
- [ ] Water ripples
- [ ] Lava bubbles
- [ ] Fountain spray
- [ ] Spell casting particles
- [ ] Potion effect auras

### 4.4 Better Entity Visuals
- [ ] Replace text billboards with sprite atlas
- [ ] Monster size scaling (tiny → gigantic)
- [ ] Unique sprites for major monsters
- [ ] Pet indicator (heart icon)
- [ ] Sleeping indicator (Zzz)

### 4.5 Environmental Animation
- [ ] Torch flame flicker
- [ ] Water surface animation
- [ ] Cloud movement in air levels
- [ ] Tree sway

---

## Phase 5: Advanced Features

### 5.1 Save/Load System
- [ ] Wire nh-save crate to Bevy
- [ ] Save game button
- [ ] Load game selection
- [ ] Auto-save on level change
- [ ] Multiple save slots

### 5.2 Minimap
- [ ] Corner minimap showing level layout
- [ ] Player position marker
- [ ] Monster dots (red = hostile)
- [ ] Stairs/special markers
- [ ] Toggle with 'M'

### 5.3 Advanced Input
- [ ] Mouse click to move (pathfinding)
- [ ] Click on monster to attack
- [ ] Click on item to examine
- [ ] Controller support
- [ ] Rebindable keys

### 5.4 Character Sheet
- [ ] Full attribute display
- [ ] Skill levels
- [ ] Resistances
- [ ] Intrinsics list
- [ ] Spell knowledge

### 5.5 Help System
- [ ] In-game key reference
- [ ] Tutorial messages for new players
- [ ] Monster/item lookup

---

## Phase 6: Future/Optional

### 6.1 3D Models
- [ ] Replace cubes with detailed dungeon tiles
- [ ] 3D monster models
- [ ] Player character model
- [ ] Weapon/armor models on character

### 6.2 Advanced Graphics
- [ ] Shadow mapping
- [ ] Ambient occlusion
- [ ] Bloom effect
- [ ] Screen shake on big hits

### 6.3 Accessibility
- [ ] Color blind mode
- [ ] High contrast option
- [ ] Adjustable text size
- [ ] Screen reader support

### 6.4 Multiplayer (Far Future)
- [ ] Online high scores
- [ ] Shared dungeons
- [ ] Ghost system (see where others died)

---

## Implementation Notes

### File Structure
```
crates/nh-bevy/src/
  plugins/
    ui/
      mod.rs          # UiPlugin
      hud.rs          # Status bars, HP, etc.
      messages.rs     # Message log
      inventory.rs    # Inventory panel
      menus.rs        # Main/pause/settings
    audio.rs          # AudioPlugin
    effects.rs        # Particle effects
    animation.rs      # Movement/combat animation
    fog.rs            # Fog of war
    lighting.rs       # Dynamic lighting
  assets/
    sprites/          # Entity sprites
    sounds/           # SFX
    music/            # Background music
    fonts/            # UI fonts
```

### Dependencies to Add
```toml
bevy_egui = "0.28"     # For UI panels
bevy_kira_audio = "0.20"  # For audio
```

### Priority Order
1. Phase 1.1-1.2 (HUD + Messages) - Essential for playability
2. Phase 1.3 (Objects) - See items on ground
3. Phase 2.3-2.4 (Combat + Movement) - Game feel
4. Phase 1.4-1.5 (Inventory + Direction) - Full interaction
5. Phase 3.1-3.3 (Menus) - Proper game flow
6. Phase 4.1 (Fog of War) - Classic roguelike feel
7. Everything else in order

### Estimated Effort
- Phase 1: ~2-3 days
- Phase 2: ~2-3 days
- Phase 3: ~2-3 days
- Phase 4: ~3-4 days
- Phase 5: ~2-3 days
- Phase 6: Ongoing/Optional
