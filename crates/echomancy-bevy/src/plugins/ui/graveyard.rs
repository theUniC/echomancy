//! GraveyardPlugin — toggle overlay panel showing graveyard contents.
//!
//! Layout (center overlay, only shown when a graveyard is selected):
//! ```text
//! ┌──────────────────────────────┐
//! │  Graveyard — Player 1 (3)   │  ← header
//! │  ┌────────────────────────┐  │
//! │  │ Bear          Creature │  │
//! │  │ Giant Growth  Instant  │  │
//! │  │ Forest        Land     │  │
//! │  └────────────────────────┘  │
//! │  [Close]                     │
//! └──────────────────────────────┘
//! ```
//!
//! Triggered by clicking "Your GY: N" or "Opp GY: N" in the HUD. The panel
//! is despawned/respawned on snapshot changes while open, and fully despawned
//! when closed.

use bevy::prelude::*;
use echomancy_core::prelude::CardType;

use crate::plugins::game::{AppState, CurrentSnapshot, PlayerIds, SnapshotChangedMessage};

// ============================================================================
// Colors
// ============================================================================

/// Background color of the graveyard panel container (semi-transparent dark).
const GY_PANEL_BG: Color = Color::srgba(0.08, 0.10, 0.12, 0.94);

/// Background color of a single card row.
const ROW_BG: Color = Color::srgb(0.14, 0.16, 0.20);

/// Text color for the header.
const HEADER_COLOR: Color = Color::srgb(0.75, 0.88, 0.95);

/// Text color for card names.
const CARD_NAME_COLOR: Color = Color::srgb(0.92, 0.92, 0.96);

/// Text color for type line (secondary).
const TYPE_COLOR: Color = Color::srgb(0.60, 0.62, 0.70);

/// Background color for the close button.
const CLOSE_BTN_BG: Color = Color::srgb(0.28, 0.28, 0.32);

// ============================================================================
// State resource
// ============================================================================

/// Which graveyard is currently being viewed (if any).
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub(crate) enum GraveyardViewState {
    /// No graveyard is open.
    #[default]
    Closed,
    /// The human player's own graveyard is open.
    Player,
    /// The opponent's graveyard is open.
    Opponent,
}

// ============================================================================
// Marker components
// ============================================================================

/// Marks the graveyard overlay panel root.
#[derive(Component)]
pub(crate) struct GraveyardPanelRoot;

/// Marks the "Your GY" HUD clickable area.
#[derive(Component)]
pub(crate) struct GraveyardPlayerButton;

/// Marks the "Opp GY" HUD clickable area.
#[derive(Component)]
pub(crate) struct GraveyardOpponentButton;

/// Marks the close button inside the panel.
#[derive(Component)]
pub(crate) struct GraveyardCloseButton;

// ============================================================================
// Pure helper functions (tested without ECS)
// ============================================================================

/// Format the graveyard panel header.
///
/// Examples:
/// - `format_graveyard_header("Player 1", 3)` → `"Graveyard — Player 1 (3)"`
/// - `format_graveyard_header("Opponent", 0)` → `"Graveyard — Opponent (0)"`
pub(crate) fn format_graveyard_header(player_name: &str, card_count: usize) -> String {
    format!("Graveyard \u{2014} {player_name} ({card_count})")
}

/// Format a card's abbreviated type line for the graveyard row.
///
/// Returns the first relevant card type as a short string.
/// Examples: `"Creature"`, `"Instant"`, `"Sorcery"`, `"Land"`, `"Enchantment"`,
/// `"Artifact"`, `"Planeswalker"`, `"Unknown"`.
pub(crate) fn format_card_type_line(types: &[CardType]) -> &'static str {
    // Return the display name for the first type in the slice.
    types
        .iter()
        .map(|t| match t {
            CardType::Creature => "Creature",
            CardType::Instant => "Instant",
            CardType::Sorcery => "Sorcery",
            CardType::Land => "Land",
            CardType::Enchantment => "Enchantment",
            CardType::Artifact => "Artifact",
            CardType::Planeswalker => "Planeswalker",
            CardType::Kindred => "Kindred",
        })
        .next()
        .unwrap_or("Unknown")
}

// ============================================================================
// Systems
// ============================================================================

/// Toggle the graveyard viewer when the "Your GY" HUD button is clicked.
pub(crate) fn toggle_player_graveyard(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<GraveyardPlayerButton>)>,
    mut gy_state: ResMut<GraveyardViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *gy_state = match *gy_state {
                GraveyardViewState::Player => GraveyardViewState::Closed,
                _ => GraveyardViewState::Player,
            };
        }
    }
}

/// Toggle the graveyard viewer when the "Opp GY" HUD button is clicked.
pub(crate) fn toggle_opponent_graveyard(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<GraveyardOpponentButton>)>,
    mut gy_state: ResMut<GraveyardViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *gy_state = match *gy_state {
                GraveyardViewState::Opponent => GraveyardViewState::Closed,
                _ => GraveyardViewState::Opponent,
            };
        }
    }
}

/// Close the graveyard viewer when the close button is clicked.
pub(crate) fn handle_close_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<GraveyardCloseButton>)>,
    mut gy_state: ResMut<GraveyardViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *gy_state = GraveyardViewState::Closed;
        }
    }
}

