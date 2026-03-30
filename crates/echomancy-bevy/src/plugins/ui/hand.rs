//! HandPlugin — renders the player's hand as an overlapping horizontal row.
//!
//! Layout (inside the bottom zone, 30% height):
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  [Card1][Card2][Card3][Card4][Card5][Card6][Card7]  │  overlapping by 45px
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! Playable cards (lands in FirstMain/SecondMain) have a bright green border
//! and a `PlayableCard` marker component that enables click-to-play.
//!
//! The `rebuild_hand` system runs whenever `SnapshotChangedMessage` is received,
//! following the same pattern as `BattlefieldPlugin`.

use bevy::prelude::*;
use echomancy_core::prelude::{Action, CardInstanceId, PlayerId};

use super::card::{CARD_BORDER, CARD_HEIGHT, CARD_WIDTH, CardNode, card_background_color, card_border_color, card_pt_text, card_type_line};
use crate::plugins::game::{HumanPlayerId, CurrentSnapshot, GameActionMessage, PendingSpell, PlayableCards, SnapshotChangedMessage, TargetSelectionState};

// ============================================================================
// Constants
// ============================================================================

/// Horizontal overlap for adjacent hand cards (negative left margin).
const HAND_CARD_OVERLAP: f32 = 45.0;

/// Brighter green border used for playable (land) cards in hand.
const PLAYABLE_BORDER_COLOR: Color = Color::srgb(0.20, 0.90, 0.30);

/// Blue border used for castable (non-land spell) cards in hand.
const CASTABLE_BORDER_COLOR: Color = Color::srgb(0.20, 0.50, 1.00);

// ============================================================================
// Marker components
// ============================================================================

/// Marks the bottom zone node that holds hand cards.
#[derive(Component)]
pub(crate) struct HandRoot;

/// Marks a card in the hand that can currently be played (e.g. a land in FirstMain).
///
/// Only added when the card's `instance_id` appears in `PlayableCards.result.playable_lands`.
#[derive(Component, Clone)]
pub(crate) struct PlayableCard {
    pub(crate) instance_id: String,
}

/// Marks a card in the hand that can currently be cast as a spell.
///
/// Only added when the card's `instance_id` appears in `PlayableCards.result.castable_spells`.
#[derive(Component, Clone)]
pub(crate) struct CastableSpell {
    pub(crate) instance_id: String,
}

// ============================================================================
// Pure helper — testable without ECS
// ============================================================================

/// Return the left margin for a card at position `index` in the hand row.
///
/// - Index 0 (first card): no negative margin.
/// - Subsequent cards: `-HAND_CARD_OVERLAP` to create the overlapping fan.
pub(crate) fn hand_card_left_margin(index: usize) -> Val {
    if index == 0 {
        Val::Px(0.0)
    } else {
        Val::Px(-HAND_CARD_OVERLAP)
    }
}

/// Return `true` when `instance_id` is in the list of playable land IDs.
pub(crate) fn is_playable_land(instance_id: &str, playable_lands: &[String]) -> bool {
    playable_lands.iter().any(|id| id == instance_id)
}

/// Return `true` when `instance_id` is in the list of castable spell IDs.
pub(crate) fn is_castable_spell(instance_id: &str, castable_spells: &[String]) -> bool {
    castable_spells.iter().any(|id| id == instance_id)
}

// ============================================================================
// Systems
// ============================================================================

/// Return `true` when `instance_id` is in the list of spells needing targets.
pub(crate) fn needs_target(instance_id: &str, spells_needing_targets: &[String]) -> bool {
    spells_needing_targets.iter().any(|id| id == instance_id)
}

