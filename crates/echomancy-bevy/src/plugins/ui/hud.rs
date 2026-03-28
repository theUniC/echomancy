//! HudPlugin — right-side panel showing turn info, life totals, priority, and action buttons.
//!
//! Layout (right panel, fixed 250px width):
//! ```text
//! ┌──────────────────────────────┐
//! │  Turn 1 - First Main         │  Turn label
//! │  Your Priority               │  Priority indicator (green/gray)
//! │  [Pass Priority]             │  Button
//! │  [End Turn]                  │  Button
//! │  ─────────────               │  separator
//! │  You: 20                     │  Player life
//! │  Opponent: 20                │  Opponent life
//! │  ─────────────               │
//! │  Opp Hand: 7 cards           │
//! │  Your Graveyard: 0           │
//! │  Opp Graveyard: 0            │
//! │  ─────────────               │
//! │  [error message if any]      │  Red error area
//! └──────────────────────────────┘
//! ```
//!
//! Buttons are only active when the player has priority. Disabled buttons are
//! shown in a muted color and clicks are ignored.

use bevy::prelude::*;
use echomancy_core::prelude::{Action, PlayerId, Step};

use crate::plugins::game::{
    ActivePlayerId, CurrentSnapshot, ErrorMessage, GameActionMessage, PlayerIds,
    SnapshotChangedMessage,
};

// ============================================================================
// Constants
// ============================================================================

/// Width of the right-side HUD panel in pixels.
pub(crate) const HUD_PANEL_WIDTH: f32 = 250.0;

/// Background color for the HUD panel.
const HUD_BG: Color = Color::srgb(0.12, 0.12, 0.16);

/// Text color for informational labels.
const LABEL_COLOR: Color = Color::srgb(0.85, 0.85, 0.90);

/// Text color for secondary/muted info.
const MUTED_COLOR: Color = Color::srgb(0.55, 0.55, 0.60);

/// Background color for the "active priority" indicator.
const PRIORITY_ACTIVE_BG: Color = Color::srgb(0.15, 0.45, 0.15);

/// Background color for the "no priority" indicator.
const PRIORITY_INACTIVE_BG: Color = Color::srgb(0.25, 0.25, 0.28);

/// Background color for enabled action buttons.
const BUTTON_ENABLED_BG: Color = Color::srgb(0.20, 0.35, 0.55);

/// Background color for disabled (no priority) buttons.
const BUTTON_DISABLED_BG: Color = Color::srgb(0.22, 0.22, 0.26);

/// Background color for the error message box.
const ERROR_BG: Color = Color::srgb(0.55, 0.10, 0.10);

// ============================================================================
// Marker components
// ============================================================================

/// Marks the HUD panel root node.
#[derive(Component)]
pub(crate) struct HudRoot;

/// Marks the "Playing as: ..." label showing whose perspective the UI displays.
#[derive(Component)]
pub(crate) struct HudActivePlayerLabel;

/// Marks the turn-info label (rebuilt on snapshot change).
#[derive(Component)]
pub(crate) struct HudTurnLabel;

/// Marks the priority indicator text.
#[derive(Component)]
pub(crate) struct HudPriorityLabel;

/// Marks the priority indicator container node (background changes color).
#[derive(Component)]
pub(crate) struct HudPriorityBox;

/// Marks the "Pass Priority" button.
#[derive(Component)]
pub(crate) struct PassPriorityButton;

/// Marks the "End Turn" button.
#[derive(Component)]
pub(crate) struct EndTurnButton;

/// Marks the player life total label.
#[derive(Component)]
pub(crate) struct HudPlayerLife;

/// Marks the opponent life total label.
#[derive(Component)]
pub(crate) struct HudOpponentLife;

/// Marks the opponent hand count label.
#[derive(Component)]
pub(crate) struct HudOpponentHandCount;

/// Marks the player graveyard count label.
#[derive(Component)]
pub(crate) struct HudPlayerGraveyard;

