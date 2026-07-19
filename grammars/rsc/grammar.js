/**
 * Tree-sitter grammar for MikroTik RouterOS Script (RSC) — simplified.
 *
 * Covers RouterOS 7.22+ scripting language.
 * This grammar focuses on structural parsing. Operators are captured
 * at the token level for highlighting but not structurally parsed.
 *
 * Reference: https://manual.mikrotik.com/llms-full.txt
 */

/// <reference types="tree-sitter-cli/dsl" />

module.exports = grammar({
  name: "rsc",

  extras: ($) => [
    /[ \t]+/,
    /\r/,
    $.comment,
  ],

  word: ($) => $.identifier,

  conflicts: ($) => [
    // subexpression `(...)` vs value-containing-parens ambiguity
    [$.subexpression, $._value],
  ],

  rules: {
    // ── Top level ──────────────────────────────────────────────
    source_file: ($) =>
      seq(
        optional($._statement),
        repeat($._terminated_statement),
        optional(choice(";", "\n")),
      ),

    _terminated_statement: ($) =>
      seq($._statement_separator, $._statement),

    _statement_separator: ($) => choice(";", "\n"),

    _statement: ($) =>
      choice(
        $.menu_command,
        $.global_command,
        $._value,
        $.line_continuation,
        $.parent_navigation,
      ),

    line_continuation: ($) => "\\",
    parent_navigation: ($) => token(".."),

    // ── Menu commands: /path param* ───────────────────────────
    menu_command: ($) =>
      prec(2, seq(
        $.menu_prefix,
        optional($.menu_path),
        repeat($.named_param),
        repeat($._value),
      )),

    menu_prefix: ($) => "/",
    menu_path: ($) =>
      seq($.identifier, repeat1(seq("/", $.identifier))),

    // ── Global commands: :name body? param* ──────────────────
    global_command: ($) =>
      prec(1, seq(
        $.global_command_name,
        optional($._command_body),
        repeat($.named_param),
        repeat($._value),
      )),

    global_command_name: ($) =>
      seq(":", $.identifier),

    _command_body: ($) =>
      choice(
        $.do_block,
        $.else_block,
        $.while_condition,
        $.for_in_clause,
      ),

    do_block: ($) => seq("do", "=", $.block),
    else_block: ($) => seq("else", "=", $.block),
    while_condition: ($) => seq("while", "=", $.subexpression),
    for_in_clause: ($) => seq("in", "=", $._value),

    // ── Named param: key=value ───────────────────────────────
    named_param: ($) =>
      prec(1, seq(
        field("name", $.identifier),
        "=",
        field("value", $._value),
      )),

    // ── Block: { ... } ──────────────────────────────────────
    block: ($) =>
      seq(
        "{",
        optional($._statement),
        repeat($._terminated_statement),
        optional(choice(";", "\n")),
        "}",
      ),

    // ── Values ───────────────────────────────────────────────
    _value: ($) =>
      choice(
        $.literal,
        $.variable_reference,
        $.command_substitution,
        $.subexpression,
        $.array,
        $.array_access,
        $.function_call,
        $.identifier,
        $.operator,
      ),

    // ── Literals ─────────────────────────────────────────────
    literal: ($) =>
      choice(
        $.number,
        $.string,
        $.boolean_literal,
        $.nil_literal,
        $.ip_address,
        $.ip_prefix,
      ),

    // ── Operators (token only, not structured) ──────────────
    operator: ($) =>
      token(choice(
        "&&", "||",
        "!=", "<=", ">=", "=", "<", ">",
        "+", "-", "*", "/", "%",
        "&", "|", "^", "<<", ">>",
        "~", ".", ",", "!", "->",
      )),

    // ── Variable references ─────────────────────────────────
    variable_reference: ($) =>
      seq("$", $.identifier),

    array_access: ($) =>
      seq(
        field("array", choice($.variable_reference, $.identifier)),
        "->",
        field("key", choice($.string, $.identifier, $.number)),
      ),

    // ── Command substitution: [cmd] ─────────────────────────
    command_substitution: ($) =>
      seq(
        "[",
        $._statement,
        "]",
      ),

    subexpression: ($) =>
      seq(
        "(",
        $._value,
        repeat($.operator),
        repeat($._value),
        ")",
      ),

    // ── Arrays: { ... } ────────────────────────────────────
    array: ($) =>
      seq(
        "{",
        optional($._array_body),
        "}",
      ),

    _array_body: ($) =>
      seq(
        $._array_element,
        repeat(seq(";", $._array_element)),
        optional(";"),
      ),

    _array_element: ($) =>
      choice(
        prec(1, $.named_param),
        $._value,
      ),

    // ── Function call: $func params ─────────────────────────
    function_call: ($) =>
      prec(1, seq(
        field("function", $.variable_reference),
        repeat1($._value),
      )),

    // ── Tokens ──────────────────────────────────────────────
    identifier: ($) =>
      /[a-zA-Z_][a-zA-Z0-9_-]*/,

    number: ($) =>
      token(choice(
        /0[xX][0-9a-fA-F]+/,
        /[0-9]+/,
      )),

    string: ($) =>
      token(seq(
        '"',
        repeat(choice(
          /[^"\\\n\r]+/,
          /\\./,
        )),
        '"',
      )),

    boolean_literal: ($) =>
      token(choice("true", "false", "yes", "no")),

    nil_literal: ($) => token("nil"),

    ip_address: ($) =>
      token(choice(
        /[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/,
        /[0-9a-fA-F:]+:[0-9a-fA-F:]+/,
      )),

    ip_prefix: ($) =>
      token(seq(
        /[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/,
        "/",
        /[0-9]+/,
      )),

    // ── Comment: # ... ─────────────────────────────────────
    comment: ($) =>
      token(seq("#", /.*/)),
  },
});
