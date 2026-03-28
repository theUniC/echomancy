/*
 * clips_helper.c — thin C shim to avoid exposing CLIPS internal struct
 * layouts to Rust. All struct field access happens here in C.
 *
 * This is part of the clips-sys PoC spike crate.
 */

#include "../clips-source/clips.h"
#include "../clips-source/factfun.h"
#include <string.h>

/*
 * Return the template (relation) name of a fact as a null-terminated C string.
 * Returns NULL if the fact is null or has no relation.
 */
const char *clips_fact_relation_name(Fact *fact) {
    if (fact == NULL) return NULL;
    CLIPSLexeme *rel = FactRelation(fact);
    if (rel == NULL) return NULL;
    return rel->contents;
}

/*
 * Read the integer value from a CLIPSValue that holds an INTEGER_TYPE.
 * Returns 0 if the value is not an integer.
 */
long long clips_value_as_integer(CLIPSValue *cv) {
    if (cv == NULL || cv->value == NULL) return 0;
    TypeHeader *h = (TypeHeader *) cv->value;
    if (h->type != INTEGER_TYPE) return 0;
    CLIPSInteger *i = (CLIPSInteger *) cv->value;
    return i->contents;
}

/*
 * Read the C string contents from a CLIPSValue that holds a SYMBOL_TYPE or STRING_TYPE.
 * Returns NULL if the value is not a lexeme type.
 */
const char *clips_value_as_string(CLIPSValue *cv) {
    if (cv == NULL || cv->value == NULL) return NULL;
    TypeHeader *h = (TypeHeader *) cv->value;
    if (h->type != SYMBOL_TYPE && h->type != STRING_TYPE && h->type != INSTANCE_NAME_TYPE) {
        return NULL;
    }
    CLIPSLexeme *lex = (CLIPSLexeme *) cv->value;
    return lex->contents;
}

/*
 * Return the type tag of a CLIPSValue (matches CLIPS type constants).
 * Returns 65535 (0xFFFF) if the value or its header is null.
 */
unsigned short clips_value_type(CLIPSValue *cv) {
    if (cv == NULL || cv->value == NULL) return 0xFFFF;
    TypeHeader *h = (TypeHeader *) cv->value;
    return h->type;
}
