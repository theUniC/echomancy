//! Card rendering helpers for Bevy UI.
//!
//! Each card is a `Node`-based UI subtree:
//! ```text
//! Card Root (Node: 140×196px, colored border)
//!   ├── Name (Text: white bold 12px)
//!   ├── Art placeholder (Node: dark, flexible height)
//!   ├── Type line (Text: gray italic 10px)
//!   └── P/T (Text: white bold 14px — creatures only)
//! ```
//!
//! Tapped cards receive a 90-degree `Transform` rotation and 0.85 opacity.

use bevy::prelude::*;
use echomancy_core::prelude::CardType;

/// Marker component on every spawned card node.
#[derive(Component)]
pub(crate) struct CardNode;

/// Card dimensions (scaled down from the TS 180×252 to fit 1280×720).
pub(crate) const CARD_WIDTH: f32 = 140.0;
pub(crate) const CARD_HEIGHT: f32 = 196.0;

/// Gap between cards in the battlefield row.
pub(crate) const CARD_GAP: f32 = 20.0;

/// Border width for the card outline.
pub(crate) const CARD_BORDER: f32 = 3.0;

/// Opacity applied to tapped cards.
const TAPPED_ALPHA: f32 = 0.85;

// ============================================================================
// Pure helper functions (testable)
// ============================================================================

/// Return the border color for a card based on its types.
///
/// Priority: Creature > Land > other (gray).
pub(crate) fn card_border_color(types: &[CardType]) -> Color {
    if types.contains(&CardType::Creature) {
        Color::srgb(0.40, 0.72, 0.45) // Soft green
    } else if types.contains(&CardType::Land) {
        Color::srgb(0.72, 0.58, 0.30) // Warm amber
    } else {
        Color::srgb(0.55, 0.55, 0.62) // Cool silver
    }
}

/// Return the background color for the card body (darker than border).
pub(crate) fn card_background_color(types: &[CardType]) -> Color {
    if types.contains(&CardType::Creature) {
        Color::srgb(0.16, 0.24, 0.18) // Dark sage
    } else if types.contains(&CardType::Land) {
        Color::srgb(0.26, 0.22, 0.14) // Dark parchment
    } else {
        Color::srgb(0.22, 0.22, 0.26) // Dark slate
    }
}

/// Build the type-line string for a card (e.g. `"Creature"`, `"Land"`, `"Creature — Land"`).
pub(crate) fn card_type_line(types: &[CardType]) -> String {
    let parts: Vec<&str> = types
        .iter()
        .map(|t| match t {
            CardType::Creature => "Creature",
            CardType::Land => "Land",
            CardType::Instant => "Instant",
            CardType::Sorcery => "Sorcery",
            CardType::Enchantment => "Enchantment",
            CardType::Artifact => "Artifact",
            CardType::Planeswalker => "Planeswalker",
        })
        .collect();
    parts.join(" \u{2014} ")
}

/// Build the power/toughness string for a creature card.
///
/// Returns `None` when either value is absent (non-creatures).
pub(crate) fn card_pt_text(power: Option<i32>, toughness: Option<i32>) -> Option<String> {
    match (power, toughness) {
        (Some(p), Some(t)) => Some(format!("{p}/{t}")),
        _ => None,
    }
}

// ============================================================================
// Spawn helper
// ============================================================================

/// All data needed to spawn a single card UI node.
pub(crate) struct CardSpawnData<'a> {
    pub(crate) name: &'a str,
    pub(crate) types: &'a [CardType],
    pub(crate) power: Option<i32>,
    pub(crate) toughness: Option<i32>,
    pub(crate) is_tapped: bool,
    /// Rotate the entire card 180° (used for opponent battlefield).
    pub(crate) flipped: bool,
}

/// Spawn a card as a Bevy UI node subtree and return its root `Entity`.
pub(crate) fn spawn_card(commands: &mut Commands, data: &CardSpawnData<'_>) -> Entity {
    spawn_card_inner(commands, data, None).id()
}

/// Spawn a card as a Bevy UI node subtree and return the `EntityCommands` for the root.
///
/// `override_border` overrides the default type-based border color (e.g. gold for
/// tappable lands).
pub(crate) fn spawn_card_with_tappable<'a>(
    commands: &'a mut Commands,
    data: &CardSpawnData<'_>,
    override_border: Option<Color>,
) -> EntityCommands<'a> {
    spawn_card_inner(commands, data, override_border)
}

