//! ExilePlugin — toggle overlay panel showing exile zone contents.
//!
//! Layout (center overlay, only shown when an exile zone is selected):
//! ```text
//! ┌──────────────────────────────┐
//! │  Exile — Player 1 (2)       │  ← header
//! │  ┌────────────────────────┐  │
//! │  │ Skullclamp   Artifact  │  │
//! │  │ Path to Exile  Instant │  │
//! │  └────────────────────────┘  │
//! │  [Close]                     │
//! └──────────────────────────────┘
//! ```
//!
//! Triggered by clicking "Your Exile: N" or "Opp Exile: N" in the HUD. The
//! panel is despawned/respawned on snapshot changes while open, and fully
//! despawned when closed.

use bevy::prelude::*;
use echomancy_core::prelude::CardType;

use crate::plugins::game::{AppState, CurrentSnapshot, PlayerIds, SnapshotChangedMessage};

// ============================================================================
// Colors
// ============================================================================

/// Background color of the exile panel container (semi-transparent dark amber).
const EXILE_PANEL_BG: Color = Color::srgba(0.10, 0.09, 0.08, 0.94);

/// Background color of a single card row.
const ROW_BG: Color = Color::srgb(0.16, 0.14, 0.12);

/// Text color for the header.
const HEADER_COLOR: Color = Color::srgb(0.95, 0.88, 0.70);

/// Text color for card names.
const CARD_NAME_COLOR: Color = Color::srgb(0.92, 0.92, 0.96);

/// Text color for type line (secondary).
const TYPE_COLOR: Color = Color::srgb(0.65, 0.60, 0.55);

/// Background color for the close button.
const CLOSE_BTN_BG: Color = Color::srgb(0.30, 0.26, 0.22);

// ============================================================================
// State resource
// ============================================================================

/// Which exile zone is currently being viewed (if any).
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExileViewState {
    /// No exile zone is open.
    #[default]
    Closed,
    /// The human player's own exile is open.
    Player,
    /// The opponent's exile is open.
    Opponent,
}

// ============================================================================
// Marker components
// ============================================================================

/// Marks the exile overlay panel root.
#[derive(Component)]
pub(crate) struct ExilePanelRoot;

/// Marks the "Your Exile" HUD clickable area.
#[derive(Component)]
pub(crate) struct ExilePlayerButton;

/// Marks the "Opp Exile" HUD clickable area.
#[derive(Component)]
pub(crate) struct ExileOpponentButton;

/// Marks the close button inside the panel.
#[derive(Component)]
pub(crate) struct ExileCloseButton;

// ============================================================================
// Pure helper functions (tested without ECS)
// ============================================================================

/// Format the exile panel header.
///
/// Examples:
/// - `format_exile_header("Player 1", 2)` → `"Exile — Player 1 (2)"`
/// - `format_exile_header("Opponent", 0)` → `"Exile — Opponent (0)"`
pub(crate) fn format_exile_header(player_name: &str, card_count: usize) -> String {
    format!("Exile \u{2014} {player_name} ({card_count})")
}

/// Format a card's abbreviated type line for the exile row.
///
/// Returns the first relevant card type as a short string.
/// Examples: `"Creature"`, `"Instant"`, `"Sorcery"`, `"Land"`, `"Enchantment"`,
/// `"Artifact"`, `"Planeswalker"`, `"Unknown"`.
pub(crate) fn format_exile_type_line(types: &[CardType]) -> &'static str {
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

/// Toggle the exile viewer when the "Your Exile" HUD button is clicked.
pub(crate) fn toggle_player_exile(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ExilePlayerButton>)>,
    mut exile_state: ResMut<ExileViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *exile_state = match *exile_state {
                ExileViewState::Player => ExileViewState::Closed,
                _ => ExileViewState::Player,
            };
        }
    }
}

/// Toggle the exile viewer when the "Opp Exile" HUD button is clicked.
pub(crate) fn toggle_opponent_exile(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ExileOpponentButton>)>,
    mut exile_state: ResMut<ExileViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *exile_state = match *exile_state {
                ExileViewState::Opponent => ExileViewState::Closed,
                _ => ExileViewState::Opponent,
            };
        }
    }
}

/// Close the exile viewer when the close button is clicked.
pub(crate) fn handle_exile_close_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ExileCloseButton>)>,
    mut exile_state: ResMut<ExileViewState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            *exile_state = ExileViewState::Closed;
        }
    }
}

