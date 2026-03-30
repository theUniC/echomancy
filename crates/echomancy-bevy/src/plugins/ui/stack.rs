//! StackPlugin — center overlay that displays the game stack.
//!
//! Layout (center of screen, absolute, only shown when stack is non-empty):
//! ```text
//! ┌─────────────────────────────┐
//! │  Stack (2 items)            │
//! │  ┌───────────────────────┐  │
//! │  │ [Spell] Goblin        │  │  ← top of stack (resolves first)
//! │  │    Cast by Player 2   │  │
//! │  ├───────────────────────┤  │
//! │  │ [Spell] Lightning     │  │  ← bottom of stack
//! │  │    Cast by Player 2   │  │
//! │  │    > Player 1         │  │  ← target
//! │  └───────────────────────┘  │
//! └─────────────────────────────┘
//! ```
//!
//! The panel is despawned entirely when the stack is empty and respawned when
//! it becomes non-empty. This keeps the ECS clean without hidden/display tricks.

use bevy::prelude::*;
use echomancy_core::prelude::StackItemKind;

use crate::plugins::game::{AppState, CurrentSnapshot, PlayerIds, SnapshotChangedMessage};

// ============================================================================
// Colors
// ============================================================================

/// Background color of the stack panel container (semi-transparent dark).
const STACK_PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.92);

/// Background color of a single stack item row.
const ITEM_BG: Color = Color::srgb(0.14, 0.14, 0.20);

/// Text color for the header ("Stack (N items)").
const HEADER_COLOR: Color = Color::srgb(0.90, 0.85, 0.60);

/// Text color for card name / kind prefix.
const CARD_NAME_COLOR: Color = Color::srgb(0.92, 0.92, 0.96);

/// Text color for controller and target lines (secondary).
const SECONDARY_COLOR: Color = Color::srgb(0.60, 0.60, 0.68);

// ============================================================================
// Marker components
// ============================================================================

/// Marks the stack overlay panel root node.
#[derive(Component)]
pub(crate) struct StackPanelRoot;

// ============================================================================
// Pure helper functions (testable without ECS)
// ============================================================================

/// Return the short kind prefix shown before the card name.
///
/// Examples:
/// - `StackItemKind::Spell` → `"[Spell]"`
/// - `StackItemKind::ActivatedAbility` → `"[Ability]"`
/// - `StackItemKind::TriggeredAbility` → `"[Trigger]"`
pub(crate) fn stack_item_kind_prefix(kind: &StackItemKind) -> &'static str {
    match kind {
        StackItemKind::Spell => "[Spell]",
        StackItemKind::ActivatedAbility => "[Ability]",
        StackItemKind::TriggeredAbility => "[Trigger]",
    }
}

/// Format the main line shown for a stack item.
///
/// Example: `"[Spell] Lightning Strike"`
pub(crate) fn format_stack_item_name(kind: &StackItemKind, card_name: &str) -> String {
    format!("{} {}", stack_item_kind_prefix(kind), card_name)
}

/// Format the "Cast by <name>" / "Controlled by <name>" controller line.
pub(crate) fn format_controller_line(kind: &StackItemKind, controller_name: &str) -> String {
    let verb = match kind {
        StackItemKind::Spell => "Cast by",
        StackItemKind::ActivatedAbility | StackItemKind::TriggeredAbility => "Controlled by",
    };
    format!("{verb} {controller_name}")
}

/// Format the stack panel header label.
///
/// Example: `"Stack (2 items)"` or `"Stack (1 item)"`
pub(crate) fn format_stack_header(item_count: usize) -> String {
    if item_count == 1 {
        "Stack (1 item)".to_owned()
    } else {
        format!("Stack ({item_count} items)")
    }
}

// ============================================================================
// Systems
// ============================================================================

/// Update system: rebuild the stack panel when the snapshot changes.
///
/// The entire panel is despawned and respawned on every snapshot change so that
/// the displayed stack always matches the current game state exactly.
pub(crate) fn rebuild_stack_panel(
    snapshot: Res<CurrentSnapshot>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    panel_q: Query<Entity, With<StackPanelRoot>>,
    mut commands: Commands,
    player_ids: Res<PlayerIds>,
) {
    if snapshot_changed.read().count() == 0 {
        return;
    }

    // Always despawn the existing panel first.
    for entity in &panel_q {
        commands.entity(entity).despawn();
    }

    let stack = &snapshot.snapshot.visible_stack;

    // Only render when there are items on the stack.
    if stack.items.is_empty() {
        return;
    }

    spawn_stack_panel(&mut commands, stack, &player_ids);
}

