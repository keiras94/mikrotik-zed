; ── Comments ─────────────────────────────────────────────────────
(comment) @comment

; ── Menu prefix / ────────────────────────────────────────────────
(menu_prefix) @string.special.path

; ── Menu command path segments and verbs ─────────────────────────
; Every identifier inside a menu_command (path parts, command verb)
; is highlighted as a builtin function.
(menu_command
  (identifier) @function.builtin)

; ── Global commands (:put, :local, :for, etc.) ─────────────────
(global_command_name) @keyword

; ── Control keywords ───────────────────────────────────────────
"do" @keyword.control
"else" @keyword.control
"while" @keyword.control

; ── Booleans ───────────────────────────────────────────────────
(boolean_literal) @boolean

; ── Nil ────────────────────────────────────────────────────────
(nil_literal) @constant.builtin

; ── Named parameters (property=value) ──────────────────────────
; The name part before = is a property
(named_param name: (identifier) @property)

; ── Function calls ─────────────────────────────────────────────
(function_call
  (identifier) @function.call)

; ── Variables ──────────────────────────────────────────────────
(variable_reference
  "$" @punctuation.special
  (identifier) @variable)

; ── Strings ────────────────────────────────────────────────────
(string) @string

; ── Numbers ────────────────────────────────────────────────────
(number) @number

; ── IP addresses / prefixes ────────────────────────────────────
(ip_address) @number
(ip_prefix) @number

; ── Arrays ─────────────────────────────────────────────────────
(array
  "{" @punctuation.bracket
  "}" @punctuation.bracket)

; ── Operators ──────────────────────────────────────────────────
(operator) @operator

; ── Brackets and punctuation ───────────────────────────────────
[
  "(" ")" "[" "]" "{" "}"
] @punctuation.bracket

; ── Statement separator ───────────────────────────────────────
";" @punctuation.delimiter

; ── Line continuation ─────────────────────────────────────────
(line_continuation) @punctuation.special

; ── Parent navigation ──────────────────────────────────────────
(parent_navigation) @string.special.path

; ── Command substitution ───────────────────────────────────────
(command_substitution
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

; ── Subexpressions ─────────────────────────────────────────────
(subexpression
  "(" @punctuation.bracket
  ")" @punctuation.bracket)

; ── Block delimiters ───────────────────────────────────────────
(block
  "{" @punctuation.bracket
  "}" @punctuation.bracket)
