//! BattlefieldPlugin — renders two horizontal rows of cards and the hand zone root.
//!
//! Layout (top to bottom):
//! ```text
//! ┌─────────────────────────────┐
//! │  Opponent Battlefield (top) │  cards rotated 180°
//! ├─────────────────────────────┤
//! │  Player Battlefield (mid)   │  cards normal
//! ├─────────────────────────────┤
//! │  Hand zone (HandRoot)       │  filled by HandPlugin (Phase 8.3)
//! └─────────────────────────────┘
//! ```
//!
//! The system `rebuild_battlefields` despawns and re-spawns all card entities
//! whenever a `SnapshotChangedMessage` is received.
//!
//! Untapped lands in the player's battlefield that have a mana ability are
//! rendered with a gold border and a `TappableLand` component + `Button` for
//! click-to-tap interaction.

use bevy::prelude::*;
use echomancy_core::prelude::{Action, CardInstanceId, PlayerId, Target};

use super::card::{CardSpawnData, CARD_GAP, spawn_card_with_tappable};
use super::hand::HandRoot;
use crate::plugins::game::{HumanPlayerId, CurrentSnapshot, GameActionMessage, PlayableCards, SnapshotChangedMessage, TargetSelectionState};

// ============================================================================
// Marker components
// ============================================================================

/// Marks the full-screen root layout node.
#[derive(Component)]
pub(crate) struct BattlefieldRoot;

/// Marks the opponent battlefield zone node.
#[derive(Component)]
pub(crate) struct OpponentBattlefieldRoot;

/// Marks the player battlefield zone node.
#[derive(Component)]
pub(crate) struct PlayerBattlefieldRoot;

/// Marks an untapped land in the player's battlefield that can be tapped for mana.
///
/// Carries the `instance_id` so the click handler can send the `ActivateAbility` action.
#[derive(Component, Clone)]
pub(crate) struct TappableLand {
    pub(crate) instance_id: String,
}

/// Marks a creature in the player's battlefield that can be declared as an attacker.
///
/// Carries the `instance_id` so the click handler can send the `DeclareAttacker` action.
#[derive(Component, Clone)]
pub(crate) struct AttackableCreature {
    pub(crate) instance_id: String,
}

/// Marks a creature in the player's battlefield that can be declared as a blocker.
///
/// During DeclareBlockers, clicking auto-assigns it to block the first available attacker.
#[derive(Component, Clone)]
pub(crate) struct BlockableCreature {
    pub(crate) instance_id: String,
}

/// Marks an opponent's creature or player area that is a valid target during
/// target-selection mode.
///
/// Carrying the `Target` domain value lets the click handler dispatch `CastSpell`
/// with the correct target without any additional look-up.
#[derive(Component, Clone)]
pub(crate) struct ValidTarget {
    pub(crate) target: Target,
}

// ============================================================================
// Colors
// ============================================================================

/// Background color for the opponent battlefield zone — warm dark slate.
const OPPONENT_ZONE_BG: Color = Color::srgb(0.18, 0.16, 0.22);

/// Background color for the player battlefield zone — warm dark teal.
const PLAYER_ZONE_BG: Color = Color::srgb(0.14, 0.22, 0.20);

/// Background color for the hand zone — warm dark brown.
const HAND_ZONE_BG: Color = Color::srgb(0.16, 0.14, 0.12);

/// Gold border for tappable (untapped mana) lands on the player's battlefield.
const TAPPABLE_LAND_BORDER_COLOR: Color = Color::srgb(0.90, 0.75, 0.10);

/// Red border for creatures that can be declared as attackers.
const ATTACKABLE_CREATURE_BORDER_COLOR: Color = Color::srgb(0.90, 0.20, 0.15);

/// Blue border for creatures that can be declared as blockers.
const BLOCKABLE_CREATURE_BORDER_COLOR: Color = Color::srgb(0.20, 0.50, 0.90);

/// Yellow border for valid targets during target-selection mode.
const VALID_TARGET_BORDER_COLOR: Color = Color::srgb(1.00, 0.85, 0.10);

