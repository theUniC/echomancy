;;; Lightning Strike — Deal 3 damage to any target.
;;;
;;; MVP: No targeting system yet. Targets the opponent of the caster.
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (data "lightning-strike")  — the card definition ID
;;;
;;; The rule finds the one player who is NOT the caster and deals 3 damage.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).
;;; Module isolation is deferred to when we have many card rules.

(defrule lightning-strike-resolve
  "Lightning Strike deals 3 damage to any target (MVP: opponent of caster)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (controller ?caster) (data "lightning-strike"))
  (player (id ?opponent&~?caster))
  =>
  (assert (action-damage (source ?spell-id) (target ?opponent) (amount 3))))