/// Marks the opponent graveyard count label.
#[derive(Component)]
pub(crate) struct HudOpponentGraveyard;

/// Marks the error message text node.
#[derive(Component)]
pub(crate) struct HudErrorText;

/// Marks the error message container (hidden when no error).
#[derive(Component)]
pub(crate) struct HudErrorBox;

// ============================================================================
// Pure helper functions (testable without ECS)
// ============================================================================

/// Convert a `Step` to a short human-readable display name.
pub(crate) fn step_display_name(step: Step) -> &'static str {
    match step {
        Step::Untap => "Untap",
        Step::Upkeep => "Upkeep",
        Step::Draw => "Draw",
        Step::FirstMain => "First Main",
        Step::BeginningOfCombat => "Begin Combat",
        Step::DeclareAttackers => "Attackers",
        Step::DeclareBlockers => "Blockers",
        Step::CombatDamage => "Combat Damage",
        Step::EndOfCombat => "End Combat",
        Step::SecondMain => "Second Main",
        Step::EndStep => "End Step",
        Step::Cleanup => "Cleanup",
    }
}

/// Return `true` if the given player currently has priority.
///
/// `priority_player_id` is `None` when no player currently holds priority
/// (e.g. during auto-resolved steps).
pub(crate) fn has_priority(priority_player_id: Option<&str>, active_player_id: &str) -> bool {
    priority_player_id
        .map(|id| id == active_player_id)
        .unwrap_or(false)
}

/// Format the turn-info label shown at the top of the HUD.
///
/// Example: `"Turn 3 — First Main"`
pub(crate) fn format_turn_label(turn_number: u32, step: Step) -> String {
    format!("Turn {} \u{2014} {}", turn_number, step_display_name(step))
}

/// Format the "Playing as" label shown in the HUD.
///
/// Example: `"Playing as: Player 1"`
pub(crate) fn format_active_player_label(player_name: &str) -> String {
    format!("Playing as: {player_name}")
}

// ============================================================================
// Spawn systems
// ============================================================================

