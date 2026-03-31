;;; Wild Bounty — When it enters the battlefield, draw a card.
;;;
;;; Enchantment, {1}{G}. Simple non-aura enchantment.
;;; The card resolves to the battlefield (permanent type), then CLIPS fires on
;;; SPELL_RESOLVING to execute the ETB effect.

(defrule wild-bounty-etb-draw
  "Wild Bounty: when it enters the battlefield, draw a card."
  (game-event (type SPELL_RESOLVING) (controller ?caster) (data "wild-bounty"))
  =>
  (assert (action-draw (player ?caster) (amount 1))))
