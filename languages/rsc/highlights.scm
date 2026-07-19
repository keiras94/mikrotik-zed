; ── Comments ─────────────────────────────────────────────────────
(comment) @comment

; ── Menu paths ──────────────────────────────────────────────────
; Menu prefix "/" 
(menu_prefix) @string.special.path

; Identifiers that are part of a menu path (first identifier after prefix)
(menu_command
  (identifier) @string.special.path
  (#not-has-parent? @string.special.path named_param))

; Menu path segments — all identifiers in menu_command before any named_param
(menu_command
  .
  (menu_prefix)
  (identifier) @string.special.path
  .
  (identifier) @string.special.path)

; ── Global commands (:put, :local, :if, :for, etc.) ────────────
(global_command_name) @keyword

; Control keywords
(global_command
  (global_command_name) @keyword.control
  (#match? @keyword.control ":(do|while|if|for|foreach|return|error|onerror|retry)$"))

; Storage keywords
(global_command
  (global_command_name) @keyword.storage.type
  (#match? @keyword.storage.type ":(local|global|set)$"))

; ── Control keywords ───────────────────────────────────────────
"do" @keyword.control
"else" @keyword.control
"while" @keyword.control
"in" @keyword

; ── Booleans ───────────────────────────────────────────────────
(boolean_literal) @boolean

; ── Nil ────────────────────────────────────────────────────────
(nil_literal) @constant.builtin

; ── Named parameters ───────────────────────────────────────────
(named_param name: (identifier) @property)
(named_param value: (string) @string)
(named_param value: (number) @number)
(named_param value: (boolean_literal) @boolean)

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
