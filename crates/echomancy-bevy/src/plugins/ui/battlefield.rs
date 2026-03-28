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
use echomancy_core::prelude::{Action, CardInstanceId, PlayerId};

use super::card::{CardSpawnData, CARD_GAP, spawn_card, spawn_card_with_tappable};
use super::hand::HandRoot;
use crate::plugins::game::{ActivePlayerId, CurrentSnapshot, GameActionMessage, PlayableCards, SnapshotChangedMessage};

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

/// Update system: rebuild battlefield card entities when the snapshot changes
/// or when the `CurrentSnapshot` resource is first added (initial render).
///
/// Despawns all existing card children of each battlefield root, then spawns
/// fresh card entities from `CurrentSnapshot`.
///
/// Untapped lands in the player's battlefield that appear in `PlayableCards.tappable_lands`
/// are rendered with a gold border and receive `TappableLand` + `Button` + `Interaction`
/// components for click-to-tap.
pub(crate) fn rebuild_battlefields(
    mut commands: Commands,
    current_snapshot: Res<CurrentSnapshot>,
    playable_cards: Res<PlayableCards>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    player_battlefield_q: Query<Entity, With<PlayerBattlefieldRoot>>,
    opponent_battlefield_q: Query<Entity, With<OpponentBattlefieldRoot>>,
) {
    // Only rebuild when the snapshot has actually changed.
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let snapshot = &current_snapshot.snapshot;
    let tappable_lands = &playable_cards.result.tappable_lands;

    // ---- Rebuild player battlefield ----
    if let Ok(player_root) = player_battlefield_q.single() {
        // Despawn all existing card children.
        commands.entity(player_root).despawn_children();

        // Spawn new cards as children.
        for card in &snapshot.private_player_state.battlefield {
            let is_tapped = card.tapped.unwrap_or(false);
            let tappable = is_tappable_land(&card.instance_id, tappable_lands);

            let mut card_entity_cmd = spawn_card_with_tappable(
                &mut commands,
                &CardSpawnData {
                    name: &card.name,
                    types: &card.types,
                    power: card.power,
                    toughness: card.toughness,
                    is_tapped,
                    flipped: false,
                },
                if tappable { Some(TAPPABLE_LAND_BORDER_COLOR) } else { None },
            );

            if tappable {
                card_entity_cmd.insert((
                    TappableLand { instance_id: card.instance_id.clone() },
                    Button,
                    Interaction::default(),
                ));
            }

            let card_entity = card_entity_cmd.id();
            commands.entity(player_root).add_child(card_entity);
        }
    }

    // ---- Rebuild opponent battlefield ----
    if let Ok(opponent_root) = opponent_battlefield_q.single() {
        commands.entity(opponent_root).despawn_children();

        if let Some(opponent) = snapshot.opponent_states.first() {
            for card in &opponent.battlefield {
                let is_tapped = card.tapped.unwrap_or(false);
                let card_entity = spawn_card(
                    &mut commands,
                    &CardSpawnData {
                        name: &card.name,
                        types: &card.types,
                        power: card.power,
                        toughness: card.toughness,
                        is_tapped,
                        flipped: true,
                    },
                );
                commands.entity(opponent_root).add_child(card_entity);
            }
        }
    }
}

/// Click handler: detect clicks on `TappableLand` buttons and send `ActivateAbility`.
pub(crate) fn handle_battlefield_land_click(
    query: Query<(&Interaction, &TappableLand), Changed<Interaction>>,
    active_player: Res<ActivePlayerId>,
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
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers battlefield rendering systems.
pub(crate) struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui_root)
            .add_systems(Update, (rebuild_battlefields, handle_battlefield_land_click));
    }
}