/// Rebuild the exile panel when the snapshot changes or viewer state changes.
///
/// Despawns any existing panel, then spawns a new one if `ExileViewState`
/// is not `Closed`.
pub(crate) fn rebuild_exile_panel(
    snapshot: Res<CurrentSnapshot>,
    exile_state: Res<ExileViewState>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    panel_q: Query<Entity, With<ExilePanelRoot>>,
    mut commands: Commands,
    player_ids: Res<PlayerIds>,
) {
    let snapshot_updated = snapshot_changed.read().count() > 0;
    let state_changed = exile_state.is_changed();

    if !snapshot_updated && !state_changed {
        return;
    }

    // Always despawn the existing panel first.
    for entity in &panel_q {
        commands.entity(entity).despawn();
    }

    match *exile_state {
        ExileViewState::Closed => {}
        ExileViewState::Player => {
            let private = &snapshot.snapshot.private_player_state;
            let player_name = player_ids.name_for(&private.player_id);
            spawn_exile_panel(&mut commands, player_name, &private.exile);
        }
        ExileViewState::Opponent => {
            if let Some(opp) = snapshot.snapshot.opponent_states.first() {
                let opp_name = player_ids.name_for(&opp.player_id);
                spawn_exile_panel(&mut commands, opp_name, &opp.exile);
            }
        }
    }
}

/// Spawn the exile overlay panel populated with the given cards.
fn spawn_exile_panel(
    commands: &mut Commands,
    player_name: &str,
    cards: &[echomancy_core::prelude::CardSnapshot],
) {
    commands
        .spawn((
            ExilePanelRoot,
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
            BackgroundColor(EXILE_PANEL_BG),
            ZIndex(20),
        ))
        .with_children(|panel| {
            // Header line.
            panel.spawn((
                Text::new(format_exile_header(player_name, cards.len())),
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
                for card in cards {
                    let type_str = format_exile_type_line(&card.types);
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
                    ExileCloseButton,
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

/// Registers exile viewer systems and the `ExileViewState` resource.
pub(crate) struct ExilePlugin;

impl Plugin for ExilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExileViewState>();
        app.add_systems(
            Update,
            (
                toggle_player_exile,
                toggle_opponent_exile,
                handle_exile_close_button,
                rebuild_exile_panel,
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

    // ---- format_exile_header -----------------------------------------------

    #[test]
    fn header_with_cards() {
        let result = format_exile_header("Player 1", 2);
        assert_eq!(result, "Exile \u{2014} Player 1 (2)");
    }

    #[test]
    fn header_empty_exile() {
        let result = format_exile_header("Opponent", 0);
        assert_eq!(result, "Exile \u{2014} Opponent (0)");
    }

    #[test]
    fn header_single_card() {
        let result = format_exile_header("Player 2", 1);
        assert_eq!(result, "Exile \u{2014} Player 2 (1)");
    }

    // ---- format_exile_type_line -------------------------------------------

    #[test]
    fn type_line_creature() {
        assert_eq!(format_exile_type_line(&[CardType::Creature]), "Creature");
    }

    #[test]
    fn type_line_instant() {
        assert_eq!(format_exile_type_line(&[CardType::Instant]), "Instant");
    }

    #[test]
    fn type_line_sorcery() {
        assert_eq!(format_exile_type_line(&[CardType::Sorcery]), "Sorcery");
    }

    #[test]
    fn type_line_land() {
        assert_eq!(format_exile_type_line(&[CardType::Land]), "Land");
    }

    #[test]
    fn type_line_enchantment() {
        assert_eq!(format_exile_type_line(&[CardType::Enchantment]), "Enchantment");
    }

    #[test]
    fn type_line_artifact() {
        assert_eq!(format_exile_type_line(&[CardType::Artifact]), "Artifact");
    }

    #[test]
    fn type_line_empty() {
        assert_eq!(format_exile_type_line(&[]), "Unknown");
    }

    #[test]
    fn type_line_creature_first() {
        assert_eq!(
            format_exile_type_line(&[CardType::Creature, CardType::Artifact]),
            "Creature"
        );
    }

    #[test]
    fn type_line_artifact_first() {
        assert_eq!(
            format_exile_type_line(&[CardType::Artifact, CardType::Creature]),
            "Artifact"
        );
    }
}