/// Startup system: spawn the HUD panel root (no data, just structure).
///
/// The HUD is positioned absolutely on the right edge so it overlays the game
/// board without affecting the flex layout of `BattlefieldRoot`.
///
/// All data labels are spawned as empty and updated by `update_hud`.
pub(crate) fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(HUD_PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(HUD_BG),
        ))
        .with_children(|panel| {
            // "Playing as" label — shows whose perspective the UI is from.
            panel.spawn((
                HudActivePlayerLabel,
                Text::new("Playing as: ..."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(MUTED_COLOR),
            ));

            // Turn info label
            panel.spawn((
                HudTurnLabel,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(LABEL_COLOR),
            ));

            // Priority indicator box
            panel
                .spawn((
                    HudPriorityBox,
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(PRIORITY_INACTIVE_BG),
                ))
                .with_children(|pbox| {
                    pbox.spawn((
                        HudPriorityLabel,
                        Text::new(""),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(LABEL_COLOR),
                    ));
                });

            // Pass Priority button
            panel
                .spawn((
                    PassPriorityButton,
                    Button,
                    Interaction::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_DISABLED_BG),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Pass Priority"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(LABEL_COLOR),
                    ));
                });

            // End Turn button
            panel
                .spawn((
                    EndTurnButton,
                    Button,
                    Interaction::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_DISABLED_BG),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("End Turn"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(LABEL_COLOR),
                    ));
                });

            // Separator
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.30, 0.30, 0.35)),
            ));

            // Player life total
            panel.spawn((
                HudPlayerLife,
                Text::new("You: —"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(LABEL_COLOR),
            ));

            // Opponent life total
            panel.spawn((
                HudOpponentLife,
                Text::new("Opponent: —"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(LABEL_COLOR),
            ));

            // Separator
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.30, 0.30, 0.35)),
            ));

            // Opponent hand count
            panel.spawn((
                HudOpponentHandCount,
                Text::new("Opp Hand: —"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(MUTED_COLOR),
            ));

            // Player graveyard count
            panel.spawn((
                HudPlayerGraveyard,
                Text::new("Your GY: 0"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(MUTED_COLOR),
            ));

            // Opponent graveyard count
            panel.spawn((
                HudOpponentGraveyard,
                Text::new("Opp GY: 0"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(MUTED_COLOR),
            ));

            // Separator
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.30, 0.30, 0.35)),
            ));

            // Error message box (always present, visibility-toggled)
            panel
                .spawn((
                    HudErrorBox,
                    Node {
                        padding: UiRect::all(Val::Px(6.0)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(ERROR_BG),
                ))
                .with_children(|ebox| {
                    ebox.spawn((
                        HudErrorText,
                        Text::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

// ============================================================================
// Type aliases — reduce clippy type_complexity warnings on query parameters
// ============================================================================

type ActivePlayerLabelQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, With<HudActivePlayerLabel>>;
type TurnLabelQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<HudTurnLabel>, Without<HudActivePlayerLabel>)>;
type PriorityLabelQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<HudPriorityLabel>, Without<HudTurnLabel>, Without<HudActivePlayerLabel>)>;
type PriorityBoxQuery<'w, 's> = Query<'w, 's, &'static mut BackgroundColor, With<HudPriorityBox>>;
type PassBtnQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut BackgroundColor,
    (With<PassPriorityButton>, Without<HudPriorityBox>, Without<EndTurnButton>),
>;
type EndTurnBtnQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut BackgroundColor,
    (With<EndTurnButton>, Without<HudPriorityBox>, Without<PassPriorityButton>),
>;
type PlayerLifeQuery<'w, 's> =
    Query<'w, 's, &'static mut Text, (With<HudPlayerLife>, Without<HudPriorityLabel>, Without<HudTurnLabel>, Without<HudActivePlayerLabel>)>;
type OpponentLifeQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (With<HudOpponentLife>, Without<HudPlayerLife>, Without<HudPriorityLabel>, Without<HudTurnLabel>, Without<HudActivePlayerLabel>),
>;
type OppHandQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<HudOpponentHandCount>,
        Without<HudOpponentLife>,
        Without<HudPlayerLife>,
        Without<HudPriorityLabel>,
        Without<HudTurnLabel>,
        Without<HudActivePlayerLabel>,
    ),
>;
type PlayerGraveyardQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<HudPlayerGraveyard>,
        Without<HudOpponentHandCount>,
        Without<HudOpponentLife>,
        Without<HudPlayerLife>,
        Without<HudPriorityLabel>,
        Without<HudTurnLabel>,
        Without<HudActivePlayerLabel>,
    ),
>;
type OppGraveyardQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<HudOpponentGraveyard>,
        Without<HudPlayerGraveyard>,
        Without<HudOpponentHandCount>,
        Without<HudOpponentLife>,
        Without<HudPlayerLife>,
        Without<HudPriorityLabel>,
        Without<HudTurnLabel>,
        Without<HudActivePlayerLabel>,
    ),
>;

// ============================================================================
// Update systems
// ============================================================================

/// Update system: refresh all HUD text labels when the snapshot changes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_hud(
    current_snapshot: Res<CurrentSnapshot>,
    active_player: Res<ActivePlayerId>,
    player_ids: Res<PlayerIds>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    mut active_player_label_q: ActivePlayerLabelQuery,
    mut turn_label_q: TurnLabelQuery,
    mut priority_label_q: PriorityLabelQuery,
    mut priority_box_q: PriorityBoxQuery,
    mut pass_btn_q: PassBtnQuery,
    mut end_turn_btn_q: EndTurnBtnQuery,
    mut player_life_q: PlayerLifeQuery,
    mut opponent_life_q: OpponentLifeQuery,
    mut opp_hand_q: OppHandQuery,
    mut player_gy_q: PlayerGraveyardQuery,
    mut opp_gy_q: OppGraveyardQuery,
) {
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let snapshot = &current_snapshot.snapshot;
    let pub_state = &snapshot.public_game_state;
    let priv_state = &snapshot.private_player_state;
    let player_id = &active_player.player_id;

    // "Playing as" label
    if let Ok(mut text) = active_player_label_q.single_mut() {
        let name = player_ids.name_for(player_id);
        *text = Text::new(format_active_player_label(name));
    }

    // Turn label
    if let Ok(mut text) = turn_label_q.single_mut() {
        *text = Text::new(format_turn_label(pub_state.turn_number, pub_state.current_step));
    }

    // Priority
    let player_has_priority = has_priority(pub_state.priority_player_id.as_deref(), player_id);

    if let Ok(mut text) = priority_label_q.single_mut() {
        *text = Text::new(if player_has_priority {
            "Your Priority"
        } else {
            "Opponent's Priority"
        });
    }

    if let Ok(mut bg) = priority_box_q.single_mut() {
        *bg = BackgroundColor(if player_has_priority {
            PRIORITY_ACTIVE_BG
        } else {
            PRIORITY_INACTIVE_BG
        });
    }

    let btn_color = if player_has_priority {
        BUTTON_ENABLED_BG
    } else {
        BUTTON_DISABLED_BG
    };

    if let Ok(mut bg) = pass_btn_q.single_mut() {
        *bg = BackgroundColor(btn_color);
    }

    if let Ok(mut bg) = end_turn_btn_q.single_mut() {
        *bg = BackgroundColor(btn_color);
    }

    // Life totals
    if let Ok(mut text) = player_life_q.single_mut() {
        *text = Text::new(format!("You: {}", priv_state.life_total));
    }

    if let Some(opponent) = snapshot.opponent_states.first() {
        if let Ok(mut text) = opponent_life_q.single_mut() {
            *text = Text::new(format!("Opponent: {}", opponent.life_total));
        }
        if let Ok(mut text) = opp_hand_q.single_mut() {
            *text = Text::new(format!("Opp Hand: {} card(s)", opponent.hand_size));
        }
        if let Ok(mut text) = opp_gy_q.single_mut() {
            *text = Text::new(format!("Opp GY: {}", opponent.graveyard.len()));
        }
    }

    if let Ok(mut text) = player_gy_q.single_mut() {
        *text = Text::new(format!("Your GY: {}", priv_state.graveyard.len()));
    }
}

