//! CardDetailPlugin — hover/click popup showing full card details.
//!
//! Layout (fixed bottom-left panel, shown when a card is hovered):
//! ```text
//! ┌─────────────────────────┐
//! │  Giant Growth            │  name
//! │  Instant                 │  type line
//! │  {G}                     │  mana cost
//! │  ─────────────           │
//! │  Target creature gets    │  oracle text
//! │  +3/+3 until end of turn │
//! │  ─────────────           │
//! │  1/1                     │  P/T (if creature)
//! └─────────────────────────┘
//! ```
//!
//! Implementation:
//! - `HoveredCard` resource tracks which card (if any) is being hovered.
//! - Cards in hand and on the battlefield carry a `CardHoverable` marker
//!   component with the instance ID. They also have `Interaction` and `Button`.
//! - `detect_hover` reads `Changed<Interaction>` on `CardHoverable` entities
//!   and updates `HoveredCard`.
//! - `rebuild_detail_panel` despawns any existing panel and spawns a new one
//!   from `HoveredCard` whenever `HoveredCard` changes or the snapshot changes.

use bevy::prelude::*;
use echomancy_core::prelude::CardSnapshot;

use super::card::card_type_line;
use crate::plugins::game::{AppState, CurrentSnapshot, SnapshotChangedMessage};

// ============================================================================
// Colors
// ============================================================================

/// Background for the detail panel (semi-transparent dark).
const PANEL_BG: Color = Color::srgba(0.08, 0.10, 0.14, 0.96);

/// Text color for the card name.
const NAME_COLOR: Color = Color::srgb(0.95, 0.95, 1.00);

/// Text color for the type line.
const TYPE_COLOR: Color = Color::srgb(0.75, 0.78, 0.88);

/// Text color for the mana cost.
const MANA_COLOR: Color = Color::srgb(0.95, 0.85, 0.40);

/// Text color for oracle text body.
const ORACLE_COLOR: Color = Color::srgb(0.85, 0.88, 0.92);

/// Text color for P/T.
const PT_COLOR: Color = Color::WHITE;

/// Width of the detail panel in pixels.
pub(crate) const DETAIL_PANEL_WIDTH: f32 = 220.0;

// ============================================================================
// Resources
// ============================================================================

/// Tracks which card is currently hovered (instance ID), if any.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct HoveredCard {
    /// Instance ID of the currently hovered card. `None` when nothing is hovered.
    pub(crate) instance_id: Option<String>,
}

// ============================================================================
// Marker components
// ============================================================================

/// Marks a card entity as hoverable and identifies it by instance ID.
///
/// Add this alongside `Button` + `Interaction::default()` to any card that
/// should trigger the detail panel on hover.
#[derive(Component, Clone, Debug)]
pub(crate) struct CardHoverable {
    pub(crate) instance_id: String,
}

/// Marks the root entity of the card-detail overlay panel.
#[derive(Component)]
pub(crate) struct CardDetailPanelRoot;

// ============================================================================
// Pure helpers (testable without ECS)
// ============================================================================

/// Build the mana cost display line.
///
/// Returns `"—"` when the card has no mana cost (basic lands).
pub(crate) fn format_mana_cost_line(mana_cost_text: Option<&str>) -> &str {
    mana_cost_text.unwrap_or("\u{2014}")
}

/// Build a shortened oracle text for display.
///
/// Returns `""` when there is no oracle text (vanilla creatures, etc.).
pub(crate) fn format_oracle_text(oracle_text: Option<&str>) -> &str {
    oracle_text.unwrap_or("")
}

/// Build the P/T line for a card, or `None` for non-creatures.
pub(crate) fn format_pt_line(power: Option<i32>, toughness: Option<i32>) -> Option<String> {
    match (power, toughness) {
        (Some(p), Some(t)) => Some(format!("{p}/{t}")),
        _ => None,
    }
}

/// Find a `CardSnapshot` by instance ID across all zones in the current snapshot.
///
/// Searches: viewer hand, viewer battlefield, opponent battlefields.
pub(crate) fn find_card_in_snapshot<'a>(
    snapshot: &'a echomancy_core::prelude::GameSnapshot,
    instance_id: &str,
) -> Option<&'a CardSnapshot> {
    // Search viewer's hand.
    if let Some(c) = snapshot
        .private_player_state
        .hand
        .iter()
        .find(|c| c.instance_id == instance_id)
    {
        return Some(c);
    }
    // Search viewer's battlefield.
    if let Some(c) = snapshot
        .private_player_state
        .battlefield
        .iter()
        .find(|c| c.instance_id == instance_id)
    {
        return Some(c);
    }
    // Search opponent battlefields.
    for opp in &snapshot.opponent_states {
        if let Some(c) = opp.battlefield.iter().find(|c| c.instance_id == instance_id) {
            return Some(c);
        }
    }
    None
}

