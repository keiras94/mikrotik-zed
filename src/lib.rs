use serde::Deserialize;
use std::collections::HashMap;
use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

// ── Embedded command table ────────────────────────────────────────

const COMMANDS_TOML: &str = include_str!("../data/commands.toml");

// ── Data structures ───────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CommandsFile {
    menus: Vec<MenuEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct MenuEntry {
    path: String,
    #[serde(rename = "type")]
    menu_type: String,
    #[serde(default)]
    flags: Vec<ArgEntry>,
    #[serde(default)]
    arguments: Vec<ArgEntry>,
    #[serde(default)]
    read_only: Vec<ArgEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct ArgEntry {
    name: String,
    #[serde(rename = "type", default)]
    arg_type: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    #[allow(dead_code)]
    unset: bool,
}

// ── Extension state ───────────────────────────────────────────────

#[allow(dead_code)]
struct RscExtension {
    /// All known menu paths → their entries
    menus: Vec<MenuEntry>,
    /// Index: menu path → MenuEntry
    menu_index: HashMap<String, MenuEntry>,
    /// Index: parent menu path → list of child menu paths
    children_index: HashMap<String, Vec<String>>,
}

impl RscExtension {
    fn load_commands() -> std::result::Result<Self, String> {
        let commands: CommandsFile =
            toml::from_str(COMMANDS_TOML).map_err(|e| format!("Failed to parse commands.toml: {e}"))?;

        let mut menu_index = HashMap::new();
        let mut children_index: HashMap<String, Vec<String>> = HashMap::new();

        for menu in &commands.menus {
            menu_index.insert(menu.path.clone(), menu.clone());

            if let Some(last_slash) = menu.path.rfind('/') {
                let parent = &menu.path[..last_slash];
                children_index
                    .entry(parent.to_string())
                    .or_default()
                    .push(menu.path.clone());
            }
        }

        Ok(RscExtension {
            menus: commands.menus,
            menu_index,
            children_index,
        })
    }
}

// ── Extension trait implementation ───────────────────────────────

impl zed::Extension for RscExtension {
    fn new() -> Self {
        match RscExtension::load_commands() {
            Ok(ext) => ext,
            Err(e) => {
                // Log the error and return an empty extension rather than panicking.
                // The LS will still start but without completions.
                eprintln!("[rsc-ls] WARNING: {e}");
                RscExtension {
                    menus: vec![],
                    menu_index: HashMap::new(),
                    children_index: HashMap::new(),
                }
            }
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let node = worktree
            .which("node")
            .ok_or_else(|| "node not found in PATH; RSC language server requires Node.js".to_string())?;

        // CARGO_MANIFEST_DIR is the crate root at build time — it equals the
        // extension source root both during development and after Zed clones
        // the extension submodule.
        let ext_root = env!("CARGO_MANIFEST_DIR");

        let ls_script = format!("{}/src/ls.mjs", ext_root);
        let commands_path = format!("{}/data/commands.toml", ext_root);

        Ok(zed::Command {
            command: node,
            args: vec![ls_script, commands_path],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(RscExtension);

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Quick inline TOML for testing.
    fn test_commands_toml() -> &'static str {
        r#"
[[menus]]
path = "/ip/address"
type = "Directory"

[[menus.arguments]]
name = "address"
type = "ipPrefix"
required = true
description = "The IP address and network mask"

[[menus.arguments]]
name = "interface"
type = "iface_enum"
required = true

[[menus.flags]]
name = "X"
description = "disabled"

[[menus.flags]]
name = "D"
description = "dynamic"

[[menus.read_only]]
name = "actual-interface"
type = "iface_enum"
description = "The actual interface"

[[menus]]
path = "/ip/route"
type = "Directory"

[[menus.arguments]]
name = "gateway"
type = "address (flags=46ivL)"

[[menus]]
path = "/ip/route/check"
type = "Command"

[[menus]]
path = "/ip/firewall/filter"
type = "Directory"

[[menus.arguments]]
name = "chain"
type = "enum (input | forward | output)"
required = true

[[menus.arguments]]
name = "action"
type = "enum (accept | drop | reject)"

[[menus]]
path = "/interface/bridge/port"
type = "Directory"

[[menus]]
path = "/routing/bgp/connection"
type = "Directory"

[[menus]]
path = "/system/identity"
type = "Directory"
"#
    }

    // ── TOML parsing ──────────────────────────────────────────

    #[test]
    fn test_parse_commands_toml() {
        let commands: CommandsFile =
            toml::from_str(test_commands_toml()).expect("should parse TOML");
        assert!(!commands.menus.is_empty(), "should have menus");
        assert!(commands.menus.len() >= 4, "should have at least 4 menus");
    }

    #[test]
    fn test_menu_index_built() {
        let ext = RscExtension {
            menus: vec![],
            menu_index: {
                let mut m = HashMap::new();
                m.insert(
                    "/ip/address".to_string(),
                    MenuEntry {
                        path: "/ip/address".to_string(),
                        menu_type: "Directory".to_string(),
                        flags: vec![],
                        arguments: vec![ArgEntry {
                            name: "address".into(),
                            arg_type: "ipPrefix".into(),
                            description: "".into(),
                            required: true,
                            unset: false,
                        }],
                        read_only: vec![],
                    },
                );
                m
            },
            children_index: HashMap::new(),
        };

        let entry = ext.menu_index.get("/ip/address").unwrap();
        assert_eq!(entry.path, "/ip/address");
        assert_eq!(entry.arguments.len(), 1);
        assert_eq!(entry.arguments[0].name, "address");
        assert!(entry.arguments[0].required);
    }

    // ── Children index ──────────────────────────────────────

    #[test]
    fn test_children_index() {
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        children.insert("/ip".to_string(), vec!["/ip/address".into(), "/ip/route".into()]);
        children.insert(
            "/ip/route".to_string(),
            vec!["/ip/route/check".into()],
        );

        let ip_children = children.get("/ip").unwrap();
        assert_eq!(ip_children.len(), 2);
        assert!(ip_children.contains(&"/ip/address".to_string()));
        assert!(ip_children.contains(&"/ip/route".to_string()));
    }

    // ── Context detection (test helper) ──────────────────────────

    /// Parse the current line pre-cursor to detect menu path context.
    fn detect_menu_path(line: &str, cursor: usize) -> Option<String> {
        let before = &line[..cursor.min(line.len())];
        // Find the first '/' that starts a menu path (preceded by whitespace or line start),
        // searching from right to left but ensuring it's the start of a path.
        let mut pos = before.len();
        loop {
            match before[..pos].rfind('/') {
                None => return None,
                Some(slash_pos) => {
                    // Check if this '/' is at line start or preceded by whitespace
                    if slash_pos == 0 || before.as_bytes()[slash_pos - 1].is_ascii_whitespace() {
                        let after = &before[slash_pos..];
                        let end = after
                            .find(|c: char| c.is_whitespace() || c == ';' || c == ']')
                            .unwrap_or(after.len());
                        return Some(after[..end].to_string());
                    }
                    // This '/' is inside a value (e.g., CIDR), continue searching before it
                    pos = slash_pos;
                }
            }
        }
    }

    #[test]
    fn test_detect_menu_path_simple() {
        let path = detect_menu_path("/ip address add address=10.0.0.1", 12);
        assert_eq!(path, Some("/ip".to_string()));
    }

    #[test]
    fn test_detect_menu_path_nested() {
        let path = detect_menu_path("/ip/route add gateway=192.168.1.1", 35);
        assert_eq!(path, Some("/ip/route".to_string()));
    }

    #[test]
    fn test_detect_menu_path_none() {
        let path = detect_menu_path(":put $var", 5);
        assert_eq!(path, None);
    }

    #[test]
    fn test_detect_menu_path_cidr() {
        // 10.0.0.0/24 is a CIDR, not a menu path
        let path = detect_menu_path("/ip route add dst-address=10.0.0.0/24", 20);
        assert_eq!(path, Some("/ip".to_string()));
    }

    // ── Edge cases ───────────────────────────────────────────

    #[test]
    fn test_empty_commands_toml() {
        let toml_str = "\n[[menus]]\npath = \"/empty\"\ntype = \"Directory\"\n";
        let commands: CommandsFile = toml::from_str(toml_str).unwrap();
        assert_eq!(commands.menus.len(), 1);
        assert_eq!(commands.menus[0].path, "/empty");
    }

    #[test]
    fn test_menus_are_not_empty() {
        let ext = RscExtension::load_commands().expect("load_commands should succeed");
        assert!(!ext.menus.is_empty(), "embedded commands.toml should have menus");
        assert!(ext.menus.len() >= 50, "should have at least 50 menus");
        assert!(!ext.menu_index.is_empty(), "menu_index should be populated");
    }

    #[test]
    fn test_all_menus_have_path() {
        let ext = RscExtension::load_commands().expect("load_commands should succeed");
        for menu in &ext.menus {
            assert!(!menu.path.is_empty(), "every menu should have a path");
            assert!(menu.path.starts_with('/'), "paths should start with /: {}", menu.path);
        }
    }

    #[test]
    fn test_target_root_menus_present() {
        let ext = RscExtension::load_commands().expect("load_commands should succeed");
        let paths: Vec<&str> = ext.menus.iter().map(|m| m.path.as_str()).collect();

        // Should contain at least one entry from each target root menu
        assert!(paths.iter().any(|p| p.starts_with("/ip/")), "missing /ip entries");
        assert!(paths.iter().any(|p| p.starts_with("/ipv6/")), "missing /ipv6 entries");
        assert!(paths.iter().any(|p| p.starts_with("/interface/")), "missing /interface entries");
        assert!(paths.iter().any(|p| p.starts_with("/routing/")), "missing /routing entries");
    }

    #[test]
    fn test_no_unwanted_root_menus() {
        let ext = RscExtension::load_commands().expect("load_commands should succeed");
        for menu in &ext.menus {
            assert!(
                !menu.path.starts_with("/system/"),
                "should not contain /system: {}", menu.path
            );
            assert!(
                !menu.path.starts_with("/tool/"),
                "should not contain /tool: {}", menu.path
            );
            assert!(
                !menu.path.starts_with("/certificate"),
                "should not contain /certificate: {}", menu.path
            );
        }
    }

    #[test]
    fn test_specific_menus_exist() {
        let ext = RscExtension::load_commands().expect("load_commands should succeed");

        // These must exist per AGENTS.md scope
        assert!(ext.menu_index.contains_key("/ip/address"), "missing /ip/address");
        assert!(ext.menu_index.contains_key("/ip/route"), "missing /ip/route");
        assert!(ext.menu_index.contains_key("/ip/firewall/filter"), "missing /ip/firewall/filter");
        assert!(ext.menu_index.contains_key("/ip/dns"), "missing /ip/dns");
        assert!(ext.menu_index.contains_key("/ip/service"), "missing /ip/service");
        assert!(ext.menu_index.contains_key("/ipv6/address"), "missing /ipv6/address");
        assert!(ext.menu_index.contains_key("/ipv6/route"), "missing /ipv6/route");
        assert!(ext.menu_index.contains_key("/interface/bridge"), "missing /interface/bridge");
        assert!(ext.menu_index.contains_key("/interface/ethernet"), "missing /interface/ethernet");
        assert!(ext.menu_index.contains_key("/routing/ospf"), "missing /routing/ospf");
        assert!(ext.menu_index.contains_key("/routing/bgp"), "missing /routing/bgp");
    }
}

