; ── Indentation rules for RSC ─────────────────────────────────
(block "{" @indent)
(block "}" @outdent)
(command_substitution "[" @indent)
(command_substitution "]" @outdent)
(subexpression "(" @indent)
(subexpression ")" @outdent)
; Line continuation: after \ the next line should be indented
(line_continuation) @indent
