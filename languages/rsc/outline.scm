; ── Outline / symbol view for RSC ─────────────────────────────

; Menu commands as sections
(menu_command) @item

; Global commands with descriptive names
(global_command
  (global_command_name) @name
  (#match? @name ":(local|global|if|for|foreach|do|while)")
) @item
