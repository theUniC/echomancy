;;; Arcane Sanctum — When it enters the battlefield, draw a card.
;;;
;;; Enchantment, {1}{U}. Simplified MVP version of a non-aura enchantment.
;;; The card resolves to the battlefield (permanent type), then CLIPS fires on
;;; SPELL_RESOLVING to execute the ETB effect.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (data "arcane-sanctum")    — the card definition ID
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP.

(defrule arcane-sanctum-etb-draw
  "Arcane Sanctum: when it enters the battlefield, draw a card."
  (game-event (type SPELL_RESOLVING) (controller ?caster) (data "arcane-sanctum"))
  =>
  (assert (action-draw (player ?caster) (amount 1))))
