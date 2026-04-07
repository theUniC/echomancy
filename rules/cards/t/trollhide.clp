;;; Trollhide — Regenerate target creature.
;;;
;;; The target chosen at cast time is forwarded into the SPELL_RESOLVING game-event's
;;; target-id slot by the Rust bridge.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "trollhide")         — the card definition ID
;;;
;;; If target-id is empty (no target), the rule does not fire.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule trollhide-resolve
  "Trollhide regenerates target creature."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "trollhide"))
  =>
  (assert (action-regenerate
    (source ?spell-id)
    (target ?target))))
