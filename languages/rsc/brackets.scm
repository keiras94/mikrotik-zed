; ── Bracket matching for RSC ──────────────────────────────────

("(" @opening ")" @closing)
("[" @opening "]" @closing)
("{" @opening "}" @closing)
("\"" @opening "\"" @closing)

; ── Command substitution ───────────────────────────────────────
(command_substitution
  "[" @opening
  "]" @closing)

(subexpression
  "(" @opening
  ")" @closing)

; ── Block delimiters ───────────────────────────────────────────
(block
  "{" @opening
  "}" @closing)
