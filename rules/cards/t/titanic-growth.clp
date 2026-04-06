;;; Titanic Growth — Target creature gets +4/+4 until end of turn.
;;;
;;; Tests Layer 7c (ModifyPowerToughness) — a larger pump than Giant Growth,
;;; useful for making layer interactions more visible in testing.
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "titanic-growth")    — the card definition ID
;;;
;;; If target-id is empty (no target), the rule does not fire.

(defrule titanic-growth-resolve
  "Titanic Growth gives its target +4/+4 until end of turn."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "titanic-growth"))
  =>
  (assert (action-modify-pt
    (source ?spell-id)
    (target ?target)
    (power 4)
    (toughness 4)
    (duration until-end-of-turn))))
