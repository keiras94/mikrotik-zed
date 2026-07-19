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
    #[serde(rename = "type")]
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
