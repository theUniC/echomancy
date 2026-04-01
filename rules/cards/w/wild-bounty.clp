;;; Wild Bounty — When it enters the battlefield, draw a card.
;;;
;;; Enchantment, {1}{G}. Simple non-aura enchantment.
;;; The ETB trigger is placed on the stack via the Rust trigger system (CR 603.3).
;;; When the triggered ability resolves, CLIPS fires on TRIGGERED_ABILITY_FIRES.
;;; The card definition ID is stored in target-id so we can match the right card.

(defrule wild-bounty-etb-draw
  "Wild Bounty: when the ETB triggered ability resolves, draw a card."
  (game-event (type TRIGGERED_ABILITY_FIRES) (controller ?caster) (target-id "wild-bounty"))
  =>
  (assert (action-draw (player ?caster) (amount 1))))
