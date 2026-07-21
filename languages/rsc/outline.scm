; ── Outline / symbol view for RSC ─────────────────────────────
; @name is required by Zed for the symbol label.
; Each menu_command or global_command becomes one outline item.
(menu_command
  (menu_prefix)
  .
  (identifier) @name) @item

(global_command
  (identifier) @name) @item
