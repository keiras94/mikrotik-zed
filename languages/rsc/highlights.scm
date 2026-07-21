; ── MikroTik RouterOS Script — highlights ────────────────────────
; Color scheme (matches RouterOS terminal):
;   Blue   = root menu (first command after /)
;   Green  = sub-menus and commands
;   Orange = properties/variables (name=value)
;   Red    = strings
;   Cyan   = numbers, IPs
;   Grey   = comments

; ── Comments ─────────────────────────────────────────────────────
(comment) @comment

; ── Menu prefix "/" ──────────────────────────────────────────────
(menu_prefix) @punctuation.special

; ── Root menu — first command after / (blue) ────────────────────
; e.g. "ip" in /ip route add …
(root_menu
  (identifier) @function)

; ── Sub-menus — subsequent segments (green) ────────────────────
; e.g. "route", "add" in /ip route add …

; ── Catch-all: bare identifiers in menu_command → orange ───────
; These are typically values after line continuation like `password=\nvalue`
; Must come BEFORE sub_menu/named_param so specific rules override it
(menu_command
  (identifier) @constant)

(sub_menu
  (identifier) @string)

; ── Action commands (purple) ──────────────────────────────────
; Override sub_menu green for commands that modify/query state
((sub_menu
  (identifier) @keyword)
  (#match? @keyword "^(add|remove|set|get|print|enable|disable|find|comment|move|export|import|edit|reset|force-update|beep|blink|password|quit|redo|undo|ping)$"))

; ── Identifiers inside command_substitution / menu_continuation ──
; Commands like "find", "set" inside [...] → green
(command_substitution
  (identifier) @string)

; Action commands inside [...] → purple
((command_substitution
  (identifier) @keyword)
  (#match? @keyword "^(add|remove|set|get|print|enable|disable|find|comment|move|export|import|edit|reset|force-update|beep|blink|password|quit|redo|undo|ping)$"))

(menu_continuation
  (identifier) @string)

; Action commands in continuation → purple
((menu_continuation
  (identifier) @keyword)
  (#match? @keyword "^(add|remove|set|get|print|enable|disable|find|comment|move|export|import|edit|reset|force-update|beep|blink|password|quit|redo|undo|ping)$"))

; ── Named parameters — property=value ──────────────────────────
; Property name → yellow (like the MikroTik terminal)
(named_param
  name: (identifier) @type)

; Property value (identifiers like ether1, bridge) → orange
(named_param
  value: (identifier) @constant)

; ── = sign in named params ──────────────────────────────────────
(named_param "=" @operator)

; ── Global commands (:put, :local, :for, etc.) ──────────────────
(global_command_name) @keyword

; ── Control flow keywords ───────────────────────────────────────
"do" @keyword.control
"else" @keyword.control
"while" @keyword.control

(do_block "=" @operator)
(else_block "=" @operator)

; ── Booleans ────────────────────────────────────────────────────
(boolean_literal) @boolean

; ── Nil ─────────────────────────────────────────────────────────
(nil_literal) @constant.builtin

; ── Function calls ──────────────────────────────────────────────
(function_call
  (identifier) @function.call)

; ── Variables ───────────────────────────────────────────────────
(variable_reference
  "$" @punctuation.special
  (identifier) @variable)

; ── Strings ────────────────────────────────────────────────────
(string) @string.special

; ── Numbers ─────────────────────────────────────────────────────
(number) @number

; ── IP addresses / prefixes ─────────────────────────────────────
(ip_address) @number
(ip_prefix) @number

; ── Arrays ──────────────────────────────────────────────────────
(array
  "{" @punctuation.bracket
  "}" @punctuation.bracket)

; ── Operators ───────────────────────────────────────────────────
(operator) @operator

; ── Brackets ────────────────────────────────────────────────────
[
  "(" ")" "[" "]" "{" "}"
] @punctuation.bracket

; ── Statement separator ────────────────────────────────────────
";" @punctuation.delimiter

; ── Line continuation ──────────────────────────────────────────
(line_continuation) @punctuation.special

; ── Parent navigation ───────────────────────────────────────────
(parent_navigation) @string.special.path

; ── Command substitution ────────────────────────────────────────
(command_substitution
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

; ── Subexpressions ──────────────────────────────────────────────
(subexpression
  "(" @punctuation.bracket
  ")" @punctuation.bracket)

; ── Block delimiters ────────────────────────────────────────────
(block
  "{" @punctuation.bracket
  "}" @punctuation.bracket)
