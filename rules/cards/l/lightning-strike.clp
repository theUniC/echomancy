;;; Lightning Strike — Deal 3 damage to any target.
;;;
;;; The target chosen at cast time is forwarded into the SPELL_RESOLVING game-event's
;;; target-id slot by the Rust bridge. This avoids heuristics like searching for the
;;; opponent and instead reads the player's actual choice.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (player ID or permanent ID)
;;;   (data "lightning-strike")  — the card definition ID
;;;
;;; If target-id is empty (no target recorded), the rule does not fire —
;;; handling fizzle when the target was removed or the field is blank.
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).
;;; Module isolation is deferred to when we have many card rules.

(defrule lightning-strike-resolve
  "Lightning Strike deals 3 damage to its chosen target (read from the game-event fact)."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "lightning-strike"))
  =>
  (assert (action-damage (source ?spell-id) (target ?target) (amount 3))))
