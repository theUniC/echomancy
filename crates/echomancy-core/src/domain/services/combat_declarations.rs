//! CombatDeclarations — validate attacker/blocker declarations.
//!
//! Stateless service that validates declarations and returns the state changes
//! to be applied by the caller (Game aggregate in Phase 6).
//!
//! Mirrors `CombatDeclarations.ts`.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{StaticAbility, Step};
use crate::domain::errors::GameError;
use crate::domain::types::CardInstanceId;
use crate::domain::value_objects::permanent_state::PermanentState;

// ============================================================================
// Context trait
// ============================================================================

/// Read-only context interface for combat validation.
///
/// The Game aggregate will implement this; in tests we use minimal structs.
pub(crate) trait CombatValidationContext {
    /// The step that is currently active.
    fn current_step(&self) -> Step;

    /// The ID of the player whose turn it is.
    fn current_player_id(&self) -> &str;

    /// Returns the opponent of the given player (MVP: 2-player game).
    fn opponent_of(&self, player_id: &str) -> &str;

    /// Returns all cards on the battlefield controlled by `player_id`.
    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance];

    /// Returns `true` if the card is a creature.
    fn is_creature(&self, card: &CardInstance) -> bool;

    /// Returns `true` if the card has the given static ability.
    fn has_static_ability(&self, card: &CardInstance, ability: StaticAbility) -> bool;

    /// Returns the `PermanentState` for the given instance ID, if on a battlefield.
    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState>;
}

// ============================================================================
// Result types
// ============================================================================

/// The state changes that should be applied when an attacker is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclareAttackerResult {
    /// The new `PermanentState` for the declared attacker.
    pub new_state: PermanentState,
}

/// The state changes that should be applied when a blocker is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclareBlockerResult {
    /// The new `PermanentState` for the blocker.
    pub new_blocker_state: PermanentState,
    /// The new `PermanentState` for the attacker (updated `blocked_by`).
    pub new_attacker_state: PermanentState,
}

// ============================================================================
// Service functions
// ============================================================================

/// Validates a declare-attacker action and returns the new state for the
/// attacking creature.
///
/// Checks (in order):
/// 1. Current step is `DeclareAttackers`.
/// 2. `player_id` is the current (active) player.
/// 3. Creature exists on the player's battlefield.
/// 4. Card is a creature.
/// 5. Creature has permanent state and creature sub-state.
/// 6. No summoning sickness (unless Haste).
/// 7. Creature is not tapped.
/// 8. Creature has not already attacked this turn.
///
/// # Errors
///
/// Returns `GameError` if any validation check fails.
pub(crate) fn validate_declare_attacker(
    ctx: &impl CombatValidationContext,
    player_id: &str,
    creature_id: &str,
) -> Result<DeclareAttackerResult, GameError> {
    // 1. Must be the DECLARE_ATTACKERS step.
    if ctx.current_step() != Step::DeclareAttackers {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "DECLARE_ATTACKER".to_owned(),
        });
    }

    // 2. Must be the active player.
    if player_id != ctx.current_player_id() {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "DECLARE_ATTACKER".to_owned(),
        });
    }

    // 3 & 4. Creature must exist on the player's battlefield and be a creature.
    let creature = ctx
        .battlefield_cards(player_id)
        .iter()
        .find(|c| c.instance_id() == creature_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(creature_id),
        })?;

    if !ctx.is_creature(creature) {
        return Err(GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(creature_id),
        });
    }

    // 5. Must have creature sub-state.
    let state = ctx
        .permanent_state(creature_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(creature_id),
        })?;

    let cs = state.creature_state().ok_or_else(|| GameError::PermanentNotFound {
        permanent_id: CardInstanceId::new(creature_id),
    })?;

    // 6. Summoning sickness check (Haste bypasses it).
    if cs.has_summoning_sickness() && !ctx.has_static_ability(creature, StaticAbility::Haste) {
        return Err(GameError::CreatureHasSummoningSickness {
            creature_id: CardInstanceId::new(creature_id),
        });
    }

    // 7. Must not be tapped.
    if state.is_tapped() {
        return Err(GameError::TappedCreatureCannotAttack {
            creature_id: CardInstanceId::new(creature_id),
        });
    }

    // 8. Must not have already attacked this turn.
    if cs.has_attacked_this_turn() {
        return Err(GameError::CreatureAlreadyAttacked {
            creature_id: CardInstanceId::new(creature_id),
        });
    }

    // Build the new state.
    let new_state = state
        .with_attacking(true)
        .map_err(|_| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(creature_id),
        })?
        .with_has_attacked_this_turn(true)
        .map_err(|_| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(creature_id),
        })?;

    // Tap unless Vigilance.
    let new_state = if ctx.has_static_ability(creature, StaticAbility::Vigilance) {
        new_state
    } else {
        new_state.with_tapped(true)
    };

    Ok(DeclareAttackerResult { new_state })
}

