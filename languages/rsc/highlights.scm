; ── Comments ─────────────────────────────────────────────────────
(comment) @comment

; ── Menu prefix ──────────────────────────────────────────────────
(menu_prefix) @string.special.path

; ── Global commands (:put, :local, :if, :for, etc.) ────────────
(global_command_name) @keyword

; ── Control keywords ───────────────────────────────────────────
"do" @keyword.control
"else" @keyword.control
"while" @keyword.control

; ── Booleans ───────────────────────────────────────────────────
(boolean_literal) @boolean

; ── Nil ────────────────────────────────────────────────────────
(nil_literal) @constant.builtin

; ── Named parameters ───────────────────────────────────────────
(named_param name: (identifier) @property)

; ── Variables ──────────────────────────────────────────────────
(variable_reference
  "$" @punctuation.special
  (identifier) @variable)

; ── Strings ────────────────────────────────────────────────────
(string) @string

; ── Numbers ────────────────────────────────────────────────────
(number) @number

; ── IP addresses ───────────────────────────────────────────────
(ip_address) @constant
(ip_prefix) @constant

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

; ── Command substitution brackets ──────────────────────────────
(command_substitution
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

(subexpression
  "(" @punctuation.bracket
  ")" @punctuation.bracket)

; ── Block delimiters ───────────────────────────────────────────
(block
  "{" @punctuation.bracket
  "}" @punctuation.bracket)