// ============================================================================
// Systems
// ============================================================================

/// Startup system: create the root layout with 3 zones.
///
/// Zones:
/// 1. Opponent battlefield (top, 35% height)
/// 2. Player battlefield (middle, 35% height)
/// 3. Hand placeholder (bottom, 30% height — filled in Phase 8.3)
///
/// The HUD panel (Phase 8.4) is an absolutely-positioned overlay spawned by
/// `HudPlugin` — it does not affect this flex layout.
pub(crate) fn spawn_ui_root(mut commands: Commands) {
    commands
        .spawn((
            BattlefieldRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                // Leave space for HUD panel on the right
                padding: UiRect {
                    right: Val::Px(super::hud::HUD_PANEL_WIDTH),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|root| {
            // Opponent battlefield zone (top, 35%)
            root.spawn((
                OpponentBattlefieldRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(35.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(CARD_GAP),
                    padding: UiRect::all(Val::Px(8.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(OPPONENT_ZONE_BG),
            ));

            // Player battlefield zone (middle, 35%)
            root.spawn((
                PlayerBattlefieldRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(35.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(CARD_GAP),
                    padding: UiRect::all(Val::Px(8.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(PLAYER_ZONE_BG),
            ));

            // Hand zone (bottom, 30% — Phase 8.3)
            root.spawn((
                HandRoot,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(HAND_ZONE_BG),
            ));
        });
}

/// Returns `true` when `instance_id` is in the list of tappable land IDs.
pub(crate) fn is_tappable_land(instance_id: &str, tappable_lands: &[String]) -> bool {
    tappable_lands.iter().any(|id| id == instance_id)
}

/// Returns `true` when `instance_id` is in the list of attackable creature IDs.
pub(crate) fn is_attackable_creature(instance_id: &str, attackable: &[String]) -> bool {
    attackable.iter().any(|id| id == instance_id)
}

/// Returns `true` when `instance_id` is in the list of blockable creature IDs.
pub(crate) fn is_blockable_creature(instance_id: &str, blockable: &[String]) -> bool {
    blockable.iter().any(|id| id == instance_id)
}

/// Determine the override border color for a player's battlefield card.
///
/// Priority (highest first):
/// 1. Attackable creature → red border.
/// 2. Blockable creature → blue border.
/// 3. Tappable land → gold border.
/// 4. No override → `None` (use the type-based default from `card_border_color`).
pub(crate) fn player_card_override_border(
    instance_id: &str,
    tappable_lands: &[String],
    attackable_creatures: &[String],
    blockable_creatures: &[String],
) -> Option<Color> {
    if is_attackable_creature(instance_id, attackable_creatures) {
        Some(ATTACKABLE_CREATURE_BORDER_COLOR)
    } else if is_blockable_creature(instance_id, blockable_creatures) {
        Some(BLOCKABLE_CREATURE_BORDER_COLOR)
    } else if is_tappable_land(instance_id, tappable_lands) {
        Some(TAPPABLE_LAND_BORDER_COLOR)
    } else {
        None
    }
}

/// Update system: rebuild battlefield card entities when the snapshot changes
/// or when the `CurrentSnapshot` resource is first added (initial render).
///
/// Despawns all existing card children of each battlefield root, then spawns
/// fresh card entities from `CurrentSnapshot`.
///
/// Cards are rendered with interactive borders and click components based on
/// the current `PlayableCards` result:
/// - Untapped mana lands → gold border + `TappableLand`.
/// - Attackable creatures (DeclareAttackers) → red border + `AttackableCreature`.
/// - Blockable creatures (DeclareBlockers) → blue border + `BlockableCreature`.
/// - Attacking creatures (combat state) → "ATK" label overlay.
/// - Blocking creatures (combat state) → "BLK" label overlay.
///
/// When `TargetSelectionState` is active, opponent creatures receive a yellow
/// border and a `ValidTarget` component so they can be clicked to cast the
/// pending spell.
pub(crate) fn rebuild_battlefields(
    mut commands: Commands,
    current_snapshot: Res<CurrentSnapshot>,
    playable_cards: Res<PlayableCards>,
    target_selection: Res<TargetSelectionState>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    player_battlefield_q: Query<Entity, With<PlayerBattlefieldRoot>>,
    opponent_battlefield_q: Query<Entity, With<OpponentBattlefieldRoot>>,
) {
    // Only rebuild when the snapshot has actually changed.
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let snapshot = &current_snapshot.snapshot;
    let result = &playable_cards.result;

    // ---- Rebuild player battlefield ----
    if let Ok(player_root) = player_battlefield_q.single() {
        // Despawn all existing card children.
        commands.entity(player_root).despawn_children();

        // Spawn new cards as children.
        for card in &snapshot.private_player_state.battlefield {
            let is_tapped = card.tapped.unwrap_or(false);

            let override_border = player_card_override_border(
                &card.instance_id,
                &result.tappable_lands,
                &result.attackable_creatures,
                &result.blockable_creatures,
            );

            let is_attacking = card
                .combat_state
                .as_ref()
                .is_some_and(|cs| cs.is_attacking);
            let is_blocking = card
                .combat_state
                .as_ref()
                .is_some_and(|cs| cs.is_blocking);

            let tappable = is_tappable_land(&card.instance_id, &result.tappable_lands);
            let attackable = is_attackable_creature(&card.instance_id, &result.attackable_creatures);
            let blockable = is_blockable_creature(&card.instance_id, &result.blockable_creatures);

            let mut card_entity_cmd = spawn_card_with_tappable(
                &mut commands,
                &CardSpawnData {
                    name: &card.name,
                    types: &card.types,
                    power: card.power,
                    toughness: card.toughness,
                    is_tapped,
                    is_opponent: false,
                },
                override_border,
            );

            // Insert interactive components based on what this card can do.
            if tappable {
                card_entity_cmd.insert((
                    TappableLand { instance_id: card.instance_id.clone() },
                    Button,
                    Interaction::default(),
                ));
            } else if attackable {
                card_entity_cmd.insert((
                    AttackableCreature { instance_id: card.instance_id.clone() },
                    Button,
                    Interaction::default(),
                ));
            } else if blockable {
                card_entity_cmd.insert((
                    BlockableCreature { instance_id: card.instance_id.clone() },
                    Button,
                    Interaction::default(),
                ));
            }

            let card_entity = card_entity_cmd.id();

            // Add ATK / BLK overlay label for combat status.
            if is_attacking || is_blocking {
                let label = if is_attacking { "ATK" } else { "BLK" };
                let label_color = if is_attacking {
                    Color::srgb(1.0, 0.25, 0.15)
                } else {
                    Color::srgb(0.25, 0.60, 1.0)
                };
                commands.entity(card_entity).with_children(|parent| {
                    parent.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(label_color),
                    ));
                });
            }

            commands.entity(player_root).add_child(card_entity);
        }
    }

    // ---- Rebuild opponent battlefield ----
    let targeting_active = target_selection.pending_spell.is_some();

    if let Ok(opponent_root) = opponent_battlefield_q.single() {
        commands.entity(opponent_root).despawn_children();

        if let Some(opponent) = snapshot.opponent_states.first() {
            for card in &opponent.battlefield {
                let is_tapped = card.tapped.unwrap_or(false);
                let is_attacking = card
                    .combat_state
                    .as_ref()
                    .is_some_and(|cs| cs.is_attacking);
                let is_creature = card.types.contains(&echomancy_core::prelude::CardType::Creature);

                // During target selection, opponent's creatures are valid targets
                // and receive a yellow border + clickable component.
                let target_highlight = targeting_active && is_creature;

                let border_override = if target_highlight {
                    Some(VALID_TARGET_BORDER_COLOR)
                } else {
                    None
                };

                let mut card_entity_cmd = spawn_card_with_tappable(
                    &mut commands,
                    &CardSpawnData {
                        name: &card.name,
                        types: &card.types,
                        power: card.power,
                        toughness: card.toughness,
                        is_tapped,
                        is_opponent: true,
                    },
                    border_override,
                );

                if target_highlight {
                    card_entity_cmd.insert((
                        ValidTarget {
                            target: Target::creature(card.instance_id.clone()),
                        },
                        Button,
                        Interaction::default(),
                    ));
                }

                let card_entity = card_entity_cmd.id();

                // Show ATK indicator for opponent's attacking creatures.
                if is_attacking {
                    commands.entity(card_entity).with_children(|parent| {
                        parent.spawn((
                            Text::new("ATK"),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.25, 0.15)),
                        ));
                    });
                }

                commands.entity(opponent_root).add_child(card_entity);
            }
        }
    }
}

/// Click handler: detect clicks on `TappableLand` buttons and send `ActivateAbility`.
pub(crate) fn handle_battlefield_land_click(
    query: Query<(&Interaction, &TappableLand), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, tappable) in &query {
        if *interaction == Interaction::Pressed {
            action_writer.write(GameActionMessage(Action::ActivateAbility {
                player_id: PlayerId::new(&active_player.player_id),
                permanent_id: CardInstanceId::new(&tappable.instance_id),
            }));
        }
    }
}

/// Click handler: detect clicks on `AttackableCreature` buttons and send `DeclareAttacker`.
pub(crate) fn handle_attacker_click(
    query: Query<(&Interaction, &AttackableCreature), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, attackable) in &query {
        if *interaction == Interaction::Pressed {
            action_writer.write(GameActionMessage(Action::DeclareAttacker {
                player_id: PlayerId::new(&active_player.player_id),
                creature_id: CardInstanceId::new(&attackable.instance_id),
            }));
        }
    }
}

/// Click handler: detect clicks on `ValidTarget` entities during target-selection mode.
///
/// Reads the pending spell from `TargetSelectionState`, dispatches `CastSpell` with
/// the chosen target, and clears the pending state.
pub(crate) fn handle_valid_target_click(
    query: Query<(&Interaction, &ValidTarget), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    mut target_selection: ResMut<TargetSelectionState>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, valid_target) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(pending) = target_selection.pending_spell.take() {
            action_writer.write(GameActionMessage(Action::CastSpell {
                player_id: PlayerId::new(&active_player.player_id),
                card_id: CardInstanceId::new(&pending.card_instance_id),
                targets: vec![valid_target.target.clone()],
            }));
        }
    }
}

