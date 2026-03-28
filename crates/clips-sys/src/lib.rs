//! Raw FFI bindings to CLIPS 6.4.2.
//!
//! This is a spike/PoC crate. Only the minimal surface area needed for the
//! SBA rules-engine evaluation is exposed. Do NOT use in production without
//! a proper safe wrapper layer.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_int, c_longlong, c_ulong, c_ushort};

// ---------------------------------------------------------------------------
// Opaque environment handle
// ---------------------------------------------------------------------------

/// Opaque CLIPS environment. Always accessed through a raw pointer.
#[repr(C)]
pub struct Environment {
    _private: [u8; 0],
}

/// Opaque CLIPS fact handle.
#[repr(C)]
pub struct Fact {
    _private: [u8; 0],
}

/// Opaque CLIPS deftemplate handle.
#[repr(C)]
pub struct Deftemplate {
    _private: [u8; 0],
}

/// Opaque CLIPS defmodule handle.
#[repr(C)]
pub struct Defmodule {
    _private: [u8; 0],
}

// ---------------------------------------------------------------------------
// CLIPSValue — the universal value union used for Eval results and slot reads
// ---------------------------------------------------------------------------

/// Inner union — mirrors `struct clipsValue` from entities.h.
/// We only surface the `value` (void*) pointer and the `header.type` field.
/// For the PoC we read results through `Eval` + `CLIPSLexeme` or integer paths.
#[repr(C)]
pub struct CLIPSValue {
    /// Discriminated union: void*, TypeHeader*, CLIPSLexeme*, etc.
    /// In practice we cast through `header` to read `type`, then pick the
    /// right union arm.
    pub value: *mut c_void,
}

/// The `TypeHeader` is the first field of every CLIPS heap object.
/// Reading `header.type` tells us what kind of value we have.
#[repr(C)]
pub struct TypeHeader {
    pub r#type: c_ushort,
}

/// A CLIPS lexeme (symbol or string) heap object.
#[repr(C)]
pub struct CLIPSLexeme {
    pub header: TypeHeader,
    pub busy_count: c_ulong,
    pub permanent: bool,
    pub bucket: c_int,
    /// Null-terminated C string contents — points into CLIPS-managed memory.
    pub contents: *const c_char,
}

/// A CLIPS integer heap object.
#[repr(C)]
pub struct CLIPSInteger {
    pub header: TypeHeader,
    pub busy_count: c_ulong,
    pub permanent: bool,
    pub bucket: c_int,
    pub contents: c_longlong,
}

/// A CLIPS float heap object.
#[repr(C)]
pub struct CLIPSFloat {
    pub header: TypeHeader,
    pub busy_count: c_ulong,
    pub permanent: bool,
    pub bucket: c_int,
    pub contents: f64,
}

// ---------------------------------------------------------------------------
// CLIPS type tag constants (from constant.h)
// ---------------------------------------------------------------------------

pub const FLOAT_TYPE: c_ushort = 0;
pub const INTEGER_TYPE: c_ushort = 1;
pub const SYMBOL_TYPE: c_ushort = 2;
pub const STRING_TYPE: c_ushort = 3;
pub const MULTIFIELD_TYPE: c_ushort = 4;
pub const EXTERNAL_ADDRESS_TYPE: c_ushort = 5;
pub const FACT_ADDRESS_TYPE: c_ushort = 6;
pub const INSTANCE_ADDRESS_TYPE: c_ushort = 7;
pub const INSTANCE_NAME_TYPE: c_ushort = 8;
pub const VOID_TYPE: c_ushort = 9;
pub const BOOLEAN_TYPE: c_ushort = 18;

// ---------------------------------------------------------------------------
// Error enums mirrored from CLIPS headers
// ---------------------------------------------------------------------------

/// EvalError from strngfun.h
#[repr(C)]
pub enum EvalError {
    NoError = 0,
    ParsingError = 1,
    ProcessingError = 2,
}

