"""Tests for the llms-full.txt command extraction script."""
import sys
import os
import tempfile
from pathlib import Path

# Add scripts to path
sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))

from extract_commands import (
    should_include,
    escape_toml_string,
    clean_type,
    generate_toml,
)


class TestShouldInclude:
    """Tests for menu path filtering."""

    def test_ip_address(self):
        assert should_include("/ip/address") is True

    def test_ip_route(self):
        assert should_include("/ip/route") is True

    def test_ip_firewall_filter(self):
        assert should_include("/ip/firewall/filter") is True

    def test_ip_firewall_nat(self):
        assert should_include("/ip/firewall/nat") is True

    def test_ip_dhcp_server(self):
        assert should_include("/ip/dhcp-server") is True

    def test_ip_dns(self):
        assert should_include("/ip/dns") is True

    def test_ip_service(self):
        assert should_include("/ip/service") is True

    def test_ipv6_address(self):
        assert should_include("/ipv6/address") is True

    def test_ipv6_dhcp_client(self):
        assert should_include("/ipv6/dhcp-client") is True

    def test_ipv6_nd(self):
        assert should_include("/ipv6/nd") is True

    def test_ipv6_firewall(self):
        assert should_include("/ipv6/firewall/filter") is True

    def test_ipv6_route(self):
        assert should_include("/ipv6/route") is True

    def test_interface_bridge(self):
        assert should_include("/interface/bridge") is True

    def test_interface_vlan(self):
        assert should_include("/interface/vlan") is True

    def test_interface_pppoe_client(self):
        assert should_include("/interface/pppoe-client") is True

    def test_interface_ethernet(self):
        assert should_include("/interface/ethernet") is True

    def test_routing_ospf(self):
        assert should_include("/routing/ospf") is True

    def test_routing_bgp(self):
        assert should_include("/routing/bgp") is True

    def test_routing_table(self):
        assert should_include("/routing/table") is True

    def test_routing_rule(self):
        assert should_include("/routing/rule") is True

    # ── Excluded menus (still out of scope) ──────────────────

    def test_excluded_certificate(self):
        assert should_include("/certificate") is False

    # ── Now included under full extraction ────────────────────

    def test_now_included_system(self):
        """System is now included under full extraction."""
        assert should_include("/system/identity") is True

    def test_now_included_tool(self):
        """Tool is now included under full extraction."""
        assert should_include("/tool/ping") is True

    def test_now_included_user(self):
        """User is now included under full extraction."""
        assert should_include("/user") is True

    def test_now_included_queue(self):
        """Queue is now included under full extraction."""
        assert should_include("/queue/simple") is True

    def test_now_included_ip_arp(self):
        """ARP is now included under full /ip extraction."""
        assert should_include("/ip/arp") is True

    def test_now_included_ip_pool(self):
        """Pool is now included under full /ip extraction."""
        assert should_include("/ip/pool") is True

    # ── Edge cases ───────────────────────────────────────────

    def test_empty_path(self):
        assert should_include("") is False

    def test_root_only(self):
        """Root-only paths are included if the root is in TARGET_ROOTS."""
        assert should_include("/ip") is True
        assert should_include("/certificate") is False  # Not in TARGET_ROOTS

    def test_deeply_nested_firewall(self):
        assert should_include("/ip/firewall/filter/reset-counters") is True

    def test_deeply_nested_bridge(self):
        assert should_include("/interface/bridge/port/monitor") is True


class TestEscapeTomlString:
    """Tests for TOML string escaping."""

    def test_simple_string(self):
        assert escape_toml_string("hello") == "hello"

    def test_backslash(self):
        assert escape_toml_string("a\\b") == "a\\\\b"

    def test_quote(self):
        assert escape_toml_string('say "hi"') == 'say \\"hi\\"'

    def test_newline(self):
        assert escape_toml_string("line1\nline2") == "line1 line2"

    def test_carriage_return(self):
        assert escape_toml_string("a\rb") == "ab"  # \r is stripped

    def test_empty(self):
        assert escape_toml_string("") == ""


class TestCleanType:
    """Tests for type string cleaning."""

    def test_simple_type(self):
        assert clean_type("bool") == "bool"

    def test_multiline_type(self):
        result = clean_type("alt { ipAddr\n, string\n }")
        assert "alt" in result
        assert "ipAddr" in result

    def test_long_type(self):
        long_type = "x" * 200
        result = clean_type(long_type)
        assert len(result) <= 103  # 100 + "..."

    def test_enum_type(self):
        result = clean_type("enum (disabled | enabled | proxy-arp)")
        assert "enum" in result
        assert "disabled" in result


class TestGenerateToml:
    """Tests for TOML generation."""

    def test_empty_menus(self):
        result = generate_toml([])
        assert "# MikroTik" in result
        assert "Auto-generated" in result

    def test_single_menu(self):
        menus = [
            {
                "path": "/ip/address",
                "type": "Directory",
                "flags": [],
                "arguments": [
                    {
                        "name": "address",
                        "type": "composite",
                        "required": True,
                        "unset": False,
                        "description": "IP address",
                    }
                ],
                "read_only": [],
            }
        ]
        result = generate_toml(menus)
        assert "[[menus]]" in result
        assert 'path = "/ip/address"' in result
        assert "[[menus.arguments]]" in result
        assert 'name = "address"' in result

    def test_menu_with_flags(self):
        menus = [
            {
                "path": "/ip/route",
                "type": "Directory",
                "flags": [
                    {"name": "X", "description": "disabled", "required": False}
                ],
                "arguments": [],
                "read_only": [],
            }
        ]
        result = generate_toml(menus)
        assert "[[menus.flags]]" in result
        assert 'name = "X"' in result

    def test_menu_with_read_only(self):
        menus = [
            {
                "path": "/interface/bridge",
                "type": "Directory",
                "flags": [],
                "arguments": [],
                "read_only": [
                    {
                        "name": "mac-address",
                        "type": "macAddr",
                        "required": False,
                        "unset": False,
                        "description": "MAC address",
                    }
                ],
            }
        ]
        result = generate_toml(menus)
        assert "[[menus.read_only]]" in result
        assert 'name = "mac-address"' in result