/// Update system: display or clear the error message from `ErrorMessage` resource.
pub(crate) fn update_error_display(
    error_message: Res<ErrorMessage>,
    mut error_box_q: Query<&mut Node, With<HudErrorBox>>,
    mut error_text_q: Query<&mut Text, With<HudErrorText>>,
) {
    if !error_message.is_changed() {
        return;
    }

    let message = error_message.message.as_deref().unwrap_or("");
    let visible = !message.is_empty();

    if let Ok(mut node) = error_box_q.single_mut() {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut text) = error_text_q.single_mut() {
        *text = Text::new(message);
    }
}

/// Update system: handle Pass Priority button clicks.
pub(crate) fn handle_pass_priority_click(
    query: Query<&Interaction, (Changed<Interaction>, With<PassPriorityButton>)>,
    current_snapshot: Res<CurrentSnapshot>,
    active_player: Res<ActivePlayerId>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            let pub_state = &current_snapshot.snapshot.public_game_state;
            let player_has_priority =
                has_priority(pub_state.priority_player_id.as_deref(), &active_player.player_id);

            if player_has_priority {
                action_writer.write(GameActionMessage(Action::AdvanceStep {
                    player_id: PlayerId::new(&active_player.player_id),
                }));
            }
        }
    }
}

/// Update system: handle End Turn button clicks.
pub(crate) fn handle_end_turn_click(
    query: Query<&Interaction, (Changed<Interaction>, With<EndTurnButton>)>,
    current_snapshot: Res<CurrentSnapshot>,
    active_player: Res<ActivePlayerId>,
    mut action_writer: MessageWriter<GameActionMessage>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            let pub_state = &current_snapshot.snapshot.public_game_state;
            let player_has_priority =
                has_priority(pub_state.priority_player_id.as_deref(), &active_player.player_id);

            if player_has_priority {
                action_writer.write(GameActionMessage(Action::EndTurn {
                    player_id: PlayerId::new(&active_player.player_id),
                }));
            }
        }
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers HUD systems.
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (update_hud, update_error_display, handle_pass_priority_click, handle_end_turn_click),
            );
    }
}

