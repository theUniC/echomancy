;;; Fog — Prevent all combat damage that would be dealt this turn.
;;;
;;; No target is required. When Fog resolves, it registers a global
;;; AllCombatDamage prevention effect that intercepts every combat damage
;;; event for the rest of the turn (CR 615.7a).
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (data "fog")               — the card definition ID
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule fog-resolve
  "Fog prevents all combat damage this turn (scope all-combat)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (data "fog"))
  =>
  (assert (action-prevent-damage
    (source ?spell-id)
    (target "")
    (amount 0)
    (scope all-combat)
    (duration until-end-of-turn))))
