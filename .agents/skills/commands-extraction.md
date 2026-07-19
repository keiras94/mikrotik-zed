# Skill: Commands Table Extraction

## Purpose

Regenerate `data/commands.toml` from the RouterOS documentation truth source (`llms-full.txt`). This file is consumed by the Phase 2 language server for autocompletion and hover documentation.

## Truth Sources

| File | Role |
|------|------|
| `llms-full.txt` | Source of truth — full RouterOS 7.22 docs |
| `llms.txt` | Index with page URLs (for cross-referencing) |
| `data/commands.toml` | Generated output — command table |
| `scripts/extract_commands.py` | Extraction script |

## When to Regenerate

- When a new RouterOS v7 major release ships
- When the extraction script is improved
- When `commands.toml` is missing expected commands
- After manually editing `llms-full.txt` (not recommended)

## How to Regenerate

```bash
cd /Users/Francisco/Developer/mikrotik-zed
python3 scripts/extract_commands.py
```

Output:
```
Parsing /Users/Francisco/Developer/mikrotik-zed/llms-full.txt...
Found N total entries, M match target menus.
Wrote /Users/Francisco/Developer/mikrotik-zed/data/commands.toml (M menus)
```

## Output Format

The generated `commands.toml` uses this structure:

```toml
[[menus]]
path = "/ip/firewall/filter"
type = "Directory"

[[menus.flags]]
name = "X"
description = "disabled"

[[menus.arguments]]
name = "chain"
type = "enum"
required = true
description = "Chain name"

[[menus.arguments]]
name = "action"
type = "enum (accept | drop | jump | return | log)"
description = "Action to take"

[[menus.read_only]]
name = "bytes"
type = "num"
```

## Target Menus (Scoped)

The extraction filters to only these root menus:

```python
TARGET_ROOTS = {
    "/ip": {"address", "route", "firewall", "dhcp-server", "dns", "service"},
    "/ipv6": {"address", "dhcp-client", "nd", "firewall", "route"},
    "/interface": {"bridge", "vlan", "pppoe-client", "ethernet"},
    "/routing": {"ospf", "bgp", "table", "rule"},
}
```

Firewall sub-menus (`/ip/firewall/filter`, `/ip/firewall/nat`, etc.) are included when `firewall` is in the allowed set.

## Extraction Logic

The script (`extract_commands.py`) parses `llms-full.txt` by:

1. **Detecting menu paths** from `##`, `###`, `####` headings containing `/`
2. **Reading menu type** from `**Type:** Directory` lines
3. **Parsing ArgTable sections** — `<ArgTable c1="Flag|Argument|Read-only Argument">`
4. **Extracting ArgTableRow entries** with attributes:
   - `arg="name"` — property name
   - `typ="type"` — data type
   - `mandatory="1"` — required flag
   - `unset="1"` — can be unset
   - Text content — description

## Verification

After regeneration, verify:

```bash
# Check total menu count
rg -c '^\[\[menus\]\]' data/commands.toml

# Check specific paths exist
rg 'path = "/ip/firewall/filter"' data/commands.toml
rg 'path = "/interface/bridge"' data/commands.toml
rg 'path = "/routing/bgp"' data/commands.toml

# Check arguments have types (not empty)
rg 'type = ""' data/commands.toml | wc -l  # Should be minimal

# Spot-check a known property
rg -A 3 'name = "chain"' data/commands.toml
```

## Improving the Script

If extraction misses entries:

1. Search `llms-full.txt` for the missing menu manually
2. Understand the heading/ArgTable format variation
3. Update `parse_llms_full()` in `extract_commands.py`
4. Re-run and verify

Common issues:
- Some menus use `####` instead of `###` for sub-paths
- ArgTable format varies between `c1="Flag"`, `c1="Argument"`, `c1="Read-only Argument"`
- Some entries use markdown tables instead of ArgTable XML
- Trailing `...` in enum types needs cleanup

## Rules

- **Always regenerate from `llms-full.txt`**, never manually edit `commands.toml` for bulk changes.
- The script is the single source of truth for how extraction works.
- If you need to add a property manually (for a missing entry), add a comment explaining why.
- Keep `commands.toml` committed to git — it's the versioned snapshot for the extension.
