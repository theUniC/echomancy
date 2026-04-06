;;; Turn to Frog — Target creature loses all abilities and becomes a base 1/1 until end of turn.
;;;
;;; This is a MULTI-LAYER effect spanning Layer 6 (RemoveAllAbilities) and Layer 7b (SetPowerToughness).
;;; Both effects share the same source and are applied with the same timestamp per CR 613.6.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "turn-to-frog")      — the card definition ID
;;;
;;; If target-id is empty (no target), neither rule fires.

(defrule turn-to-frog-remove-abilities
  "Turn to Frog removes all abilities from the target (Layer 6)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "turn-to-frog"))
  =>
  (assert (action-remove-all-abilities
    (source ?spell-id)
    (target ?target)
    (duration until-end-of-turn))))

(defrule turn-to-frog-set-pt
  "Turn to Frog sets the target's P/T to 1/1 (Layer 7b)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "turn-to-frog"))
  =>
  (assert (action-set-pt
    (source ?spell-id)
    (target ?target)
    (power 1)
    (toughness 1)
    (duration until-end-of-turn))))
