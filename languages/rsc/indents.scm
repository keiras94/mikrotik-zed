; ── Indentation rules for RSC ─────────────────────────────────

; Indent after opening a block or array {
(block "{" @indent.begin)
(block "}" @indent.end)

; Indent after line continuation
(line_continuation) @indent.continue

; Dedent before else={
(else_block "else" @indent.dedent)

; Dedent closing blocks
(block "}" @indent.zero)

; Indent after do={
(do_block "{" @indent.begin)
(else_block "{" @indent.begin)