// ---------------------------------------------------------------------------
// Core FFI declarations
// ---------------------------------------------------------------------------

extern "C" {
    // --- Environment lifecycle ---

    /// Create a new CLIPS environment. Returns null on failure.
    pub fn CreateEnvironment() -> *mut Environment;

    /// Destroy a CLIPS environment and free all its resources.
    pub fn DestroyEnvironment(env: *mut Environment) -> bool;

    // --- Loading constructs ---

    /// Load constructs (defrule, deftemplate, etc.) from a string.
    /// `len` is the byte length of the string, or `SIZE_MAX` to use strlen.
    pub fn LoadFromString(env: *mut Environment, code: *const c_char, len: usize) -> bool;

    // --- Fact assertion ---

    /// Assert a fact from a string representation, e.g. `"(sba-destroy c1)"`.
    /// Returns null on failure (check `GetAssertStringError`).
    pub fn AssertString(env: *mut Environment, fact: *const c_char) -> *mut Fact;

    /// Retract all facts from the environment.
    pub fn RetractAllFacts(env: *mut Environment) -> c_int;

    // --- Rule execution ---

    /// Run inference engine for up to `limit` rule firings (-1 = unlimited).
    /// Returns the number of rules actually fired.
    pub fn Run(env: *mut Environment, limit: c_longlong) -> c_longlong;

    /// Reset the environment: retract all facts, re-assert deffacts.
    pub fn Reset(env: *mut Environment);

    // --- Expression evaluation ---

    /// Evaluate a CLIPS expression string. Stores result in `result`.
    /// Returns EvalError code.
    pub fn Eval(env: *mut Environment, expr: *const c_char, result: *mut CLIPSValue) -> EvalError;

    // --- Fact navigation ---

    /// Get the first fact, or the fact after `fact` (pass null for first).
    pub fn GetNextFact(env: *mut Environment, fact: *mut Fact) -> *mut Fact;

    /// Get the index of a fact.
    pub fn FactIndex(fact: *mut Fact) -> c_longlong;

    /// Get a slot value from a fact. Stores result in `result`.
    pub fn GetFactSlot(
        fact: *mut Fact,
        slot_name: *const c_char,
        result: *mut CLIPSValue,
    ) -> c_int;

    /// Find a deftemplate by name.
    pub fn FindDeftemplate(env: *mut Environment, name: *const c_char) -> *mut Deftemplate;

    /// Get the first fact for a specific deftemplate, or the next after `fact`.
    pub fn GetNextFactInTemplate(
        deftemplate: *mut Deftemplate,
        fact: *mut Fact,
    ) -> *mut Fact;

    /// Get the fact list as a CLIPSValue (multifield of fact addresses).
    pub fn GetFactList(env: *mut Environment, result: *mut CLIPSValue, module: *mut Defmodule);

    /// Get the template (relation) name of a fact.
    /// Returns the `CLIPSLexeme` whose `contents` is the null-terminated template name.
    pub fn FactRelation(fact: *mut Fact) -> *mut CLIPSLexeme;
}

// ---------------------------------------------------------------------------
// C helper shim functions (from clips-source-helper/clips_helper.c)
// These avoid unsafe struct field access from Rust.
// ---------------------------------------------------------------------------

extern "C" {
    /// Return the template name of a fact as a C string. Returns null if fact is null.
    pub fn clips_fact_relation_name(fact: *mut Fact) -> *const c_char;

    /// Read the integer contents of a CLIPSValue (must be INTEGER_TYPE). Returns 0 otherwise.
    pub fn clips_value_as_integer(cv: *mut CLIPSValue) -> c_longlong;

    /// Read the string contents of a CLIPSValue (SYMBOL or STRING type). Returns null otherwise.
    pub fn clips_value_as_string(cv: *mut CLIPSValue) -> *const c_char;

    /// Return the type tag of a CLIPSValue. Returns 0xFFFF if null.
    pub fn clips_value_type(cv: *mut CLIPSValue) -> c_ushort;
}
