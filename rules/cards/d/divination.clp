;;; Divination — Draw two cards.
;;;
;;; Sorcery, {2}{U}. The controller draws two cards when this resolves.
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (data "divination")        — the card definition ID
;;;
;;; Note: No defmodule — all rules live in MAIN for MVP (M5).

(defrule divination-resolve
  "Divination: draw two cards."
  (game-event (type SPELL_RESOLVING) (controller ?caster) (data "divination"))
  =>
  (assert (action-draw (player ?caster) (amount 2))))