// ============================================================================
// Tests (TDD: written before implementation)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- step_display_name -------------------------------------------------

    #[test]
    fn untap_step_display_name() {
        assert_eq!(step_display_name(Step::Untap), "Untap");
    }

    #[test]
    fn upkeep_step_display_name() {
        assert_eq!(step_display_name(Step::Upkeep), "Upkeep");
    }

    #[test]
    fn draw_step_display_name() {
        assert_eq!(step_display_name(Step::Draw), "Draw");
    }

    #[test]
    fn first_main_step_display_name() {
        assert_eq!(step_display_name(Step::FirstMain), "First Main");
    }

    #[test]
    fn beginning_of_combat_step_display_name() {
        assert_eq!(step_display_name(Step::BeginningOfCombat), "Begin Combat");
    }

    #[test]
    fn declare_attackers_step_display_name() {
        assert_eq!(step_display_name(Step::DeclareAttackers), "Attackers");
    }

    #[test]
    fn declare_blockers_step_display_name() {
        assert_eq!(step_display_name(Step::DeclareBlockers), "Blockers");
    }

    #[test]
    fn combat_damage_step_display_name() {
        assert_eq!(step_display_name(Step::CombatDamage), "Combat Damage");
    }

    #[test]
    fn end_of_combat_step_display_name() {
        assert_eq!(step_display_name(Step::EndOfCombat), "End Combat");
    }

    #[test]
    fn second_main_step_display_name() {
        assert_eq!(step_display_name(Step::SecondMain), "Second Main");
    }

    #[test]
    fn end_step_display_name() {
        assert_eq!(step_display_name(Step::EndStep), "End Step");
    }

    #[test]
    fn cleanup_step_display_name() {
        assert_eq!(step_display_name(Step::Cleanup), "Cleanup");
    }

    // ---- has_priority ------------------------------------------------------

    #[test]
    fn player_with_matching_priority_id_has_priority() {
        assert!(has_priority(Some("player-1"), "player-1"));
    }

    #[test]
    fn player_with_different_priority_id_has_no_priority() {
        assert!(!has_priority(Some("player-2"), "player-1"));
    }

    #[test]
    fn no_priority_player_means_no_priority() {
        assert!(!has_priority(None, "player-1"));
    }

    // ---- format_turn_label -------------------------------------------------

    #[test]
    fn format_turn_label_first_turn_untap() {
        let label = format_turn_label(1, Step::Untap);
        assert_eq!(label, "Turn 1 \u{2014} Untap");
    }

    #[test]
    fn format_turn_label_third_turn_first_main() {
        let label = format_turn_label(3, Step::FirstMain);
        assert_eq!(label, "Turn 3 \u{2014} First Main");
    }

    #[test]
    fn format_turn_label_contains_turn_number() {
        let label = format_turn_label(7, Step::Draw);
        assert!(label.contains("7"), "Label should contain the turn number");
    }

    #[test]
    fn format_turn_label_contains_step_name() {
        let label = format_turn_label(1, Step::CombatDamage);
        assert!(
            label.contains("Combat Damage"),
            "Label should contain the step name"
        );
    }

    // ---- format_active_player_label ----------------------------------------

    #[test]
    fn format_active_player_label_includes_player_name() {
        let label = format_active_player_label("Player 1");
        assert_eq!(label, "Playing as: Player 1");
    }

    #[test]
    fn format_active_player_label_includes_player_2_name() {
        let label = format_active_player_label("Player 2");
        assert_eq!(label, "Playing as: Player 2");
    }
}
