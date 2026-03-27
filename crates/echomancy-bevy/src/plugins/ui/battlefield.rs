//! BattlefieldPlugin — renders two horizontal rows of cards.
//!
//! Layout (top to bottom):
//! ```text
//! ┌─────────────────────────────┐
//! │  Opponent Battlefield (top) │  cards rotated 180°
//! ├─────────────────────────────┤
//! │  Player Battlefield (mid)   │  cards normal
//! ├─────────────────────────────┤
//! │  Hand placeholder (Phase 8.3)│
//! └─────────────────────────────┘
//! ```
//!
//! The system `rebuild_battlefields` despawns and re-spawns all card entities
//! whenever a `SnapshotChangedMessage` is received.

use bevy::prelude::*;

use super::card::{CardSpawnData, CARD_GAP, spawn_card};
use crate::plugins::game::{CurrentSnapshot, SnapshotChangedMessage};

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

// ============================================================================
// Colors
// ============================================================================

/// Background color for the opponent battlefield zone.
const OPPONENT_ZONE_BG: Color = Color::srgb(0.08, 0.08, 0.10);

/// Background color for the player battlefield zone.
const PLAYER_ZONE_BG: Color = Color::srgb(0.06, 0.10, 0.08);

/// Background color for the hand placeholder zone.
const HAND_ZONE_BG: Color = Color::srgb(0.05, 0.05, 0.07);

// ============================================================================
// Systems
// ============================================================================

/// Startup system: create the root layout with 3 zones.
///
/// Zones:
/// 1. Opponent battlefield (top, 35% height)
/// 2. Player battlefield (middle, 35% height)
/// 3. Hand placeholder (bottom, 30% height — filled in Phase 8.3)
pub(crate) fn spawn_ui_root(mut commands: Commands) {
    commands
        .spawn((
            BattlefieldRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
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

            // Hand placeholder (bottom, 30% — Phase 8.3)
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(HAND_ZONE_BG),
            ));
        });
}

/// Update system: rebuild battlefield card entities when the snapshot changes.
///
/// Despawns all existing card children of each battlefield root, then spawns
/// fresh card entities from `CurrentSnapshot`.
pub(crate) fn rebuild_battlefields(
    mut commands: Commands,
    current_snapshot: Res<CurrentSnapshot>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    player_battlefield_q: Query<Entity, With<PlayerBattlefieldRoot>>,
    opponent_battlefield_q: Query<Entity, With<OpponentBattlefieldRoot>>,
) {
    // Only rebuild when the snapshot has actually changed.
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let snapshot = &current_snapshot.snapshot;

    // ---- Rebuild player battlefield ----
    if let Ok(player_root) = player_battlefield_q.single() {
        // Despawn all existing card children.
        commands.entity(player_root).despawn_children();

        // Spawn new cards as children.
        for card in &snapshot.private_player_state.battlefield {
            let is_tapped = card.tapped.unwrap_or(false);
            let card_entity = spawn_card(
                &mut commands,
                &CardSpawnData {
                    name: &card.name,
                    types: &card.types,
                    power: card.power,
                    toughness: card.toughness,
                    is_tapped,
                    flipped: false,
                },
            );
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

// ============================================================================
// Plugin
// ============================================================================

/// Registers battlefield rendering systems.
pub(crate) struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui_root)
            .add_systems(Update, rebuild_battlefields);
    }
}
