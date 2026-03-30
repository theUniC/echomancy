//! MulliganPlugin — renders the mulligan screen and handles P1's decisions.
//!
//! Layout:
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  [Card1][Card2][Card3][Card4][Card5][Card6][Card7]   │  P1's hand
//! │                                                      │
//! │  Opening hand — keep or mulligan?                    │  status label
//! │                                                      │
//! │           [Keep]  [Mulligan]                         │  buttons
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! After P1 mulligans N times and clicks Keep:
//! - Buttons are replaced by: "Put N card(s) on the bottom. Click a card."
//! - Each card click dispatches `PutCardOnBottom`.
//! - When all cards are placed, the system transitions to `AppState::InGame`.
//!
//! All entities use `StateScoped(AppState::Mulligan)` so they despawn
//! automatically when leaving the Mulligan state.

use bevy::prelude::*;
use bevy::state::prelude::DespawnOnExit;
use echomancy_core::prelude::{Action, CardInstanceId, PlayerId};

use super::card::{CARD_BORDER, CARD_HEIGHT, CARD_WIDTH, CardNode, card_background_color, card_border_color, card_pt_text, card_type_line};
use crate::plugins::game::{
    AppState, CurrentSnapshot, GameActionMessage, PlayerIds, SnapshotChangedMessage,
};

// ============================================================================
// Constants
// ============================================================================

const BG_COLOR: Color = Color::srgb(0.08, 0.08, 0.12);
const LABEL_COLOR: Color = Color::srgb(0.88, 0.88, 0.92);
const BUTTON_BG: Color = Color::srgb(0.18, 0.35, 0.55);
const BUTTON_HOVER_BG: Color = Color::srgb(0.25, 0.45, 0.70);
const BUTTON_PRESSED_BG: Color = Color::srgb(0.12, 0.25, 0.40);
/// Green border for cards the player can click to put on the bottom.
const SELECTABLE_BORDER: Color = Color::srgb(0.20, 0.90, 0.30);
/// Standard instruction text color.
const INSTRUCTION_COLOR: Color = Color::srgb(0.90, 0.80, 0.30);

// ============================================================================
// Marker components
// ============================================================================

/// Root node for the entire mulligan screen.
#[derive(Component)]
struct MulliganRoot;

/// Marker for the status / instruction label.
#[derive(Component)]
struct MulliganStatusLabel;

/// Marker for the hand cards container.
#[derive(Component)]
struct MulliganHandRoot;

/// Marker for the buttons row.
#[derive(Component)]
struct MulliganButtonRow;

/// Marker component on the Keep button.
#[derive(Component)]
struct KeepButton;

/// Marker component on the Mulligan button.
#[derive(Component)]
struct MulliganButton;

/// Marks a hand card that is clickable for put-back (during put-back sub-step).
#[derive(Component, Clone)]
struct PutBackCard {
    instance_id: String,
}

// ============================================================================
// Startup system — spawn mulligan screen
// ============================================================================