/// Rebuild the graveyard panel when the snapshot changes or viewer state changes.
///
/// Despawns any existing panel, then spawns a new one if `GraveyardViewState`
/// is not `Closed`.
pub(crate) fn rebuild_graveyard_panel(
    snapshot: Res<CurrentSnapshot>,
    gy_state: Res<GraveyardViewState>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    panel_q: Query<Entity, With<GraveyardPanelRoot>>,
    mut commands: Commands,
    player_ids: Res<PlayerIds>,
) {
    // Rebuild when the snapshot changes OR when the viewer state changed.
    let snapshot_updated = snapshot_changed.read().count() > 0;
    let state_changed = gy_state.is_changed();

    if !snapshot_updated && !state_changed {
        return;
    }

    // Always despawn the existing panel first.
    for entity in &panel_q {
        commands.entity(entity).despawn();
    }

    match *gy_state {
        GraveyardViewState::Closed => {}
        GraveyardViewState::Player => {
            let private = &snapshot.snapshot.private_player_state;
            let player_name = player_ids.name_for(&private.player_id);
            spawn_graveyard_panel(&mut commands, player_name, &private.graveyard);
        }
        GraveyardViewState::Opponent => {
            // In a 2-player game the opponent is always opponent_states[0].
            if let Some(opp) = snapshot.snapshot.opponent_states.first() {
                let opp_name = player_ids.name_for(&opp.player_id);
                spawn_graveyard_panel(&mut commands, opp_name, &opp.graveyard);
            }
        }
    }
}

/// Spawn the graveyard overlay panel populated with the given cards.
fn spawn_graveyard_panel(
    commands: &mut Commands,
    player_name: &str,
    cards: &[echomancy_core::prelude::CardSnapshot],
) {
    commands
        .spawn((
            GraveyardPanelRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                right: Val::Percent(25.0),
                top: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(GY_PANEL_BG),
            ZIndex(20),
        ))
        .with_children(|panel| {
            // Header line.
            panel.spawn((
                Text::new(format_graveyard_header(player_name, cards.len())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(HEADER_COLOR),
            ));

            if cards.is_empty() {
                panel.spawn((
                    Text::new("(empty)"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(TYPE_COLOR),
                ));
            } else {
                // One row per card.
                for card in cards {
                    let type_str = format_card_type_line(&card.types);
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(ROW_BG),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(card.name.clone()),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(CARD_NAME_COLOR),
                            ));
                            row.spawn((
                                Text::new(type_str),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(TYPE_COLOR),
                            ));
                        });
                }
            }

            // Close button.
            panel
                .spawn((
                    GraveyardCloseButton,
                    Button,
                    Interaction::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(CLOSE_BTN_BG),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Close"),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers graveyard viewer systems and the `GraveyardViewState` resource.
pub(crate) struct GraveyardPlugin;

impl Plugin for GraveyardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraveyardViewState>();
        app.add_systems(
            Update,
            (
                toggle_player_graveyard,
                toggle_opponent_graveyard,
                handle_close_button,
                rebuild_graveyard_panel,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

// ============================================================================
// Tests (TDD: written before implementation)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- format_graveyard_header ------------------------------------------

    #[test]
    fn header_with_cards() {
        let result = format_graveyard_header("Player 1", 3);
        assert_eq!(result, "Graveyard \u{2014} Player 1 (3)");
    }

    #[test]
    fn header_empty_graveyard() {
        let result = format_graveyard_header("Opponent", 0);
        assert_eq!(result, "Graveyard \u{2014} Opponent (0)");
    }

    #[test]
    fn header_single_card() {
        let result = format_graveyard_header("Player 2", 1);
        assert_eq!(result, "Graveyard \u{2014} Player 2 (1)");
    }

    // ---- format_card_type_line --------------------------------------------

    #[test]
    fn type_line_creature() {
        assert_eq!(format_card_type_line(&[CardType::Creature]), "Creature");
    }

    #[test]
    fn type_line_instant() {
        assert_eq!(format_card_type_line(&[CardType::Instant]), "Instant");
    }

    #[test]
    fn type_line_sorcery() {
        assert_eq!(format_card_type_line(&[CardType::Sorcery]), "Sorcery");
    }

    #[test]
    fn type_line_land() {
        assert_eq!(format_card_type_line(&[CardType::Land]), "Land");
    }

    #[test]
    fn type_line_enchantment() {
        assert_eq!(format_card_type_line(&[CardType::Enchantment]), "Enchantment");
    }

    #[test]
    fn type_line_artifact() {
        assert_eq!(format_card_type_line(&[CardType::Artifact]), "Artifact");
    }

    #[test]
    fn type_line_empty() {
        assert_eq!(format_card_type_line(&[]), "Unknown");
    }

    #[test]
    fn type_line_artifact_first_returns_artifact() {
        // When Artifact appears before Creature in the slice, Artifact is shown.
        assert_eq!(
            format_card_type_line(&[CardType::Artifact, CardType::Creature]),
            "Artifact"
        );
    }

    #[test]
    fn type_line_creature_first_returns_creature() {
        // When Creature appears before Artifact, Creature is shown.
        assert_eq!(
            format_card_type_line(&[CardType::Creature, CardType::Artifact]),
            "Creature"
        );
    }
}
