# Skill: RouterOS Command Reference Lookup

## Purpose

Look up MikroTik RouterOS 7.22 CLI commands, properties, types, and descriptions from the project's truth sources. Never guess RouterOS syntax — always verify against the docs.

## Truth Sources

| Source | Location | Use For |
|--------|----------|---------|
| `llms-full.txt` | Project root (`/Users/Francisco/Developer/mikrotik-zed/llms-full.txt`) | Full command reference with ArgTable rows, types, descriptions |
| `llms.txt` | Project root | Index of all doc pages with URLs and summaries |
| `data/commands.toml` | Project root | Pre-extracted command table for the 4 target menus |

## Target Menus (Phase 2 Scope)

Only these 4 root menus are in scope:

- `/ip` — address, route, firewall, dhcp-server, dns, service
- `/ipv6` — address, dhcp-client, nd, firewall, route
- `/interface` — bridge, vlan, pppoe-client, ethernet
- `/routing` — ospf, bgp, table, rule

## How to Look Up a Command

### Step 1: Check `data/commands.toml` first

```bash
rg -n "/ip/firewall" data/commands.toml
rg -n 'path = "/ip/address"' data/commands.toml
```

The TOML file contains `[[menus]]` entries with:
- `path` — CLI menu path (e.g., `/ip/firewall/filter`)
- `type` — Directory, Command, or Settings Directory
- `[[menus.flags]]` — Output flags (Y=managed, D=dynamic, X=disabled, R=running)
- `[[menus.arguments]]` — Writable properties with `name`, `type`, `description`, `required`
- `[[menus.read_only]]` — Read-only properties

### Step 2: Search `llms-full.txt` for detailed docs

```bash
rg -n "## /ip/firewall/filter" llms-full.txt
rg -n "ArgTable" llms-full.txt | head -20
rg -n "arg=\"chain\"" llms-full.txt
```

The `llms-full.txt` format uses:
- `## /path/to/menu` — Section headings with menu paths
- `**Type:** Directory` — Menu type declaration
- `<ArgTable c1="Flag">` — Flag table start
- `<ArgTable c1="Argument">` — Argument table start
- `<ArgTable c1="Read-only Argument">` — Read-only table start
- `<ArgTableRow arg="name" typ="type" mandatory="1">Description</ArgTableRow>` — Property row

### Step 3: Search `llms.txt` for page URLs

```bash
rg -n "firewall" llms.txt
```

Returns links like:
```
- [Filter](https://manual.mikrotik.com/docs/cli-reference/ip/firewall/filter.md): -----------
```

## Type Reference

Common RouterOS types found in `commands.toml`:

| Type | Meaning |
|------|---------|
| `string` | Free-form text |
| `num` | Integer number |
| `bool` | Boolean (yes/no, true/false) |
| `time` | Duration (e.g., `1d12h`, `30m`, `1h30m`) |
| `ipAddr` | IPv4 address |
| `ip6Addr` | IPv6 address |
| `ipAddr/prefix` | CIDR notation |
| `macAddr` | MAC address |
| `iface_enum` | Interface name (dropdown) |
| `enum (a \| b \| c)` | Enumerated choice |
| `multi { ... }` | Multi-select |
| `composite { , }` | Composite read-only value |
| `switch` | Boolean switch for filtering |
| `days` | Number of days |
| `{ }` suffix | Can be unset |

## Example Lookup

To find all properties of `/ip/firewall/filter`:

1. `rg -A 200 'path = "/ip/firewall/filter"' data/commands.toml`
2. For detailed descriptions: `rg -n "## /ip/firewall/filter" llms-full.txt` then read 200 lines after

## Rules

- **Never invent** RouterOS command names, property names, or types. Always verify.
- The `commands.toml` file is generated from `llms-full.txt` via `scripts/extract_commands.py`.
- If `commands.toml` is missing a property, check `llms-full.txt` directly.
- The grammar and commands are separate sources of truth — grammar defines syntax, commands define semantics.
