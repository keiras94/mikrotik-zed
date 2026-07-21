#!/usr/bin/env python3
"""
Extract RouterOS CLI command data from llms-full.txt and generate commands.toml.

Parses the CLI Reference section of llms-full.txt to extract menu paths,
argument names, types, descriptions, and flags for the 4 target root menus:

  /ip, /ipv6, /interface, /routing

Output: data/commands.toml (TOML format for the Zed language server)
"""

import re
import sys
from pathlib import Path


# Target root menus and their sub-menus (from AGENTS.md)
# Phase 2: full extraction for /interface, /ip, /ipv6
# /routing keeps scoped whitelist
TARGET_ROOTS = {
    "/ip": None,          # None = include ALL sub-menus recursively
    "/ipv6": None,        # None = include ALL sub-menus recursively
    "/interface": None,   # None = include ALL sub-menus recursively
    "/routing": {"ospf", "bgp", "table", "rule"},
}

# Firewall-specific sub-menus that need special path mapping
# The llms.txt file has firewall entries under both /ip and /ipv6
FIREWALL_PREFIXES = {
    "ip/firewall": "/ip/firewall",
    "ipv6/firewall": "/ipv6/firewall",
}


def should_include(menu_path: str) -> bool:
    """Check if a menu path belongs to one of the target root menus."""
    if not menu_path:
        return False

    # Normalize: strip trailing sub-command indicators like /monitor, /print, etc.
    parts = menu_path.strip("/").split("/")
    if len(parts) < 2:
        return False

    root = "/" + parts[0]
    if root not in TARGET_ROOTS:
        return False

    allowed = TARGET_ROOTS[root]

    # None means include ALL sub-menus under this root
    if allowed is None:
        return True

    # Otherwise, check if the first sub-menu is in the whitelist
    # Handle firewall specially: /ip/firewall/filter -> parts[1] = "firewall"
    first_sub = parts[1]

    if first_sub in allowed:
        return True

    # Special case: firewall has many sub-sub-menus (filter, nat, mangle, etc.)
    if "firewall" in menu_path.lower():
        return True

    return False


def parse_llms_full(filepath: str) -> list[dict]:
    """Parse llms-full.txt and extract menu entries."""
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    menus = []
    current_menu = None
    current_section = None  # "flags", "arguments", or "readonly"
    in_argtable = False
    argtable_c1 = None

    lines = content.split("\n")

    for i, line in enumerate(lines):
        # Detect menu path from ##, ###, or #### headings containing "/"
        # Skip headings that are just human-readable titles (no "/")
        menu_match = re.match(r"^#{2,4}\s+(.+)", line)
        if menu_match:
            path = menu_match.group(1).strip()
            # Clean up: remove trailing punctuation, markdown links
            path = re.sub(r"\[.*?\]\(.*?\)", "", path).strip()
            path = path.rstrip(".")

            # Only process menu paths (containing /) and not section headers
            # Filter out known non-CLI headings like "Scripting", "CLI Reference", etc.
            if "/" in path and not path.startswith("#"):
                # Save previous menu if it exists and should be included
                if current_menu and should_include(current_menu["path"]):
                    menus.append(current_menu)

                current_menu = {
                    "path": "/" + path,  # Add leading /
                    "type": "Directory",
                    "flags": [],
                    "arguments": [],
                    "read_only": [],
                }
                current_section = None
                in_argtable = False
            continue

        # Detect Type
        type_match = re.match(r"^\*\*Type:\*\*\s+(.+)", line)
        if type_match and current_menu:
            current_menu["type"] = type_match.group(1).strip()
            continue

        # Detect ArgTable end first (before start, since </ArgTable> also contains <ArgTable)
        if "</ArgTable>" in line:
            in_argtable = False
            current_section = None
            continue

        # Detect ArgTableRow (before ArgTable, since <ArgTableRow contains <ArgTable)
        if in_argtable and current_menu and "<ArgTableRow" in line:
            arg_match = re.search(r'arg="([^"]+)"', line)
            typ_match = re.search(r'typ="([^"]*)"', line)
            mandatory_match = re.search(r'mandatory="1"', line)
            unset_match = re.search(r'unset="1"', line)

            # Extract description (text between > and </ArgTableRow>)
            desc_match = re.search(r">([^<]*)</ArgTableRow", line)
            description = desc_match.group(1).strip() if desc_match else ""

            entry = {
                "name": arg_match.group(1) if arg_match else "",
                "type": typ_match.group(1) if typ_match else "",
                "required": bool(mandatory_match),
                "unset": bool(unset_match),
                "description": description,
            }

            if current_section == "flags":
                current_menu["flags"].append(entry)
            elif current_section == "arguments":
                current_menu["arguments"].append(entry)
            elif current_section == "readonly":
                current_menu["read_only"].append(entry)
            continue

        # Detect ArgTable start
        if "<ArgTable" in line:
            in_argtable = True
            c1_match = re.search(r'c1="([^"]+)"', line)
            if c1_match:
                argtable_c1 = c1_match.group(1)
                if argtable_c1 == "Flag":
                    current_section = "flags"
                elif argtable_c1 == "Argument":
                    current_section = "arguments"
                elif "Read-only" in argtable_c1:
                    current_section = "readonly"
            continue

        # Handle sub-menus that use #### heading (like #### ip/firewall/address-list)
        # These appear under ## headings and use ### or #### for the actual path
        sub_match = re.match(r"^#{2,4}\s+(.+)", line)
        if sub_match and current_menu:
            path = sub_match.group(1).strip()
            path = re.sub(r"\[.*?\]\(.*?\)", "", path).strip()
            path = path.rstrip(".")
            if "/" in path and path != current_menu["path"].lstrip("/"):
                # Save current menu and start new one
                if should_include(current_menu["path"]):
                    menus.append(current_menu)
                current_menu = {
                    "path": "/" + path,
                    "type": "Directory",
                    "flags": [],
                    "arguments": [],
                    "read_only": [],
                }
                current_section = None
                in_argtable = False
                continue

    # Save last menu
    if current_menu and should_include(current_menu["path"]):
        menus.append(current_menu)

    return menus


