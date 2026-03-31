;;; Giant Growth — Target creature gets +3/+3 until end of turn.
;;;
;;; The target chosen at cast time is forwarded into the SPELL_RESOLVING game-event's
;;; target-id slot by the Rust bridge.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "giant-growth")      — the card definition ID
;;;
;;; If target-id is empty (no target), the rule does not fire.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule giant-growth-resolve
  "Giant Growth gives its target +3/+3 until end of turn."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "giant-growth"))
  =>
  (assert (action-modify-pt
    (source ?spell-id)
    (target ?target)
    (power 3)
    (toughness 3)
    (duration until-end-of-turn))))