// ============================================================================
// Systems
// ============================================================================

/// Detect hover changes on `CardHoverable` entities and update `HoveredCard`.
///
/// When any card becomes `Hovered`, stores its instance ID.
/// When a card leaves the hovered state (`None` interaction), clears `HoveredCard`
/// only if that card was the one being tracked (prevents flicker when moving
/// between adjacent cards).
pub(crate) fn detect_hover(
    query: Query<(&Interaction, &CardHoverable), Changed<Interaction>>,
    mut hovered_card: ResMut<HoveredCard>,
) {
    for (interaction, hoverable) in &query {
        match interaction {
            Interaction::Hovered => {
                hovered_card.instance_id = Some(hoverable.instance_id.clone());
            }
            Interaction::None => {
                // Only clear if this is the card we are tracking.
                if hovered_card.instance_id.as_deref() == Some(&hoverable.instance_id) {
                    hovered_card.instance_id = None;
                }
            }
            Interaction::Pressed => {
                // Keep showing the panel while pressed.
                hovered_card.instance_id = Some(hoverable.instance_id.clone());
            }
        }
    }
}

/// Rebuild the card-detail panel when the hovered card or snapshot changes.
///
/// Despawns any existing panel entity, then spawns a new one if `HoveredCard`
/// holds an instance ID that can be found in the current snapshot.
pub(crate) fn rebuild_detail_panel(
    mut commands: Commands,
    hovered_card: Res<HoveredCard>,
    current_snapshot: Res<CurrentSnapshot>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    panel_q: Query<Entity, With<CardDetailPanelRoot>>,
) {
    let snapshot_updated = snapshot_changed.read().count() > 0;
    let hovered_changed = hovered_card.is_changed();

    if !snapshot_updated && !hovered_changed {
        return;
    }

    // Always despawn existing panel.
    for entity in &panel_q {
        commands.entity(entity).despawn();
    }

    // Only spawn if something is hovered.
    let Some(ref instance_id) = hovered_card.instance_id else {
        return;
    };

    let Some(card) = find_card_in_snapshot(&current_snapshot.snapshot, instance_id) else {
        return;
    };

    spawn_detail_panel(&mut commands, card);
}

/// Spawn the card-detail overlay panel for the given card snapshot.
fn spawn_detail_panel(commands: &mut Commands, card: &CardSnapshot) {
    let type_line = card_type_line(&card.types);
    let mana_line = format_mana_cost_line(card.mana_cost_text.as_deref()).to_owned();
    let oracle_str = format_oracle_text(card.oracle_text.as_deref()).to_owned();
    let pt_line = format_pt_line(card.power, card.toughness);

    commands
        .spawn((
            CardDetailPanelRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                bottom: Val::Px(8.0),
                width: Val::Px(DETAIL_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            ZIndex(30),
        ))
        .with_children(|panel| {
            // Card name
            panel.spawn((
                Text::new(card.name.clone()),
                TextFont { font_size: 14.0, ..default() },
                TextColor(NAME_COLOR),
            ));

            // Type line
            panel.spawn((
                Text::new(type_line),
                TextFont { font_size: 12.0, ..default() },
                TextColor(TYPE_COLOR),
            ));

            // Mana cost
            panel.spawn((
                Text::new(mana_line),
                TextFont { font_size: 12.0, ..default() },
                TextColor(MANA_COLOR),
            ));

            // Oracle text (only if non-empty)
            if !oracle_str.is_empty() {
                panel.spawn((
                    Text::new(oracle_str),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(ORACLE_COLOR),
                ));
            }

            // P/T (creatures only)
            if let Some(pt) = pt_line {
                panel.spawn((
                    Text::new(pt),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(PT_COLOR),
                ));
            }
        });
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers card-detail hover panel systems.
pub(crate) struct CardDetailPlugin;

impl Plugin for CardDetailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoveredCard>();
        app.add_systems(
            Update,
            (detect_hover, rebuild_detail_panel).run_if(in_state(AppState::InGame)),
        );
    }
}

