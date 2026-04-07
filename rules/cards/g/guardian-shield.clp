;;; Guardian Shield — Prevent all damage that would be dealt to target creature this turn.
;;;
;;; The target chosen at cast time is forwarded into the SPELL_RESOLVING game-event's
;;; target-id slot by the Rust bridge.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "guardian-shield")   — the card definition ID
;;;
;;; If target-id is empty (no target), the rule does not fire.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule guardian-shield-resolve
  "Guardian Shield prevents all damage to target creature this turn (scope targeted)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "guardian-shield"))
  =>
  (assert (action-prevent-damage
    (source ?spell-id)
    (target ?target)
    (amount 0)
    (scope targeted)
    (duration until-end-of-turn))))