/// Click handler: detect clicks on `BlockableCreature` buttons and send `DeclareBlocker`.
///
/// MVP simplification: the blocker is auto-assigned to the first available attacker
/// on the opponent's battlefield. This avoids needing a two-click targeting UI.
pub(crate) fn handle_blocker_click(
    query: Query<(&Interaction, &BlockableCreature), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    current_snapshot: Res<CurrentSnapshot>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, blockable) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Find the first attacking creature on the opponent's battlefield.
        let attacker_id = current_snapshot
            .snapshot
            .opponent_states
            .first()
            .and_then(|opp| {
                opp.battlefield
                    .iter()
                    .find(|card| {
                        card.combat_state
                            .as_ref()
                            .is_some_and(|cs| cs.is_attacking && cs.blocked_by.is_empty())
                    })
                    .map(|card| card.instance_id.clone())
            });

        if let Some(attacker_id) = attacker_id {
            action_writer.write(GameActionMessage(Action::DeclareBlocker {
                player_id: PlayerId::new(&active_player.player_id),
                blocker_id: CardInstanceId::new(&blockable.instance_id),
                attacker_id: CardInstanceId::new(&attacker_id),
            }));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn land_in_tappable_list_is_tappable() {
        let tappable = vec!["land-1".to_owned(), "land-2".to_owned()];
        assert!(is_tappable_land("land-1", &tappable));
        assert!(is_tappable_land("land-2", &tappable));
    }

    #[test]
    fn land_not_in_tappable_list_is_not_tappable() {
        let tappable = vec!["land-1".to_owned()];
        assert!(!is_tappable_land("land-99", &tappable));
    }

    #[test]
    fn empty_tappable_list_means_nothing_is_tappable() {
        assert!(!is_tappable_land("land-1", &[]));
    }

    #[test]
    fn tappable_check_is_exact_match() {
        let tappable = vec!["land-1-long".to_owned()];
        assert!(!is_tappable_land("land-1", &tappable));
    }

    // ---- is_attackable_creature --------------------------------------------

    #[test]
    fn creature_in_attackable_list_is_attackable() {
        let attackable = vec!["bear-1".to_owned(), "bear-2".to_owned()];
        assert!(is_attackable_creature("bear-1", &attackable));
        assert!(is_attackable_creature("bear-2", &attackable));
    }

    #[test]
    fn creature_not_in_attackable_list_is_not_attackable() {
        let attackable = vec!["bear-1".to_owned()];
        assert!(!is_attackable_creature("bear-99", &attackable));
    }

    #[test]
    fn empty_attackable_list_means_nothing_is_attackable() {
        assert!(!is_attackable_creature("bear-1", &[]));
    }

    // ---- is_blockable_creature --------------------------------------------

    #[test]
    fn creature_in_blockable_list_is_blockable() {
        let blockable = vec!["goblin-1".to_owned()];
        assert!(is_blockable_creature("goblin-1", &blockable));
    }

    #[test]
    fn creature_not_in_blockable_list_is_not_blockable() {
        let blockable = vec!["goblin-1".to_owned()];
        assert!(!is_blockable_creature("goblin-99", &blockable));
    }

    // ---- player_card_override_border -------------------------------------

    #[test]
    fn attackable_creature_gets_red_border() {
        let border = player_card_override_border(
            "bear-1",
            &[],
            &["bear-1".to_owned()],
            &[],
        );
        assert_eq!(border, Some(ATTACKABLE_CREATURE_BORDER_COLOR));
    }

    #[test]
    fn blockable_creature_gets_blue_border() {
        let border = player_card_override_border(
            "goblin-1",
            &[],
            &[],
            &["goblin-1".to_owned()],
        );
        assert_eq!(border, Some(BLOCKABLE_CREATURE_BORDER_COLOR));
    }

    #[test]
    fn tappable_land_gets_gold_border() {
        let border = player_card_override_border(
            "forest-1",
            &["forest-1".to_owned()],
            &[],
            &[],
        );
        assert_eq!(border, Some(TAPPABLE_LAND_BORDER_COLOR));
    }

    #[test]
    fn no_special_card_has_no_override_border() {
        let border = player_card_override_border("plain-1", &[], &[], &[]);
        assert_eq!(border, None);
    }

    #[test]
    fn attackable_takes_priority_over_tappable() {
        // A card that is somehow both attackable and tappable → red wins.
        let border = player_card_override_border(
            "card-1",
            &["card-1".to_owned()],
            &["card-1".to_owned()],
            &[],
        );
        assert_eq!(border, Some(ATTACKABLE_CREATURE_BORDER_COLOR));
    }

    // ---- VALID_TARGET_BORDER_COLOR sanity ------------------------------------

    #[test]
    fn valid_target_border_is_bright_yellow() {
        let Color::Srgba(srgba) = VALID_TARGET_BORDER_COLOR else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 1.00).abs() < 0.01, "red channel mismatch");
        assert!((srgba.green - 0.85).abs() < 0.01, "green channel mismatch");
        assert!((srgba.blue - 0.10).abs() < 0.01, "blue channel mismatch");
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers battlefield rendering systems.
pub(crate) struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui_root)
            .add_systems(
                Update,
                (
                    rebuild_battlefields,
                    handle_battlefield_land_click,
                    handle_attacker_click,
                    handle_blocker_click,
                    handle_valid_target_click,
                ),
            );
    }
}