/// Update system: rebuild hand card entities when the snapshot changes.
///
/// Despawns all children of `HandRoot`, then spawns fresh card entities from
/// `CurrentSnapshot.private_player_state.hand`. Playable lands receive a green
/// border and a `PlayableCard` + `Button` + `Interaction` for click detection.
///
/// When `TargetSelectionState` is active (a spell is waiting for a target),
/// hand cards are rendered without interactive components so they cannot be
/// clicked accidentally.
pub(crate) fn rebuild_hand(
    mut commands: Commands,
    current_snapshot: Res<CurrentSnapshot>,
    playable_cards: Res<PlayableCards>,
    target_selection: Res<TargetSelectionState>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    hand_root_q: Query<Entity, With<HandRoot>>,
) {
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let Ok(hand_root) = hand_root_q.single() else {
        return;
    };

    // Despawn all existing card children.
    commands.entity(hand_root).despawn_children();

    let hand = &current_snapshot.snapshot.private_player_state.hand;
    let playable_lands = &playable_cards.result.playable_lands;
    let castable_spells = &playable_cards.result.castable_spells;
    // When target selection is active, hand cards are not interactive.
    let targeting_active = target_selection.pending_spell.is_some();

    for (index, card) in hand.iter().enumerate() {
        let playable = is_playable_land(&card.instance_id, playable_lands);
        let castable = is_castable_spell(&card.instance_id, castable_spells);
        let border_color = if playable {
            PLAYABLE_BORDER_COLOR
        } else if castable {
            CASTABLE_BORDER_COLOR
        } else {
            card_border_color(&card.types)
        };
        let bg_color = card_background_color(&card.types);
        let type_line = card_type_line(&card.types);
        let pt_text = card_pt_text(card.power, card.toughness);
        let name = card.name.clone();
        let instance_id = card.instance_id.clone();
        let left_margin = hand_card_left_margin(index);

        let mut entity_cmd = commands.spawn((
            CardNode,
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                border: UiRect::all(Val::Px(CARD_BORDER)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                margin: UiRect {
                    left: left_margin,
                    ..default()
                },
                ..default()
            },
            BorderColor::all(border_color),
            BackgroundColor(bg_color),
        ));

        // During target selection, hand cards are non-interactive regardless of
        // their play/cast state. The player must first click a valid target or cancel.
        if !targeting_active {
            if playable {
                entity_cmd.insert((
                    PlayableCard { instance_id },
                    Button,
                    Interaction::default(),
                ));
            } else if castable {
                entity_cmd.insert((
                    CastableSpell { instance_id },
                    Button,
                    Interaction::default(),
                ));
            }
        }

        let card_entity = entity_cmd
            .with_children(|parent| {
                // Card name
                parent.spawn((
                    Text::new(name),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Art placeholder
                parent.spawn((
                    Node {
                        flex_grow: 1.0,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
                ));

                // Type line
                parent.spawn((
                    Text::new(type_line),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                // Power/toughness — creatures only
                if let Some(pt) = pt_text {
                    parent.spawn((
                        Text::new(pt),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                    ));
                }
            })
            .id();

        commands.entity(hand_root).add_child(card_entity);
    }
}

/// Update system: detect clicks on playable hand cards and send a `PlayLand` action.
///
/// Uses Bevy's built-in `Interaction` component — `Button` nodes get `Interaction`
/// automatically. Only `Interaction::Pressed` triggers an action.
pub(crate) fn handle_card_clicks(
    query: Query<(&Interaction, &PlayableCard), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, playable) in &query {
        if *interaction == Interaction::Pressed {
            action_writer.write(GameActionMessage(Action::PlayLand {
                player_id: PlayerId::new(&active_player.player_id),
                card_id: CardInstanceId::new(&playable.instance_id),
            }));
        }
    }
}

/// Update system: detect clicks on castable spell cards.
///
/// - If the spell is in `spells_needing_targets`, enters target-selection mode
///   by setting `TargetSelectionState.pending_spell` instead of dispatching immediately.
/// - Otherwise dispatches `CastSpell` with an empty targets vec (current behaviour).
///
/// Only `Interaction::Pressed` triggers an action.
pub(crate) fn handle_castable_spell_clicks(
    query: Query<(&Interaction, &CastableSpell), Changed<Interaction>>,
    active_player: Res<HumanPlayerId>,
    playable_cards: Res<PlayableCards>,
    current_snapshot: Res<CurrentSnapshot>,
    mut target_selection: ResMut<TargetSelectionState>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for (interaction, castable) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if needs_target(&castable.instance_id, &playable_cards.result.spells_needing_targets) {
            // Look up the card's definition ID from the snapshot so we store it
            // in the pending spell (useful for future display or CLIPS routing).
            let definition_id = current_snapshot
                .snapshot
                .private_player_state
                .hand
                .iter()
                .find(|c| c.instance_id == castable.instance_id)
                .map(|c| c.definition_id.clone())
                .unwrap_or_default();

            target_selection.pending_spell = Some(PendingSpell {
                card_instance_id: castable.instance_id.clone(),
                card_definition_id: definition_id,
            });
        } else {
            action_writer.write(GameActionMessage(Action::CastSpell {
                player_id: PlayerId::new(&active_player.player_id),
                card_id: CardInstanceId::new(&castable.instance_id),
                targets: vec![],
            }));
        }
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers hand rendering and click-to-play systems.
pub(crate) struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        use crate::plugins::game::AppState;
        app.add_systems(
            Update,
            (rebuild_hand, handle_card_clicks, handle_castable_spell_clicks)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

// ============================================================================
// Tests (TDD: written before implementation was final)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- hand_card_left_margin ---------------------------------------------

    #[test]
    fn first_card_has_no_negative_margin() {
        assert_eq!(hand_card_left_margin(0), Val::Px(0.0));
    }

    #[test]
    fn second_card_has_overlap_margin() {
        assert_eq!(hand_card_left_margin(1), Val::Px(-HAND_CARD_OVERLAP));
    }

    #[test]
    fn seventh_card_has_overlap_margin() {
        assert_eq!(hand_card_left_margin(6), Val::Px(-HAND_CARD_OVERLAP));
    }

    // ---- is_playable_land --------------------------------------------------

    #[test]
    fn card_in_playable_list_is_playable() {
        let playable = vec!["card-1".to_owned(), "card-2".to_owned()];
        assert!(is_playable_land("card-1", &playable));
        assert!(is_playable_land("card-2", &playable));
    }

    #[test]
    fn card_not_in_playable_list_is_not_playable() {
        let playable = vec!["card-1".to_owned()];
        assert!(!is_playable_land("card-99", &playable));
    }

    #[test]
    fn empty_playable_list_means_nothing_is_playable() {
        assert!(!is_playable_land("card-1", &[]));
    }

    #[test]
    fn playable_check_is_exact_match_not_substring() {
        let playable = vec!["card-1-long".to_owned()];
        assert!(!is_playable_land("card-1", &playable));
    }

    // ---- PLAYABLE_BORDER_COLOR sanity --------------------------------------

    #[test]
    fn playable_border_is_bright_green() {
        let Color::Srgba(srgba) = PLAYABLE_BORDER_COLOR else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.20).abs() < 0.01, "red channel mismatch");
        assert!((srgba.green - 0.90).abs() < 0.01, "green channel mismatch");
        assert!((srgba.blue - 0.30).abs() < 0.01, "blue channel mismatch");
    }

    // ---- is_castable_spell -------------------------------------------------

    #[test]
    fn card_in_castable_list_is_castable() {
        let castable = vec!["spell-1".to_owned(), "spell-2".to_owned()];
        assert!(is_castable_spell("spell-1", &castable));
        assert!(is_castable_spell("spell-2", &castable));
    }

    #[test]
    fn card_not_in_castable_list_is_not_castable() {
        let castable = vec!["spell-1".to_owned()];
        assert!(!is_castable_spell("spell-99", &castable));
    }

    #[test]
    fn empty_castable_list_means_nothing_is_castable() {
        assert!(!is_castable_spell("spell-1", &[]));
    }

    #[test]
    fn castable_check_is_exact_match_not_substring() {
        let castable = vec!["spell-1-long".to_owned()];
        assert!(!is_castable_spell("spell-1", &castable));
    }

    // ---- needs_target ----------------------------------------------------------

    #[test]
    fn spell_in_needs_target_list_needs_target() {
        let needs = vec!["spell-1".to_owned(), "spell-2".to_owned()];
        assert!(needs_target("spell-1", &needs));
        assert!(needs_target("spell-2", &needs));
    }

    #[test]
    fn spell_not_in_needs_target_list_does_not_need_target() {
        let needs = vec!["spell-1".to_owned()];
        assert!(!needs_target("spell-99", &needs));
    }

    #[test]
    fn empty_needs_target_list_means_nothing_needs_target() {
        assert!(!needs_target("spell-1", &[]));
    }

    #[test]
    fn needs_target_check_is_exact_match_not_substring() {
        let needs = vec!["spell-1-long".to_owned()];
        assert!(!needs_target("spell-1", &needs));
    }

    // ---- CASTABLE_BORDER_COLOR sanity --------------------------------------

    #[test]
    fn castable_border_is_blue() {
        let Color::Srgba(srgba) = CASTABLE_BORDER_COLOR else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.20).abs() < 0.01, "red channel mismatch");
        assert!((srgba.green - 0.50).abs() < 0.01, "green channel mismatch");
        assert!((srgba.blue - 1.00).abs() < 0.01, "blue channel mismatch");
    }
}
