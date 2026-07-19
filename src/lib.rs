use serde::Deserialize;
use std::collections::HashMap;
use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

// ── Embedded command table ────────────────────────────────────────

const COMMANDS_TOML: &str = include_str!("../data/commands.toml");

// ── Data structures ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CommandsFile {
    menus: Vec<MenuEntry>,
}

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

struct RscExtension {
    /// All known menu paths → their entries
    menus: Vec<MenuEntry>,
    /// Index: menu path → MenuEntry
    menu_index: HashMap<String, MenuEntry>,
    /// Index: parent menu path → list of child menu paths
    children_index: HashMap<String, Vec<String>>,
}

impl RscExtension {
    fn load_commands() -> Self {
        let commands: CommandsFile =
            toml::from_str(COMMANDS_TOML).expect("Failed to parse embedded commands.toml");

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

        RscExtension {
            menus: commands.menus,
            menu_index,
            children_index,
        }
    }

    /// Build a flat list of all known completions keyed by menu path.
    /// Each key is a menu path, value is a list of (label, detail) pairs.
    fn build_completion_data(&self) -> serde_json::Value {
        let mut data = serde_json::Map::new();

        // Root-level completions (root menus)
        let roots = vec![
            ("/ip", "IP menu — addresses, routes, firewall, DHCP, DNS"),
            ("/ipv6", "IPv6 menu — addresses, DHCPv6, ND, firewall, routes"),
            ("/interface", "Interface menu — bridge, VLAN, PPPoE, ethernet"),
            ("/routing", "Routing menu — OSPF, BGP, tables, rules"),
        ];
        data.insert(
            "__root__".to_string(),
            serde_json::to_value(
                roots
                    .iter()
                    .map(|(path, desc)| {
                        serde_json::json!({"label": path, "detail": desc})
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        );

        // Common commands available at any menu level
        let common_commands: Vec<serde_json::Value> = vec![
            "add", "remove", "enable", "disable", "set", "get",
            "print", "find", "export", "edit", "comment", "move", "reset",
        ]
        .into_iter()
        .map(|cmd| {
            serde_json::json!({"label": cmd, "detail": format!("{} — common RouterOS command", cmd)})
        })
        .collect();

        // Build completions per menu
        for menu in &self.menus {
            let path = &menu.path;
            let mut items: Vec<serde_json::Value> = Vec::new();

            // Add common commands
            items.extend(common_commands.clone());

            // Add sub-menus (children)
            if let Some(children) = self.children_index.get(path) {
                for child_path in children {
                    let name = child_path.rsplit('/').next().unwrap_or(child_path);
                    if let Some(child) = self.menu_index.get(child_path) {
                        items.push(serde_json::json!({
                            "label": name,
                            "detail": format!("{} — {}", child_path, child.menu_type),
                            "kind": 19, // Module/namespace kind
                        }));
                    }
                }
            }

            // Add writable arguments
            for arg in &menu.arguments {
                let mut detail = format!("{} — type: {}", arg.name, arg.arg_type);
                if arg.required {
                    detail.push_str(" [required]");
                }
                if !arg.description.is_empty() {
                    detail.push_str(&format!("\n{}", arg.description));
                }
                items.push(serde_json::json!({
                    "label": arg.name,
                    "detail": detail,
                    "kind": 10, // Property kind
                    "insertText": format!("{}=", arg.name),
                }));
            }

            data.insert(
                path.clone(),
                serde_json::json!(items),
            );
        }

        serde_json::Value::Object(data)
    }

    /// Build hover data: menu → property → description
    fn build_hover_data(&self) -> serde_json::Value {
        let mut data = serde_json::Map::new();

        for menu in &self.menus {
            let path = &menu.path;
            let mut props = serde_json::Map::new();

            for arg in &menu.arguments {
                let mut value = format!("**{}**\n\nType: `{}`", arg.name, arg.arg_type);
                if arg.required {
                    value.push_str("\n\n*Required*");
                }
                if !arg.description.is_empty() {
                    value.push_str(&format!("\n\n{}", arg.description));
                }
                props.insert(arg.name.clone(), serde_json::Value::String(value));
            }

            for flag in &menu.flags {
                let value = format!(
                    "**{}** (flag)\n\nDescription: {}",
                    flag.name,
                    if flag.description.is_empty() {
                        flag.arg_type.as_str()
                    } else {
                        flag.description.as_str()
                    }
                );
                props.insert(flag.name.clone(), serde_json::Value::String(value));
            }

            for ro in &menu.read_only {
                let mut value = format!(
                    "**{}** (read-only)\n\nType: `{}`",
                    ro.name, ro.arg_type
                );
                if !ro.description.is_empty() {
                    value.push_str(&format!("\n\n{}", ro.description));
                }
                props.insert(ro.name.clone(), serde_json::Value::String(value));
            }

            if !props.is_empty() {
                data.insert(path.clone(), serde_json::Value::Object(props));
            }
        }

        serde_json::Value::Object(data)
    }
}

// ── Extension trait implementation ───────────────────────────────

impl zed::Extension for RscExtension {
    fn new() -> Self {
        RscExtension::load_commands()
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let completions = self.build_completion_data();
        let hovers = self.build_hover_data();

        Ok(Some(serde_json::json!({
            "rsc": {
                "completions": completions,
                "hoverData": hovers,
            }
        })))
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

    // ── Completions ──────────────────────────────────────────

    #[test]
    fn test_build_completion_data_has_root() {
        let ext = RscExtension {
            menus: vec![],
            menu_index: HashMap::new(),
            children_index: HashMap::new(),
        };
        let data = ext.build_completion_data();
        assert!(data.get("__root__").is_some(), "should have root completions");
    }

    #[test]
    fn test_build_completion_data_has_menus() {
        let commands: CommandsFile =
            toml::from_str(test_commands_toml()).unwrap();
        let mut menu_index = HashMap::new();
        for menu in &commands.menus {
            menu_index.insert(menu.path.clone(), menu.clone());
        }

        let mut children_index: HashMap<String, Vec<String>> = HashMap::new();
        children_index.insert(
            "/ip".to_string(),
            vec!["/ip/address".into(), "/ip/route".into()],
        );

        let ext = RscExtension {
            menus: commands.menus,
            menu_index,
            children_index,
        };

        let data = ext.build_completion_data();
        // Each menu path should have completion items
        assert!(data.get("/ip/address").is_some());
        assert!(data.get("/ip/route").is_some());
        assert!(data.get("/ip/firewall/filter").is_some());
    }

    // ── Hover data ───────────────────────────────────────────

    #[test]
    fn test_build_hover_data() {
        let commands: CommandsFile =
            toml::from_str(test_commands_toml()).unwrap();
        let mut menu_index = HashMap::new();
        for menu in &commands.menus {
            menu_index.insert(menu.path.clone(), menu.clone());
        }

        let ext = RscExtension {
            menus: commands.menus,
            menu_index,
            children_index: HashMap::new(),
        };

        let hover = ext.build_hover_data();
        // /ip/address should have hover data for its arguments
        assert!(hover.get("/ip/address").is_some());
        let addr_hover = &hover["/ip/address"];
        assert!(addr_hover.get("address").is_some());
        assert!(addr_hover.get("interface").is_some());
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
        let ext = RscExtension::load_commands();
        assert!(!ext.menus.is_empty(), "embedded commands.toml should have menus");
        assert!(ext.menus.len() >= 50, "should have at least 50 menus");
        assert!(!ext.menu_index.is_empty(), "menu_index should be populated");
    }

    #[test]
    fn test_all_menus_have_path() {
        let ext = RscExtension::load_commands();
        for menu in &ext.menus {
            assert!(!menu.path.is_empty(), "every menu should have a path");
            assert!(menu.path.starts_with('/'), "paths should start with /: {}", menu.path);
        }
    }

    #[test]
    fn test_target_root_menus_present() {
        let ext = RscExtension::load_commands();
        let paths: Vec<&str> = ext.menus.iter().map(|m| m.path.as_str()).collect();

        // Should contain at least one entry from each target root menu
        assert!(paths.iter().any(|p| p.starts_with("/ip/")), "missing /ip entries");
        assert!(paths.iter().any(|p| p.starts_with("/ipv6/")), "missing /ipv6 entries");
        assert!(paths.iter().any(|p| p.starts_with("/interface/")), "missing /interface entries");
        assert!(paths.iter().any(|p| p.starts_with("/routing/")), "missing /routing entries");
    }

    #[test]
    fn test_no_unwanted_root_menus() {
        let ext = RscExtension::load_commands();
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
        let ext = RscExtension::load_commands();

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

