#!/usr/bin/env python3
"""Debug version — trace what's happening with interface/bridge parsing."""
import re

with open("llms-full.txt", "r") as f:
    lines = f.readlines()

in_zone = False
for i, line in enumerate(lines[18840:18930], start=18840):
    stripped = line.rstrip()
    # Check for menu path
    m = re.match(r"^#{2,4}\s+(.+)", stripped)
    has_slash = "/" in stripped if m else False
    has_type = "**Type:**" in stripped
    has_argtable = "<ArgTable" in stripped
    has_row = "<ArgTableRow" in stripped
    end_table = "</ArgTable>" in stripped
    
    if m or has_type or has_argtable or end_table:
        print(f"L{i}: {stripped[:120]}")
    elif has_row:
        # Just count rows
        pass

print(f"\nTotal ArgTableRows between L18849 and L18930:")
count = sum(1 for l in lines[18848:18930] if "<ArgTableRow" in l)
print(f"  {count} rows")
