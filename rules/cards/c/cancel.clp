;;; Cancel — Counter target spell.
;;;
;;; Instant, {1}. Simplified counterspell.
;;; When Cancel resolves, the targeted spell is removed from the stack
;;; and put into its owner's graveyard.

(defrule cancel-counter-spell
  "Cancel: counter target spell."
  (game-event (type SPELL_RESOLVING) (data "cancel") (target ?target-spell))
  =>
  (assert (action-counter-spell (target ?target-spell))))