/// Internal implementation: spawn the card node subtree and return `EntityCommands`.
fn spawn_card_inner<'a>(
    commands: &'a mut Commands,
    data: &CardSpawnData<'_>,
    override_border: Option<Color>,
) -> EntityCommands<'a> {
    let border_color = override_border.unwrap_or_else(|| card_border_color(data.types));
    let bg_color = card_background_color(data.types);
    let alpha = if data.is_tapped { TAPPED_ALPHA } else { 1.0 };

    // Rotation: tapped = 90°, flipped = 180°, both = 270°.
    let rotation_z: f32 = match (data.is_tapped, data.flipped) {
        (false, false) => 0.0,
        (true, false) => std::f32::consts::FRAC_PI_2,
        (false, true) => std::f32::consts::PI,
        (true, true) => std::f32::consts::FRAC_PI_2 + std::f32::consts::PI,
    };

    // Capture derived strings before the closure borrows `data`.
    let type_line = card_type_line(data.types);
    let pt_text = card_pt_text(data.power, data.toughness);
    let name = data.name.to_owned();

    let mut ec = commands.spawn((
        CardNode,
        Node {
            width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),
            border: UiRect::all(Val::Px(CARD_BORDER)),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(4.0)),
            flex_shrink: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(border_color),
        BackgroundColor(bg_color.with_alpha(alpha)),
        Transform::from_rotation(Quat::from_rotation_z(rotation_z)),
    ));

    ec.with_children(|parent| {
        // Card name
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        // Art placeholder (fills remaining space)
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
    });

    ec
}

// ============================================================================
// Tests (TDD: written before implementation was final)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- card_border_color -------------------------------------------------

    #[test]
    fn creature_has_green_border() {
        let color = card_border_color(&[CardType::Creature]);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.40).abs() < 0.01, "red channel mismatch");
        assert!((srgba.green - 0.72).abs() < 0.01, "green channel mismatch");
        assert!((srgba.blue - 0.45).abs() < 0.01, "blue channel mismatch");
    }

    #[test]
    fn land_has_brown_border() {
        let color = card_border_color(&[CardType::Land]);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.72).abs() < 0.01, "red channel mismatch");
        assert!((srgba.green - 0.58).abs() < 0.01, "green channel mismatch");
        assert!((srgba.blue - 0.30).abs() < 0.01, "blue channel mismatch");
    }

    #[test]
    fn instant_has_gray_border() {
        let color = card_border_color(&[CardType::Instant]);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.55).abs() < 0.01);
        assert!((srgba.green - 0.55).abs() < 0.01);
        assert!((srgba.blue - 0.62).abs() < 0.01);
    }

    #[test]
    fn creature_takes_priority_over_land_for_border_color() {
        let color = card_border_color(&[CardType::Creature, CardType::Land]);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!(
            (srgba.red - 0.40).abs() < 0.01,
            "Expected creature (green) border; got red={}", srgba.red
        );
    }

    #[test]
    fn empty_types_has_gray_border() {
        let color = card_border_color(&[]);
        let Color::Srgba(srgba) = color else {
            panic!("Expected Srgba color");
        };
        assert!((srgba.red - 0.55).abs() < 0.01);
    }

    // ---- card_type_line ----------------------------------------------------

    #[test]
    fn creature_type_line_is_creature() {
        assert_eq!(card_type_line(&[CardType::Creature]), "Creature");
    }

    #[test]
    fn land_type_line_is_land() {
        assert_eq!(card_type_line(&[CardType::Land]), "Land");
    }

    #[test]
    fn instant_type_line_is_instant() {
        assert_eq!(card_type_line(&[CardType::Instant]), "Instant");
    }

    #[test]
    fn sorcery_type_line_is_sorcery() {
        assert_eq!(card_type_line(&[CardType::Sorcery]), "Sorcery");
    }

    #[test]
    fn multiple_types_joined_with_em_dash() {
        let line = card_type_line(&[CardType::Creature, CardType::Land]);
        assert_eq!(line, "Creature \u{2014} Land");
    }

    #[test]
    fn empty_types_gives_empty_string() {
        assert_eq!(card_type_line(&[]), "");
    }

    // ---- card_pt_text ------------------------------------------------------

    #[test]
    fn pt_text_for_two_two_creature() {
        assert_eq!(card_pt_text(Some(2), Some(2)), Some("2/2".to_owned()));
    }

    #[test]
    fn pt_text_for_three_zero_creature() {
        assert_eq!(card_pt_text(Some(3), Some(0)), Some("3/0".to_owned()));
    }

    #[test]
    fn pt_text_none_when_only_power_present() {
        assert_eq!(card_pt_text(Some(2), None), None);
    }

    #[test]
    fn pt_text_none_when_only_toughness_present() {
        assert_eq!(card_pt_text(None, Some(2)), None);
    }

    #[test]
    fn pt_text_none_for_non_creature() {
        assert_eq!(card_pt_text(None, None), None);
    }

    #[test]
    fn pt_text_for_large_creature() {
        assert_eq!(card_pt_text(Some(10), Some(10)), Some("10/10".to_owned()));
    }
}