/// Spawn a fresh stack panel from the given stack snapshot.
fn spawn_stack_panel(
    commands: &mut Commands,
    stack: &echomancy_core::prelude::StackSnapshot,
    player_ids: &PlayerIds,
) {
    commands
        .spawn((
            StackPanelRoot,
            Node {
                position_type: PositionType::Absolute,
                // Center horizontally and vertically.
                left: Val::Percent(30.0),
                right: Val::Percent(30.0),
                top: Val::Percent(30.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(STACK_PANEL_BG),
            // Ensure the overlay sits on top of other UI nodes.
            ZIndex(10),
        ))
        .with_children(|panel| {
            // Header: "Stack (N items)"
            panel.spawn((
                Text::new(format_stack_header(stack.items.len())),
                TextFont { font_size: 14.0, ..default() },
                TextColor(HEADER_COLOR),
            ));

            // One row per stack item (index 0 = top of stack).
            for item in &stack.items {
                let controller_name = player_ids.name_for(&item.controller_id);
                let name_line = format_stack_item_name(&item.kind, &item.source_card_name);
                let controller_line = format_controller_line(&item.kind, controller_name);

                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(8.0)),
                            row_gap: Val::Px(2.0),
                            ..default()
                        },
                        BackgroundColor(ITEM_BG),
                    ))
                    .with_children(|row| {
                        // Card name with kind prefix.
                        row.spawn((
                            Text::new(name_line),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(CARD_NAME_COLOR),
                        ));

                        // Controller line.
                        row.spawn((
                            Text::new(controller_line),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(SECONDARY_COLOR),
                        ));

                        // Target lines (one per target description).
                        for target_desc in &item.target_descriptions {
                            row.spawn((
                                Text::new(format!("> {target_desc}")),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(SECONDARY_COLOR),
                            ));
                        }
                    });
            }
        });
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers stack display systems.
pub(crate) struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rebuild_stack_panel.run_if(in_state(AppState::InGame)),
        );
    }
}

// ============================================================================
// Tests (TDD: written before implementation)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- stack_item_kind_prefix --------------------------------------------

    #[test]
    fn spell_kind_prefix() {
        assert_eq!(stack_item_kind_prefix(&StackItemKind::Spell), "[Spell]");
    }

    #[test]
    fn activated_ability_kind_prefix() {
        assert_eq!(
            stack_item_kind_prefix(&StackItemKind::ActivatedAbility),
            "[Ability]"
        );
    }

    #[test]
    fn triggered_ability_kind_prefix() {
        assert_eq!(
            stack_item_kind_prefix(&StackItemKind::TriggeredAbility),
            "[Trigger]"
        );
    }

    // ---- format_stack_item_name -------------------------------------------

    #[test]
    fn format_stack_item_name_spell() {
        let result = format_stack_item_name(&StackItemKind::Spell, "Lightning Strike");
        assert_eq!(result, "[Spell] Lightning Strike");
    }

    #[test]
    fn format_stack_item_name_ability() {
        let result = format_stack_item_name(&StackItemKind::ActivatedAbility, "Forest");
        assert_eq!(result, "[Ability] Forest");
    }

    #[test]
    fn format_stack_item_name_trigger() {
        let result = format_stack_item_name(&StackItemKind::TriggeredAbility, "Bear");
        assert_eq!(result, "[Trigger] Bear");
    }

    // ---- format_controller_line -------------------------------------------

    #[test]
    fn format_controller_line_spell() {
        let result = format_controller_line(&StackItemKind::Spell, "Player 1");
        assert_eq!(result, "Cast by Player 1");
    }

    #[test]
    fn format_controller_line_activated_ability() {
        let result = format_controller_line(&StackItemKind::ActivatedAbility, "Player 2");
        assert_eq!(result, "Controlled by Player 2");
    }

    #[test]
    fn format_controller_line_triggered_ability() {
        let result = format_controller_line(&StackItemKind::TriggeredAbility, "Player 1");
        assert_eq!(result, "Controlled by Player 1");
    }

    // ---- format_stack_header ----------------------------------------------

    #[test]
    fn stack_header_singular() {
        assert_eq!(format_stack_header(1), "Stack (1 item)");
    }

    #[test]
    fn stack_header_plural() {
        assert_eq!(format_stack_header(2), "Stack (2 items)");
    }

    #[test]
    fn stack_header_zero() {
        // Edge case: called with 0 items (won't normally be shown, but must not panic).
        assert_eq!(format_stack_header(0), "Stack (0 items)");
    }

    #[test]
    fn stack_header_large() {
        assert_eq!(format_stack_header(10), "Stack (10 items)");
    }
}
