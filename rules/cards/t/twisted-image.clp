;;; Twisted Image — Switch target creature's power and toughness until end of turn. Draw a card.
;;;
;;; Tests Layer 7d (SwitchPowerToughness).
;;;
;;; The game-event (type SPELL_RESOLVING) carries:
;;;   (source-id <instance-id>)  — the spell instance
;;;   (controller <player-id>)   — the casting player
;;;   (target-id <target-id>)    — the chosen target (permanent ID)
;;;   (data "twisted-image")     — the card definition ID
;;;
;;; If target-id is empty (no target), the switch rule does not fire.
;;; The draw always fires (controller draws a card).

(defrule twisted-image-switch-pt
  "Twisted Image switches power and toughness of the target."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (target-id ?target&~"") (data "twisted-image"))
  =>
  (assert (action-switch-pt
    (source ?spell-id)
    (target ?target)
    (duration until-end-of-turn))))

(defrule twisted-image-draw
  "Twisted Image draws a card for the controller."
  (game-event (type SPELL_RESOLVING) (source-id ?spell-id) (controller ?ctrl) (data "twisted-image"))
  =>
  (assert (action-draw
    (player ?ctrl)
    (amount 1))))
