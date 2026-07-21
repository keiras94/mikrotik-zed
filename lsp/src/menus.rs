// ── Data structures and indices for the RSC language server ────────
//
// Loads commands.toml at compile time via include_str!() and builds
// all necessary lookup structures (path index, parent→children index,
// implicit root entries).

use serde::Deserialize;
use std::collections::HashMap;

// ── Embedded command table ────────────────────────────────────────

const COMMANDS_TOML: &str = include_str!("../../data/commands.toml");

// ── TOML data structures ──────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CommandsFile {
    #[serde(default)]
    pub(crate) menus: Vec<RawMenuEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawMenuEntry {
    path: String,
    #[serde(rename = "type", default)]
    menu_type: String,
    #[serde(default)]
    flags: Vec<RawArgEntry>,
    #[serde(default)]
    arguments: Vec<RawArgEntry>,
    #[serde(default)]
    read_only: Vec<RawArgEntry>,
}

#[derive(Debug, Deserialize)]
struct RawArgEntry {
    name: String,
    #[serde(rename = "type", default)]
    arg_type: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub path: String,
    pub menu_type: String,
    pub flags: Vec<ArgEntry>,
    pub arguments: Vec<ArgEntry>,
    pub read_only: Vec<ArgEntry>,
}

#[derive(Debug, Clone)]
pub struct ArgEntry {
    pub name: String,
    pub arg_type: String,
    pub description: String,
}

// ── Child entry (for populating implicit children) ────────────────

#[derive(Debug, Clone)]
pub struct ChildEntry {
    pub name: String,
    pub path: String,
    pub menu_type: String,
}

// ── Context (output of parse_line) ────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct LineContext {
    pub path: String,
    pub command: Option<String>,
    /// property name → value (empty string if just "key=")
    pub properties: HashMap<String, String>,
    pub last_token: String,
}

// ── Global state ──────────────────────────────────────────────────

pub struct MenuData {
    pub menus: Vec<MenuEntry>,
    pub menu_by_path: HashMap<String, MenuEntry>,
    pub child_names_by_parent: HashMap<String, Vec<ChildEntry>>,
}

impl MenuData {
    pub fn load() -> Self {
        let commands: CommandsFile =
            toml::from_str(COMMANDS_TOML).expect("failed to parse embedded commands.toml");

        let menus: Vec<MenuEntry> = commands
            .menus
            .into_iter()
            .map(|raw| MenuEntry {
                path: raw.path,
                menu_type: raw.menu_type,
                flags: raw.flags.into_iter().map(Into::into).collect(),
                arguments: raw.arguments.into_iter().map(Into::into).collect(),
                read_only: raw.read_only.into_iter().map(Into::into).collect(),
            })
            .collect();

        let mut menu_by_path: HashMap<String, MenuEntry> = HashMap::new();
        for m in &menus {
            menu_by_path.insert(m.path.clone(), m.clone());
        }

        // Build parent→children index from ALL paths
        let mut child_map: HashMap<String, HashMap<String, ChildEntry>> = HashMap::new();

        for m in &menus {
            let parts: Vec<&str> = m.path.split('/').collect();
            for i in 2..parts.len() {
                let parent_path = format!("/{}", parts[1..i].join("/"));
                let child_name = parts[i].to_string();
                let child_path = format!("/{}", parts[1..i + 1].join("/"));

                let entry = child_map.entry(parent_path).or_default();

                let child = entry.entry(child_name.clone()).or_insert_with(|| ChildEntry {
                    name: child_name,
                    path: child_path,
                    menu_type: m.menu_type.clone(),
                });
                if m.menu_type == "Directory" || m.menu_type == "Settings Directory" {
                    child.menu_type = m.menu_type.clone();
                }
            }
        }

        let mut root_children: HashMap<String, ChildEntry> = HashMap::new();
        for m in &menus {
            if let Some(root_name) = m.path.split('/').nth(1) {
                let root_name = root_name.to_string();
                root_children.entry(root_name.clone()).or_insert_with(|| ChildEntry {
                    name: root_name.clone(),
                    path: format!("/{root_name}"),
                    menu_type: "Directory".to_string(),
                });
            }
        }
        child_map.insert(String::new(), root_children);

        let child_names_by_parent: HashMap<String, Vec<ChildEntry>> = child_map
            .into_iter()
            .map(|(k, v)| (k, v.into_values().collect()))
            .collect();

        MenuData {
            menus,
            menu_by_path,
            child_names_by_parent,
        }
    }

    /// Standard RouterOS verbs available on most Directory-type menus
    pub const STANDARD_VERBS: &'static [&'static str] = &[
        "add", "remove", "set", "get", "print", "enable", "disable",
        "find", "comment", "move", "export", "import", "edit",
        "reset", "force-update",
    ];
}

// ── Conversions from raw (Deserialize) to clean types ────────────

impl From<RawArgEntry> for ArgEntry {
    fn from(raw: RawArgEntry) -> Self {
        ArgEntry {
            name: raw.name,
            arg_type: raw.arg_type,
            description: raw.description,
        }
    }
}