/// Validates a declare-blocker action and returns the new states for both
/// the blocker and the attacker.
///
/// Checks (in order):
/// 1. Current step is `DeclareBlockers`.
/// 2. `player_id` is the defending player (opponent of the active player).
/// 3. Blocker exists on the defending player's battlefield and is a creature.
/// 4. Blocker is not tapped.
/// 5. Blocker is not already blocking.
/// 6. Attacker exists and has creature sub-state.
/// 7. Attacker is actually attacking.
/// 8. Attacker is not already blocked (MVP: one blocker per attacker).
/// 9. Flying restriction: if attacker has Flying, blocker needs Flying or Reach.
///
/// # Errors
///
/// Returns `GameError` if any validation check fails.
pub(crate) fn validate_declare_blocker(
    ctx: &impl CombatValidationContext,
    player_id: &str,
    blocker_id: &str,
    attacker_id: &str,
) -> Result<DeclareBlockerResult, GameError> {
    // 1. Must be the DECLARE_BLOCKERS step.
    if ctx.current_step() != Step::DeclareBlockers {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "DECLARE_BLOCKER".to_owned(),
        });
    }

    // 2. Must be the defending player.
    let defending_player = ctx.opponent_of(ctx.current_player_id());
    if player_id != defending_player {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "DECLARE_BLOCKER".to_owned(),
        });
    }

    // 3. Blocker must exist on the defender's battlefield and be a creature.
    let blocker = ctx
        .battlefield_cards(player_id)
        .iter()
        .find(|c| c.instance_id() == blocker_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(blocker_id),
        })?;

    if !ctx.is_creature(blocker) {
        return Err(GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(blocker_id),
        });
    }

    let blocker_state = ctx
        .permanent_state(blocker_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(blocker_id),
        })?;

    let blocker_cs = blocker_state
        .creature_state()
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(blocker_id),
        })?;

    // 4. Blocker must not be tapped.
    if blocker_state.is_tapped() {
        return Err(GameError::TappedCreatureCannotBlock {
            creature_id: CardInstanceId::new(blocker_id),
        });
    }

    // 5. Blocker must not already be blocking.
    if blocker_cs.blocking_creature_id().is_some() {
        return Err(GameError::CreatureAlreadyBlocking {
            creature_id: CardInstanceId::new(blocker_id),
        });
    }

    // 6 & 7. Attacker must exist and be attacking.
    let attacker_state = ctx
        .permanent_state(attacker_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(attacker_id),
        })?;

    let attacker_cs = attacker_state
        .creature_state()
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(attacker_id),
        })?;

    if !attacker_cs.is_attacking() {
        return Err(GameError::CannotBlockNonAttackingCreature {
            attacker_id: CardInstanceId::new(attacker_id),
        });
    }

    // 9. Flying restriction.
    let attacker = ctx
        .battlefield_cards(ctx.current_player_id())
        .iter()
        .find(|c| c.instance_id() == attacker_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(attacker_id),
        })?;

    if ctx.has_static_ability(attacker, StaticAbility::Flying) {
        let blocker_can_block = ctx.has_static_ability(blocker, StaticAbility::Flying)
            || ctx.has_static_ability(blocker, StaticAbility::Reach);

        if !blocker_can_block {
            return Err(GameError::CannotBlockFlyingCreature {
                blocker_id: CardInstanceId::new(blocker_id),
                attacker_id: CardInstanceId::new(attacker_id),
            });
        }
    }

    // Build new states.
    let new_blocker_state = blocker_state
        .with_blocking_creature_id(Some(CardInstanceId::new(attacker_id)))
        .map_err(|_| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(blocker_id),
        })?;

    let new_attacker_state = attacker_state
        .with_blocked_by(Some(CardInstanceId::new(blocker_id)))
        .map_err(|_| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(attacker_id),
        })?;

    Ok(DeclareBlockerResult {
        new_blocker_state,
        new_attacker_state,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, StaticAbility, Step};
    use crate::domain::types::CardInstanceId;
    use crate::domain::value_objects::permanent_state::PermanentState;
    use std::collections::HashMap;

    // ---- minimal test context -----------------------------------------------

    struct TestCtx {
        step: Step,
        current_player: String,
        players: HashMap<String, Vec<CardInstance>>,
        permanent_states: HashMap<String, PermanentState>,
    }

    impl TestCtx {
        fn new(step: Step, current_player: &str) -> Self {
            TestCtx {
                step,
                current_player: current_player.to_owned(),
                players: HashMap::new(),
                permanent_states: HashMap::new(),
            }
        }

        fn add_permanent(mut self, player: &str, card: CardInstance, state: PermanentState) -> Self {
            self.players
                .entry(player.to_owned())
                .or_default()
                .push(card.clone());
            self.permanent_states.insert(card.instance_id().to_owned(), state);
            self
        }
    }

    impl CombatValidationContext for TestCtx {
        fn current_step(&self) -> Step {
            self.step
        }

        fn current_player_id(&self) -> &str {
            &self.current_player
        }

        fn opponent_of(&self, player_id: &str) -> &str {
            // MVP: 2-player — opponent is whichever player isn't `player_id`.
            for key in self.players.keys() {
                if key != player_id {
                    return key.as_str();
                }
            }
            // Fallback for tests that don't add both players.
            if player_id == "p1" { "p2" } else { "p1" }
        }

        fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
            self.players
                .get(player_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        }

        fn is_creature(&self, card: &CardInstance) -> bool {
            card.definition().is_creature()
        }

        fn has_static_ability(&self, card: &CardInstance, ability: StaticAbility) -> bool {
            card.definition().has_static_ability(ability)
        }

        fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
            self.permanent_states.get(instance_id)
        }
    }

    // ---- helpers ------------------------------------------------------------

    fn make_creature(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(2, 2);
        CardInstance::new(id, def, owner)
    }

    fn make_creature_with(id: &str, owner: &str, ability: StaticAbility) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(ability);
        CardInstance::new(id, def, owner)
    }

    fn ready_creature_state() -> PermanentState {
        PermanentState::for_creature(2, 2)
            .with_summoning_sickness(false)
            .unwrap()
    }

    fn attacking_state() -> PermanentState {
        ready_creature_state()
            .with_attacking(true)
            .unwrap()
            .with_has_attacked_this_turn(true)
            .unwrap()
    }

    // ---- validate_declare_attacker -----------------------------------------

    #[test]
    fn valid_declare_attacker_returns_new_state() {
        let card = make_creature("a1", "p1");
        let state = ready_creature_state();
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let result = validate_declare_attacker(&ctx, "p1", "a1").unwrap();
        let cs = result.new_state.creature_state().unwrap();
        assert!(cs.is_attacking());
        assert!(cs.has_attacked_this_turn());
        assert!(result.new_state.is_tapped()); // tapped by default (no Vigilance)
    }

    #[test]
    fn vigilance_creature_not_tapped_when_attacking() {
        let card = make_creature_with("a1", "p1", StaticAbility::Vigilance);
        let state = ready_creature_state();
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let result = validate_declare_attacker(&ctx, "p1", "a1").unwrap();
        assert!(!result.new_state.is_tapped());
    }

    #[test]
    fn error_wrong_step() {
        let card = make_creature("a1", "p1");
        let state = ready_creature_state();
        let ctx = TestCtx::new(Step::FirstMain, "p1")
            .add_permanent("p1", card, state);

        let err = validate_declare_attacker(&ctx, "p1", "a1").unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_not_active_player() {
        let card = make_creature("a1", "p1");
        let state = ready_creature_state();
        let ctx = TestCtx::new(Step::DeclareAttackers, "p2")
            .add_permanent("p1", card, state);

        let err = validate_declare_attacker(&ctx, "p1", "a1").unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_creature_not_on_battlefield() {
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1");
        let err = validate_declare_attacker(&ctx, "p1", "missing").unwrap_err();
        assert!(matches!(err, GameError::PermanentNotFound { .. }));
    }

    #[test]
    fn error_summoning_sickness() {
        let card = make_creature("a1", "p1");
        // Default creature state has summoning sickness.
        let state = PermanentState::for_creature(2, 2);
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let err = validate_declare_attacker(&ctx, "p1", "a1").unwrap_err();
        assert!(matches!(err, GameError::CreatureHasSummoningSickness { .. }));
    }

    #[test]
    fn haste_bypasses_summoning_sickness() {
        let card = make_creature_with("a1", "p1", StaticAbility::Haste);
        // Creature has summoning sickness but Haste.
        let state = PermanentState::for_creature(2, 2); // has_summoning_sickness = true by default
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let result = validate_declare_attacker(&ctx, "p1", "a1").unwrap();
        assert!(result.new_state.creature_state().unwrap().is_attacking);
    }

    #[test]
    fn error_tapped_creature_cannot_attack() {
        let card = make_creature("a1", "p1");
        let state = ready_creature_state().with_tapped(true);
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let err = validate_declare_attacker(&ctx, "p1", "a1").unwrap_err();
        assert!(matches!(err, GameError::TappedCreatureCannotAttack { .. }));
    }

    #[test]
    fn error_already_attacked_this_turn() {
        let card = make_creature("a1", "p1");
        let state = ready_creature_state()
            .with_has_attacked_this_turn(true)
            .unwrap();
        let ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", card, state);

        let err = validate_declare_attacker(&ctx, "p1", "a1").unwrap_err();
        assert!(matches!(err, GameError::CreatureAlreadyAttacked { .. }));
    }

    // ---- validate_declare_blocker ------------------------------------------

    #[test]
    fn valid_declare_blocker_returns_new_states() {
        let attacker_card = make_creature("a1", "p1");
        let attacker_state = attacking_state();

        let blocker_card = make_creature("b1", "p2");
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", attacker_card, attacker_state)
            .add_permanent("p2", blocker_card, blocker_state);

        let result = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap();

        // Blocker now has blocking_creature_id = a1.
        assert_eq!(
            result
                .new_blocker_state
                .creature_state()
                .unwrap()
                .blocking_creature_id(),
            Some("a1")
        );
        // Attacker now has blocked_by containing b1.
        let attacker_cs = result.new_attacker_state.creature_state().unwrap();
        assert_eq!(attacker_cs.blocked_by().len(), 1);
        assert_eq!(attacker_cs.blocked_by()[0].as_str(), "b1");
    }

    #[test]
    fn error_blocker_wrong_step() {
        let ctx = TestCtx::new(Step::FirstMain, "p1");
        let err = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_active_player_cannot_block() {
        let ctx = TestCtx::new(Step::DeclareBlockers, "p1");
        // p1 is the active player — they cannot declare blockers.
        let err = validate_declare_blocker(&ctx, "p1", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_tapped_creature_cannot_block() {
        let attacker_card = make_creature("a1", "p1");
        let attacker_state = attacking_state();

        let blocker_card = make_creature("b1", "p2");
        let blocker_state = ready_creature_state().with_tapped(true);

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", attacker_card, attacker_state)
            .add_permanent("p2", blocker_card, blocker_state);

        let err = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::TappedCreatureCannotBlock { .. }));
    }

    #[test]
    fn error_creature_already_blocking() {
        let attacker_card = make_creature("a1", "p1");
        let attacker_state = attacking_state();

        let blocker_card = make_creature("b1", "p2");
        let blocker_state = ready_creature_state()
            .with_blocking_creature_id(Some(CardInstanceId::new("other")))
            .unwrap();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", attacker_card, attacker_state)
            .add_permanent("p2", blocker_card, blocker_state);

        let err = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::CreatureAlreadyBlocking { .. }));
    }

    #[test]
    fn error_cannot_block_non_attacking_creature() {
        let non_attacker_card = make_creature("a1", "p1");
        let non_attacker_state = ready_creature_state(); // not attacking

        let blocker_card = make_creature("b1", "p2");
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", non_attacker_card, non_attacker_state)
            .add_permanent("p2", blocker_card, blocker_state);

        let err = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::CannotBlockNonAttackingCreature { .. }));
    }

    #[test]
    fn multiple_blockers_for_same_attacker_are_allowed() {
        let attacker_card = make_creature("a1", "p1");
        let attacker_state = attacking_state()
            .with_blocked_by(Some(CardInstanceId::new("other")))
            .unwrap();

        let blocker_card = make_creature("b1", "p2");
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", attacker_card, attacker_state)
            .add_permanent("p2", blocker_card, blocker_state);

        // A second blocker should now be allowed — no error expected.
        let result = validate_declare_blocker(&ctx, "p2", "b1", "a1");
        assert!(result.is_ok());
        let declare_result = result.unwrap();
        let attacker_cs = declare_result.new_attacker_state.creature_state().unwrap();
        assert_eq!(attacker_cs.blocked_by().len(), 2);
    }

    #[test]
    fn error_cannot_block_flying_with_ground_creature() {
        let flyer_card = make_creature_with("a1", "p1", StaticAbility::Flying);
        let attacker_state = attacking_state();

        let ground_blocker = make_creature("b1", "p2"); // no Flying or Reach
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", flyer_card, attacker_state)
            .add_permanent("p2", ground_blocker, blocker_state);

        let err = validate_declare_blocker(&ctx, "p2", "b1", "a1").unwrap_err();
        assert!(matches!(err, GameError::CannotBlockFlyingCreature { .. }));
    }

    #[test]
    fn creature_with_reach_can_block_flyer() {
        let flyer_card = make_creature_with("a1", "p1", StaticAbility::Flying);
        let attacker_state = attacking_state();

        let reach_blocker = make_creature_with("b1", "p2", StaticAbility::Reach);
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", flyer_card, attacker_state)
            .add_permanent("p2", reach_blocker, blocker_state);

        let result = validate_declare_blocker(&ctx, "p2", "b1", "a1");
        assert!(result.is_ok());
    }

    #[test]
    fn creature_with_flying_can_block_flyer() {
        let flyer_card = make_creature_with("a1", "p1", StaticAbility::Flying);
        let attacker_state = attacking_state();

        let flying_blocker = make_creature_with("b1", "p2", StaticAbility::Flying);
        let blocker_state = ready_creature_state();

        let ctx = TestCtx::new(Step::DeclareBlockers, "p1")
            .add_permanent("p1", flyer_card, attacker_state)
            .add_permanent("p2", flying_blocker, blocker_state);

        let result = validate_declare_blocker(&ctx, "p2", "b1", "a1");
        assert!(result.is_ok());
    }
}
