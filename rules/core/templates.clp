;;; Core CLIPS deftemplate definitions for Echomancy.
;;;
;;; This file defines all input and output fact schemas used in the
;;; Game state -> CLIPS bridge (M2) and the action collection (M2).
;;;
;;; Input facts are asserted by Rust before Run(). Output (action-*) facts
;;; are asserted by CLIPS rules and collected by Rust after Run().

;;; ============================================================================
;;; Input facts: game state (Rust -> CLIPS)
;;; ============================================================================

(deftemplate player
  "A player in the game."
  (slot id (type STRING))
  (slot life (type INTEGER))
  (slot is-active (type SYMBOL) (allowed-symbols TRUE FALSE))
  (slot has-priority (type SYMBOL) (allowed-symbols TRUE FALSE)))

(deftemplate permanent
  "A permanent on a player's battlefield."
  (slot instance-id (type STRING))
  (slot card-id (type STRING))
  (slot card-name (type STRING))
  (slot controller (type STRING))
  (slot owner (type STRING))
  (slot zone (type SYMBOL) (default battlefield))
  (slot card-type (type SYMBOL))
  (slot tapped (type SYMBOL) (allowed-symbols TRUE FALSE) (default FALSE))
  (slot summoning-sick (type SYMBOL) (allowed-symbols TRUE FALSE) (default FALSE))
  (slot power (type INTEGER) (default 0))
  (slot toughness (type INTEGER) (default 0))
  (slot damage (type INTEGER) (default 0))
  (multislot keywords)
  (multislot counters))

(deftemplate mana-pool
  "The current mana pool for a player."
  (slot player-id (type STRING))
  (slot white (type INTEGER) (default 0))
  (slot blue (type INTEGER) (default 0))
  (slot black (type INTEGER) (default 0))
  (slot red (type INTEGER) (default 0))
  (slot green (type INTEGER) (default 0))
  (slot colorless (type INTEGER) (default 0)))

(deftemplate stack-item
  "An item on the stack (spell or ability)."
  (slot id (type STRING))
  (slot card-id (type STRING))
  (slot controller (type STRING))
  (slot status (type SYMBOL))
  (slot target (type STRING) (default "")))

(deftemplate turn-state
  "The current turn and step state."
  (slot current-step (type SYMBOL))
  (slot active-player (type STRING))
  (slot turn-number (type INTEGER)))

(deftemplate game-event
  "A transient event asserted once per evaluation cycle."
  (slot type (type SYMBOL))
  (slot source-id (type STRING) (default ""))
  (slot controller (type STRING) (default ""))
  (slot target-id (type STRING) (default ""))
  (slot data (type STRING) (default "")))

(deftemplate attached
  "An enchantment or equipment attached to a permanent (M4+)."
  (slot enchantment-id (type STRING))
  (slot target-id (type STRING)))

;;; ============================================================================
;;; Output facts: actions proposed by CLIPS rules (CLIPS -> Rust)
;;; ============================================================================

(deftemplate action-draw
  "A player draws one or more cards."
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-damage
  "Deal damage from a source to a target."
  (slot priority (type INTEGER) (default 100))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-destroy
  "Destroy a permanent."
  (slot priority (type INTEGER) (default 100))
  (slot target (type STRING)))

(deftemplate action-gain-life
  "A player gains life."
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-lose-life
  "A player loses life."
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-move-zone
  "Move a card from one zone to another."
  (slot priority (type INTEGER) (default 100))
  (slot card-id (type STRING))
  (slot from-zone (type SYMBOL))
  (slot to-zone (type SYMBOL)))

(deftemplate action-add-mana
  "Add mana to a player's pool."
  (slot priority (type INTEGER) (default 100))
  (slot player (type STRING))
  (slot color (type SYMBOL))
  (slot amount (type INTEGER)))

(deftemplate action-tap
  "Tap a permanent."
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING)))

(deftemplate action-untap
  "Untap a permanent."
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING)))

(deftemplate action-add-counter
  "Add a counter to a permanent."
  (slot priority (type INTEGER) (default 100))
  (slot permanent-id (type STRING))
  (slot counter-type (type STRING))
  (slot amount (type INTEGER)))

(deftemplate action-create-token
  "Create a token permanent on the battlefield."
  (slot priority (type INTEGER) (default 100))
  (slot controller (type STRING))
  (slot name (type STRING))
  (slot power (type INTEGER))
  (slot toughness (type INTEGER))
  (multislot types)
  (multislot keywords))

(deftemplate action-counter-spell
  "Counter a spell on the stack (remove it, put in owner's graveyard)."
  (slot priority (type INTEGER) (default 0))
  (slot target (type STRING)))

(deftemplate action-modify-pt
  "Apply a temporary power/toughness modification to a creature (until end of turn)."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot power (type INTEGER))
  (slot toughness (type INTEGER))
  (slot duration (type SYMBOL) (default until-end-of-turn)))

(deftemplate action-set-pt
  "Set a creature's power and toughness to specific values (Layer 7b, until end of turn)."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot power (type INTEGER))
  (slot toughness (type INTEGER))
  (slot duration (type SYMBOL) (default until-end-of-turn)))

(deftemplate action-switch-pt
  "Switch a creature's power and toughness (Layer 7d, until end of turn)."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot duration (type SYMBOL) (default until-end-of-turn)))

(deftemplate action-remove-all-abilities
  "Remove all abilities from a creature (Layer 6, until end of turn)."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot duration (type SYMBOL) (default until-end-of-turn)))

(deftemplate action-prevent-damage
  "Register a damage prevention shield on a target (R11, CR 615).
   Prevents up to amount damage the next time damage would be dealt to target."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING))
  (slot amount (type INTEGER))
  (slot duration (type SYMBOL) (default next-occurrence)))

(deftemplate action-regenerate
  "Register a regeneration shield on a creature (R11, CR 701.15).
   The next time the creature would be destroyed, instead tap it,
   remove all damage, and remove it from combat."
  (slot priority (type INTEGER) (default 0))
  (slot source (type STRING))
  (slot target (type STRING)))

;;; ============================================================================
;;; Signal facts: CLIPS requesting player input
;;; ============================================================================

(deftemplate awaiting-input
  "Signals that a rule needs player input before continuing (triggers halt)."
  (slot type (type SYMBOL))
  (slot player (type STRING))
  (slot prompt (type STRING)))
