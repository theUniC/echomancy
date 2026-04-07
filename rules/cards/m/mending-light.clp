;;; Mending Light — Prevent the next 3 damage that would be dealt to target creature.
;;;
;;; The target chosen at cast time is forwarded into the SPELL_RESOLVING game-event's
;;; target-id slot by the Rust bridge.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "mending-light")     — the card definition ID
;;;
;;; If target-id is empty (no target), the rule does not fire.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule mending-light-resolve
  "Mending Light prevents the next 3 damage to target creature."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "mending-light"))
  =>
  (assert (action-prevent-damage
    (source ?spell-id)
    (target ?target)
    (amount 3)
    (duration until-depleted))))