def clean_type(typ: str) -> str:
    """Clean and simplify type strings for the TOML output."""
    # Remove excessive whitespace
    typ = re.sub(r"\s+", " ", typ).strip()
    # Truncate very long type descriptions
    if len(typ) > 100:
        typ = typ[:97] + "..."
    return typ


def escape_toml_string(s: str) -> str:
    """Escape a string for TOML literal string representation."""
    # Replace backslashes and quotes
    s = s.replace("\\", "\\\\")
    s = s.replace('"', '\\"')
    # Remove newlines from descriptions
    s = s.replace("\n", " ").replace("\r", "")
    return s


def generate_toml(menus: list[dict]) -> str:
    """Generate TOML output from parsed menus."""
    lines = []
    lines.append(
        "# MikroTik RouterOS CLI Command Table"
    )
    lines.append(
        "# Auto-generated from llms-full.txt"
    )
    lines.append(
        "# Covers: /interface, /ip, /ipv6 (full), /routing (scoped)"
    )
    lines.append("")

    for menu in menus:
        path = menu["path"]
        menu_type = menu["type"]
        lines.append("[[menus]]")
        lines.append(f'path = "{escape_toml_string(path)}"')
        lines.append(f'type = "{escape_toml_string(menu_type)}"')

        # Flags
        if menu["flags"]:
            for flag in menu["flags"]:
                name = escape_toml_string(flag["name"])
                desc = escape_toml_string(flag["description"])
                lines.append("[[menus.flags]]")
                lines.append(f'name = "{name}"')
                lines.append(f'description = "{desc}"')
                if flag.get("required"):
                    lines.append("required = true")

        # Arguments
        if menu["arguments"]:
            for arg in menu["arguments"]:
                name = escape_toml_string(arg["name"])
                typ = clean_type(arg["type"])
                desc = escape_toml_string(arg["description"])
                lines.append("[[menus.arguments]]")
                lines.append(f'name = "{name}"')
                lines.append(f'type = "{typ}"')
                if desc:
                    lines.append(f'description = "{desc}"')
                if arg.get("required"):
                    lines.append("required = true")
                if arg.get("unset"):
                    lines.append("unset = true")

        # Read-only arguments
        if menu["read_only"]:
            for arg in menu["read_only"]:
                name = escape_toml_string(arg["name"])
                typ = clean_type(arg["type"])
                desc = escape_toml_string(arg["description"])
                lines.append("[[menus.read_only]]")
                lines.append(f'name = "{name}"')
                lines.append(f'type = "{typ}"')
                if desc:
                    lines.append(f'description = "{desc}"')

        lines.append("")

    return "\n".join(lines)


def main():
    script_dir = Path(__file__).parent
    project_root = script_dir.parent if script_dir.name == "scripts" else script_dir
    input_file = project_root / "llms-full.txt"
    output_file = project_root / "data" / "commands.toml"

    if not input_file.exists():
        print(f"Error: {input_file} not found", file=sys.stderr)
        sys.exit(1)

    print(f"Parsing {input_file}...")
    menus = parse_llms_full(str(input_file))

    # Filter to target menus
    filtered = [m for m in menus if should_include(m["path"])]
    print(f"Found {len(menus)} total entries, {len(filtered)} match target menus.")

    # Deduplicate by path
    seen = set()
    unique = []
    for m in filtered:
        if m["path"] not in seen:
            seen.add(m["path"])
            unique.append(m)

    # Sort by path
    unique.sort(key=lambda m: m["path"])

    toml_content = generate_toml(unique)

    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(toml_content, encoding="utf-8")
    print(f"Wrote {output_file} ({len(unique)} menus)")


if __name__ == "__main__":
    main()
