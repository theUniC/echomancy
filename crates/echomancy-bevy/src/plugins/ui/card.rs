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
//! Tapped cards receive a small 10° tilt and 0.6 opacity so text stays readable.
//! Opponent cards receive a darkened background color instead of a 180° rotation.

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

/// Opacity applied to tapped cards (reduced to visually indicate tap state).
const TAPPED_ALPHA: f32 = 0.6;

/// Small rotation angle applied to tapped cards (10 degrees in radians).
///
/// A subtle tilt keeps card text readable while still indicating tap state,
/// unlike a full 90° rotation which makes text sideways and illegible.
pub(crate) const TAPPED_ROTATION_RADIANS: f32 = std::f32::consts::PI / 18.0; // 10°

/// Background color multiplier for opponent cards (darker, muted palette).
///
/// Returns a darkened version of the type-based background so opponent cards
/// are visually distinct without rotating the text 180°.
pub(crate) fn card_opponent_background_color(types: &[CardType]) -> Color {
    // Take the normal background and darken it by ~40%.
    let base = card_background_color(types);
    let Color::Srgba(srgba) = base else {
        return base;
    };
    Color::srgba(
        srgba.red * 0.55,
        srgba.green * 0.55,
        srgba.blue * 0.55,
        srgba.alpha,
    )
}

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
    /// Render this card as belonging to the opponent (muted background, no rotation).
    ///
    /// Previously this applied a 180° rotation which made text upside-down and
    /// unreadable. Now we apply a darker background color instead so the card
    /// position (top of screen) already contextualises whose card it is.
    pub(crate) is_opponent: bool,
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
    // Opponent cards use a darker, muted background so they stand out visually
    // without rotating text into an unreadable orientation.
    let bg_color = if data.is_opponent {
        card_opponent_background_color(data.types)
    } else {
        card_background_color(data.types)
    };
    let alpha = if data.is_tapped { TAPPED_ALPHA } else { 1.0 };

    // Tapped cards get a small 10° tilt so the text remains readable.
    // We no longer rotate 90° (sideways text) or 180° (upside-down text).
    // Opponent cards get no rotation — their position at the top of the
    // screen already communicates ownership.
    let rotation_angle: f32 = if data.is_tapped {
        TAPPED_ROTATION_RADIANS
    } else {
        0.0
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
        UiTransform::from_rotation(Rot2::radians(rotation_angle)),
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

    // ---- TAPPED_ROTATION_RADIANS -------------------------------------------

    #[test]
    fn tapped_rotation_is_small_angle_not_ninety_degrees() {
        // Must be less than 30° (π/6) so text remains readable.
        let thirty_degrees = std::f32::consts::PI / 6.0;
        assert!(
            TAPPED_ROTATION_RADIANS < thirty_degrees,
            "Tapped rotation {TAPPED_ROTATION_RADIANS:.4} rad should be a small tilt, not 90°"
        );
        assert!(
            TAPPED_ROTATION_RADIANS > 0.0,
            "Tapped rotation should be positive (non-zero tilt)"
        );
    }

    // ---- card_opponent_background_color ------------------------------------

    #[test]
    fn opponent_creature_background_is_darker_than_normal() {
        let normal = card_background_color(&[CardType::Creature]);
        let opponent = card_opponent_background_color(&[CardType::Creature]);
        let Color::Srgba(n) = normal else { panic!("Expected Srgba") };
        let Color::Srgba(o) = opponent else { panic!("Expected Srgba") };
        // All channels should be darker (lower value).
        assert!(o.red < n.red, "Opponent red channel should be darker");
        assert!(o.green < n.green, "Opponent green channel should be darker");
        assert!(o.blue < n.blue, "Opponent blue channel should be darker");
    }

    #[test]
    fn opponent_land_background_is_darker_than_normal() {
        let normal = card_background_color(&[CardType::Land]);
        let opponent = card_opponent_background_color(&[CardType::Land]);
        let Color::Srgba(n) = normal else { panic!("Expected Srgba") };
        let Color::Srgba(o) = opponent else { panic!("Expected Srgba") };
        assert!(o.red < n.red, "Opponent land red should be darker");
        assert!(o.green < n.green, "Opponent land green should be darker");
    }

    #[test]
    fn opponent_background_preserves_alpha() {
        let opponent = card_opponent_background_color(&[CardType::Creature]);
        let Color::Srgba(o) = opponent else { panic!("Expected Srgba") };
        assert!(
            (o.alpha - 1.0).abs() < 0.01,
            "Opponent background alpha should remain 1.0"
        );
    }
}
