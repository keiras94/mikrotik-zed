; ── Indentation rules for RSC ─────────────────────────────────
(block "{" @indent)
(block "}" @outdent)
(command_substitution "[" @indent)
(command_substitution "]" @outdent)
(subexpression "(" @indent)
(subexpression ")" @outdent)