// ============================================================================
// Tests (TDD: written before implementation was finalised)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- format_mana_cost_line ------------------------------------------------

    #[test]
    fn mana_cost_line_with_value() {
        assert_eq!(format_mana_cost_line(Some("{1}{G}")), "{1}{G}");
    }

    #[test]
    fn mana_cost_line_none_shows_dash() {
        assert_eq!(format_mana_cost_line(None), "\u{2014}");
    }

    // ---- format_oracle_text ---------------------------------------------------

    #[test]
    fn oracle_text_with_value() {
        assert_eq!(
            format_oracle_text(Some("Target creature gets +3/+3 until end of turn.")),
            "Target creature gets +3/+3 until end of turn."
        );
    }

    #[test]
    fn oracle_text_none_is_empty_string() {
        assert_eq!(format_oracle_text(None), "");
    }

    // ---- format_pt_line -------------------------------------------------------

    #[test]
    fn pt_line_for_creature() {
        assert_eq!(format_pt_line(Some(2), Some(2)), Some("2/2".to_owned()));
    }

    #[test]
    fn pt_line_for_large_creature() {
        assert_eq!(format_pt_line(Some(10), Some(10)), Some("10/10".to_owned()));
    }

    #[test]
    fn pt_line_none_for_non_creature() {
        assert_eq!(format_pt_line(None, None), None);
    }

    #[test]
    fn pt_line_none_when_only_power() {
        assert_eq!(format_pt_line(Some(3), None), None);
    }

    // ---- find_card_in_snapshot ------------------------------------------------

    fn make_card_snapshot(instance_id: &str, name: &str) -> CardSnapshot {
        CardSnapshot {
            instance_id: instance_id.to_owned(),
            definition_id: "test".to_owned(),
            name: name.to_owned(),
            types: vec![echomancy_core::prelude::CardType::Creature],
            static_keywords: vec![],
            controller_id: "p1".to_owned(),
            owner_id: "p1".to_owned(),
            tapped: None,
            counters: None,
            damage_marked: None,
            power: Some(2),
            toughness: Some(2),
            combat_state: None,
            mana_cost_text: None,
            oracle_text: None,
        }
    }

    fn make_snapshot_with_cards(
        hand: Vec<CardSnapshot>,
        bf: Vec<CardSnapshot>,
    ) -> echomancy_core::prelude::GameSnapshot {
        use std::collections::HashMap;
        use echomancy_core::prelude::{
            GameSnapshot, PrivatePlayerState, PublicGameState, StackSnapshot,
        };
        use echomancy_core::prelude::{GameLifecycleState, Step};

        GameSnapshot {
            viewer_player_id: "p1".to_owned(),
            public_game_state: PublicGameState {
                turn_number: 1,
                current_player_id: "p1".to_owned(),
                active_player_id: "p1".to_owned(),
                priority_player_id: None,
                current_phase: "Main".to_owned(),
                current_step: Step::FirstMain,
                combat_summary: None,
                stack_size: 0,
                lifecycle_state: GameLifecycleState::Started,
                game_outcome: None,
            },
            private_player_state: PrivatePlayerState {
                player_id: "p1".to_owned(),
                life_total: 20,
                poison_counters: 0,
                mana_pool: HashMap::new(),
                hand,
                battlefield: bf,
                graveyard: vec![],
                exile: vec![],
            },
            opponent_states: vec![],
            visible_stack: StackSnapshot { items: vec![] },
            ui_hints: None,
            mulligan_info: None,
        }
    }

    #[test]
    fn find_card_in_hand() {
        let card = make_card_snapshot("card-1", "Bear");
        let snapshot = make_snapshot_with_cards(vec![card.clone()], vec![]);
        let result = find_card_in_snapshot(&snapshot, "card-1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Bear");
    }

    #[test]
    fn find_card_on_battlefield() {
        let card = make_card_snapshot("card-2", "Forest");
        let snapshot = make_snapshot_with_cards(vec![], vec![card.clone()]);
        let result = find_card_in_snapshot(&snapshot, "card-2");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Forest");
    }

    #[test]
    fn find_card_returns_none_for_unknown_id() {
        let snapshot = make_snapshot_with_cards(vec![], vec![]);
        assert!(find_card_in_snapshot(&snapshot, "nonexistent").is_none());
    }

    #[test]
    fn find_card_prefers_hand_over_battlefield_when_same_id() {
        // In practice instance IDs are unique, but test the search order.
        let hand_card = make_card_snapshot("card-x", "Hand Card");
        let bf_card = make_card_snapshot("card-x", "BF Card");
        let snapshot = make_snapshot_with_cards(vec![hand_card], vec![bf_card]);
        let result = find_card_in_snapshot(&snapshot, "card-x");
        assert_eq!(result.unwrap().name, "Hand Card");
    }
}
