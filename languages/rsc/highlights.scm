; ── Comments ─────────────────────────────────────────────────────
(comment) @comment

; ── Menu prefix / ────────────────────────────────────────────────
(menu_prefix) @string.special.path

; ── Known command verbs inside menu_command ──────────────────────
; These are highlighted as keywords so they stand out from path
; segments and property names.
(menu_command
  [
    "add" "remove" "set" "get" "print" "enable" "disable"
    "find" "comment" "move" "export" "import" "edit"
    "monitor" "reset-counters" "check" "clear" "flush" "renew"
    "release" "scan" "blink" "pause" "reload" "power-cycle"
  ] @keyword.function)

; ── Other path segments and identifiers in menu_command ─────────
(menu_command
  (identifier) @function.builtin)

; ── Global commands (:put, :local, :for, etc.) ─────────────────
(global_command_name) @keyword

; ── Control keywords ───────────────────────────────────────────
"do" @keyword.control
"else" @keyword.control
"while" @keyword.control

; The = in `do = {` and `else = {` blocks
(do_block "=" @operator)
(else_block "=" @operator)

; ── Booleans ───────────────────────────────────────────────────
(boolean_literal) @boolean

; ── Nil ────────────────────────────────────────────────────────
(nil_literal) @constant.builtin

; ── Named parameters (property=value) ──────────────────────────
; The name part before = is a property
(named_param name: (identifier) @property)

; The = sign in property=value assignments
(named_param "=" @operator)

; ── Array values ───────────────────────────────────────────────
; Commas between array values
(array "," @punctuation.delimiter)

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