/// Startup system: builds the full mulligan screen UI tree.
fn setup_mulligan_screen(
    mut commands: Commands,
    snapshot: Res<CurrentSnapshot>,
    _player_ids: Res<PlayerIds>,
) {
    let hand = &snapshot.snapshot.private_player_state.hand;
    let mulligan_info = &snapshot.snapshot.mulligan_info;

    let mulligan_count = mulligan_info.as_ref().map(|m| m.mulligan_count).unwrap_or(0);
    let in_put_back = mulligan_info
        .as_ref()
        .map(|m| m.has_kept && m.cards_to_put_back > 0)
        .unwrap_or(false);
    let cards_to_put_back = mulligan_info
        .as_ref()
        .map(|m| m.cards_to_put_back)
        .unwrap_or(0);

    let status_text = status_label_text(mulligan_count, in_put_back, cards_to_put_back);

    commands.spawn((
        MulliganRoot,
        DespawnOnExit(AppState::Mulligan),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(BG_COLOR),
    ))
    .with_children(|root| {
        // Hand row
        root.spawn((
            MulliganHandRoot,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|hand_row| {
            for card in hand.iter() {
                let border_color = card_border_color(&card.types);
                let bg_color = card_background_color(&card.types);
                let type_line = card_type_line(&card.types);
                let pt_text = card_pt_text(card.power, card.toughness);
                let name = card.name.clone();
                let instance_id = card.instance_id.clone();

                let mut entity_cmd = hand_row.spawn((
                    CardNode,
                    Node {
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        border: UiRect::all(Val::Px(CARD_BORDER)),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BorderColor::all(if in_put_back { SELECTABLE_BORDER } else { border_color }),
                    BackgroundColor(bg_color),
                ));

                if in_put_back {
                    entity_cmd.insert((
                        Button,
                        PutBackCard { instance_id: instance_id.clone() },
                    ));
                }

                entity_cmd.with_children(|card_node| {
                    card_node.spawn((
                        Text::new(name),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    card_node.spawn((
                        Text::new(type_line),
                        TextFont { font_size: 9.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    if let Some(pt) = pt_text {
                        card_node.spawn((
                            Text::new(pt),
                            TextFont { font_size: 10.0, ..default() },
                            TextColor(Color::srgb(0.9, 0.9, 0.5)),
                        ));
                    }
                });
            }
        });

        // Status label
        root.spawn((
            MulliganStatusLabel,
            Text::new(status_text),
            TextFont { font_size: 18.0, ..default() },
            TextColor(if in_put_back { INSTRUCTION_COLOR } else { LABEL_COLOR }),
        ));

        // Buttons row (hidden in put-back mode)
        if !in_put_back {
            let mulligan_label = if mulligan_count == 0 {
                "Mulligan".to_owned()
            } else {
                format!("Mulligan (to {})", 7u32.saturating_sub(mulligan_count + 1))
            };

            root.spawn((
                MulliganButtonRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
            ))
            .with_children(|btn_row| {
                // Keep button
                btn_row
                    .spawn((
                        KeepButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Keep"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                // Mulligan button
                btn_row
                    .spawn((
                        MulliganButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(mulligan_label),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        }
    });
}

// ============================================================================
// Rebuild system — refreshes the screen on snapshot change
// ============================================================================

/// Update system: rebuild the mulligan screen whenever the snapshot changes.
fn rebuild_mulligan_screen(
    mut commands: Commands,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    snapshot: Res<CurrentSnapshot>,
    _player_ids: Res<PlayerIds>,
    root_q: Query<Entity, With<MulliganRoot>>,
) {
    if snapshot_changed.read().count() == 0 {
        return;
    }

    // Despawn old screen
    for entity in root_q.iter() {
        commands.entity(entity).despawn();
    }

    // Respawn with fresh data
    let hand = &snapshot.snapshot.private_player_state.hand;
    let mulligan_info = &snapshot.snapshot.mulligan_info;

    let mulligan_count = mulligan_info.as_ref().map(|m| m.mulligan_count).unwrap_or(0);
    let in_put_back = mulligan_info
        .as_ref()
        .map(|m| m.has_kept && m.cards_to_put_back > 0)
        .unwrap_or(false);
    let cards_to_put_back = mulligan_info
        .as_ref()
        .map(|m| m.cards_to_put_back)
        .unwrap_or(0);

    let status_text = status_label_text(mulligan_count, in_put_back, cards_to_put_back);

    commands.spawn((
        MulliganRoot,
        DespawnOnExit(AppState::Mulligan),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(BG_COLOR),
    ))
    .with_children(|root| {
        // Hand row
        root.spawn((
            MulliganHandRoot,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|hand_row| {
            for card in hand.iter() {
                let border_color = card_border_color(&card.types);
                let bg_color = card_background_color(&card.types);
                let type_line = card_type_line(&card.types);
                let pt_text = card_pt_text(card.power, card.toughness);
                let name = card.name.clone();
                let instance_id = card.instance_id.clone();

                let mut entity_cmd = hand_row.spawn((
                    CardNode,
                    Node {
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        border: UiRect::all(Val::Px(CARD_BORDER)),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BorderColor::all(if in_put_back { SELECTABLE_BORDER } else { border_color }),
                    BackgroundColor(bg_color),
                ));

                if in_put_back {
                    entity_cmd.insert((
                        Button,
                        PutBackCard { instance_id: instance_id.clone() },
                    ));
                }

                entity_cmd.with_children(|card_node| {
                    card_node.spawn((
                        Text::new(name),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                    card_node.spawn((
                        Text::new(type_line),
                        TextFont { font_size: 9.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    if let Some(pt) = pt_text {
                        card_node.spawn((
                            Text::new(pt),
                            TextFont { font_size: 10.0, ..default() },
                            TextColor(Color::srgb(0.9, 0.9, 0.5)),
                        ));
                    }
                });
            }
        });

        // Status label
        root.spawn((
            MulliganStatusLabel,
            Text::new(status_text),
            TextFont { font_size: 18.0, ..default() },
            TextColor(if in_put_back { INSTRUCTION_COLOR } else { LABEL_COLOR }),
        ));

        // Buttons row (hidden in put-back mode)
        if !in_put_back {
            let mulligan_label = if mulligan_count == 0 {
                "Mulligan".to_owned()
            } else {
                format!("Mulligan (to {})", 7u32.saturating_sub(mulligan_count + 1))
            };

            root.spawn((
                MulliganButtonRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                },
            ))
            .with_children(|btn_row| {
                btn_row
                    .spawn((
                        KeepButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Keep"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                btn_row
                    .spawn((
                        MulliganButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(mulligan_label),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        }
    });
}

// ============================================================================
// Button hover / press visual feedback
// ============================================================================

fn update_button_colors(
    mut keep_q: Query<(&Interaction, &mut BackgroundColor), With<KeepButton>>,
    mut mulligan_q: Query<
        (&Interaction, &mut BackgroundColor),
        (With<MulliganButton>, Without<KeepButton>),
    >,
) {
    for (interaction, mut bg) in keep_q.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BUTTON_PRESSED_BG),
            Interaction::Hovered => BackgroundColor(BUTTON_HOVER_BG),
            Interaction::None => BackgroundColor(BUTTON_BG),
        };
    }
    for (interaction, mut bg) in mulligan_q.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(BUTTON_PRESSED_BG),
            Interaction::Hovered => BackgroundColor(BUTTON_HOVER_BG),
            Interaction::None => BackgroundColor(BUTTON_BG),
        };
    }
}

// ============================================================================
// Keep button handler
// ============================================================================

fn handle_keep_button(
    keep_q: Query<&Interaction, (Changed<Interaction>, With<KeepButton>)>,
    mut action_messages: MessageWriter<GameActionMessage>,
    player_ids: Res<PlayerIds>,
) {
    for interaction in keep_q.iter() {
        if *interaction == Interaction::Pressed {
            action_messages.write(GameActionMessage(Action::MulliganKeep {
                player_id: PlayerId::new(&player_ids.p1.id),
            }));
        }
    }
}

// ============================================================================
// Mulligan (redraw) button handler
// ============================================================================

fn handle_mulligan_button(
    mulligan_q: Query<&Interaction, (Changed<Interaction>, With<MulliganButton>)>,
    mut action_messages: MessageWriter<GameActionMessage>,
    player_ids: Res<PlayerIds>,
) {
    for interaction in mulligan_q.iter() {
        if *interaction == Interaction::Pressed {
            action_messages.write(GameActionMessage(Action::MulliganRedraw {
                player_id: PlayerId::new(&player_ids.p1.id),
            }));
        }
    }
}

// ============================================================================
// Put-back card click handler
// ============================================================================

fn handle_put_back_card(
    put_back_q: Query<(&Interaction, &PutBackCard), Changed<Interaction>>,
    mut action_messages: MessageWriter<GameActionMessage>,
    player_ids: Res<PlayerIds>,
) {
    for (interaction, put_back) in put_back_q.iter() {
        if *interaction == Interaction::Pressed {
            action_messages.write(GameActionMessage(Action::PutCardOnBottom {
                player_id: PlayerId::new(&player_ids.p1.id),
                card_id: CardInstanceId::new(&put_back.instance_id),
            }));
        }
    }
}

// ============================================================================
// Mulligan action handler — applies actions and triggers snapshot rebuild
// ============================================================================

/// Update system: drain mulligan-related `GameActionMessage`s during the
/// Mulligan phase, apply to the domain, recompute snapshot, and send
/// `SnapshotChangedMessage`.
///
/// Also checks for mulligan completion: if the game is no longer in the
/// mulligan phase after the action, transitions to `AppState::InGame`.
fn handle_mulligan_actions(
    mut game_state: ResMut<crate::plugins::game::GameState>,
    mut action_messages: MessageReader<GameActionMessage>,
    mut snapshot_res: ResMut<crate::plugins::game::CurrentSnapshot>,
    mut playable_res: ResMut<crate::plugins::game::PlayableCards>,
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
    mut next_state: ResMut<NextState<AppState>>,
    player_ids: Res<PlayerIds>,
) {
    use crate::plugins::game::snapshot::compute_snapshot;

    let mut any_applied = false;

    for message in action_messages.read() {
        let is_mulligan_action = matches!(
            &message.0,
            Action::MulliganKeep { .. }
                | Action::MulliganRedraw { .. }
                | Action::PutCardOnBottom { .. }
        );

        if !is_mulligan_action {
            continue;
        }

        match game_state.game.apply(message.0.clone()) {
            Ok(_) => {
                any_applied = true;
            }
            Err(err) => {
                warn!(%err, "Mulligan action rejected");
            }
        }
    }

    if any_applied {
        let view_player_id = &player_ids.p1.id;
        match compute_snapshot(&game_state.game, view_player_id) {
            Ok((snapshot, playable_cards)) => {
                *snapshot_res = crate::plugins::game::CurrentSnapshot { snapshot };
                *playable_res = crate::plugins::game::PlayableCards {
                    result: playable_cards,
                };
                snapshot_changed.write(SnapshotChangedMessage);
            }
            Err(err) => {
                error!(%err, "Failed to compute snapshot after mulligan action");
            }
        }

        // Check if mulligan is complete — if so, transition to InGame.
        if !game_state.game.is_in_mulligan() {
            debug!("Mulligan complete — transitioning to AppState::InGame");
            next_state.set(AppState::InGame);
        }
    }
}

// ============================================================================
// Pure helper — status label text
// ============================================================================

/// Compute the status/instruction label text for the mulligan screen.
pub(crate) fn status_label_text(
    mulligan_count: u32,
    in_put_back: bool,
    cards_to_put_back: u32,
) -> String {
    if in_put_back {
        if cards_to_put_back == 1 {
            "Put 1 card on the bottom. Click a card.".to_owned()
        } else {
            format!("Put {cards_to_put_back} cards on the bottom. Click a card.")
        }
    } else if mulligan_count == 0 {
        "Opening hand — keep or mulligan?".to_owned()
    } else if mulligan_count == 1 {
        "You will put 1 card on the bottom if you keep.".to_owned()
    } else {
        format!("You will put {mulligan_count} cards on the bottom if you keep.")
    }
}

// ============================================================================
// Plugin
// ============================================================================

pub(crate) struct MulliganPlugin;

impl Plugin for MulliganPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup_mulligan_screen.run_if(in_state(AppState::Mulligan)))
            .add_systems(
                Update,
                (
                    rebuild_mulligan_screen,
                    handle_keep_button,
                    handle_mulligan_button,
                    handle_put_back_card,
                    update_button_colors,
                    handle_mulligan_actions,
                )
                    .run_if(in_state(AppState::Mulligan)),
            );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_label_initial() {
        let text = status_label_text(0, false, 0);
        assert_eq!(text, "Opening hand — keep or mulligan?");
    }

    #[test]
    fn status_label_after_one_mulligan() {
        let text = status_label_text(1, false, 0);
        assert_eq!(text, "You will put 1 card on the bottom if you keep.");
    }

    #[test]
    fn status_label_after_two_mulligans() {
        let text = status_label_text(2, false, 0);
        assert_eq!(text, "You will put 2 cards on the bottom if you keep.");
    }

    #[test]
    fn status_label_put_back_singular() {
        let text = status_label_text(0, true, 1);
        assert_eq!(text, "Put 1 card on the bottom. Click a card.");
    }

    #[test]
    fn status_label_put_back_plural() {
        let text = status_label_text(0, true, 3);
        assert_eq!(text, "Put 3 cards on the bottom. Click a card.");
    }
}
