// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use serde::Deserialize;
use serde_xml_rs as xml;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;

// As per XKB official configuration guide, on Linux we use `evdev.xml`
#[cfg(target_os = "linux")]
const RULES_FILE: &str = "evdev.xml";
#[cfg(target_os = "linux")]
const X11_BASE_RULES: &str = "/usr/share/X11/xkb/rules/evdev.xml";
#[cfg(target_os = "linux")]
const X11_EXTRAS_RULES: &str = "/usr/share/X11/xkb/rules/evdev.extras.xml";
// when not on Linux, we use `base.xml`
#[cfg(not(target_os = "linux"))]
const RULES_FILE: &str = "base.xml";
#[cfg(not(target_os = "linux"))]
const X11_BASE_RULES: &str = "/usr/share/X11/xkb/rules/base.xml";
#[cfg(not(target_os = "linux"))]
const X11_EXTRAS_RULES: &str = "/usr/share/X11/xkb/rules/base.extras.xml";

/// A list of keyboard layouts parsed from X11_BASE_RULES.
#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardLayouts {
    #[serde(rename = "layoutList")]
    pub layout_list: LayoutList,
}

impl KeyboardLayouts {
    /// Fetch the layouts from the layout list.
    pub fn layouts(&self) -> &[KeyboardLayout] { &self.layout_list.layout }

    /// Fetch the layouts from the layout list.
    pub fn layouts_mut(&mut self) -> &mut [KeyboardLayout] { &mut self.layout_list.layout }
}

/// A list of keyboard layouts.
#[derive(Debug, Deserialize, Clone)]
pub struct LayoutList {
    pub layout: Vec<KeyboardLayout>,
}

/// A keyboard layout, which contains an optional list of variants, a name, and a description.
#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardLayout {
    #[serde(rename = "configItem")]
    pub config_item:  ConfigItem,
    #[serde(rename = "variantList")]
    pub variant_list: Option<VariantList>,
}

impl KeyboardLayout {
    /// Fetches the name of the keyboard layout.
    pub fn name(&self) -> &str { &self.config_item.name }

    /// Fetches a description of the layout.
    pub fn description(&self) -> &str { &self.config_item.description }

    /// Fetches a list of possible layout variants.
    pub fn variants(&self) -> Option<&Vec<KeyboardVariant>> {
        self.variant_list.as_ref().and_then(|x| x.variant.as_ref())
    }
}

/// Contains the name and description of a keyboard layout.
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigItem {
    pub name:              String,
    #[serde(rename = "shortDescription")]
    pub short_description: Option<String>,
    pub description:       String,
}

/// A list of possible variants of a keyboard layout.
#[derive(Debug, Deserialize, Clone)]
pub struct VariantList {
    pub variant: Option<Vec<KeyboardVariant>>,
}

/// A variant of a keyboard layout.
#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardVariant {
    #[serde(rename = "configItem")]
    pub config_item: ConfigItem,
}

impl KeyboardVariant {
    /// The name of this variant of a keybaord layout.
    pub fn name(&self) -> &str { &self.config_item.name }

    /// A description of this variant of a keyboard layout.
    pub fn description(&self) -> &str { &self.config_item.description }
}

/// Fetches a list of keyboard layouts from a path.
pub fn get_keyboard_layouts(path: &str) -> io::Result<KeyboardLayouts> {
    xml::from_reader(BufReader::new(File::open(path)?))
        .map_err(|why| io::Error::new(io::ErrorKind::InvalidData, format!("{}", why)))
}

/// Fetches a list of keyboard layouts from the X11_BASE_RULES const or the file defined in the X11_BASE_RULES_XML environment variable.
pub fn keyboard_layouts() -> io::Result<KeyboardLayouts> {
    if let Ok(x11_base_rules_xml) = std::env::var("X11_BASE_RULES_XML") {
        get_keyboard_layouts(&x11_base_rules_xml)
    }
    else {
        get_keyboard_layouts(X11_BASE_RULES)
    }
}

/// Fetches a list of keyboard layouts from the X11_EXTRAS_RULES const or the file defined in the X11_EXTRA_RULES_XML environment variable.
pub fn extra_keyboard_layouts() -> io::Result<KeyboardLayouts> {
    if let Ok(x11_extra_rules_xml) = std::env::var("X11_EXTRA_RULES_XML") {
        get_keyboard_layouts(&x11_extra_rules_xml)
    }
    else {
        get_keyboard_layouts(X11_EXTRAS_RULES)
    }
}

/// Fetches user-specific keyboard layouts from XDG config paths.
/// On Linux, looks for `evdev.xml`, on other systems looks for `base.xml`.
/// Returns all layouts found from these paths:
/// 1. `$XDG_CONFIG_HOME/xkb/rules/` (defaults to `$HOME/.config/xkb/rules/`)
/// 2. `$HOME/.xkb/rules/`
/// Returns empty layouts if no files are found.
pub fn user_keyboard_layouts() -> io::Result<KeyboardLayouts> {
    let paths = user_xkb_rules_paths();

    let layout_lists: Vec<LayoutList> = paths
        .into_iter()
        .filter_map(|path| get_keyboard_layouts(path.to_string_lossy().as_ref()).ok())
        .map(|layouts| layouts.layout_list)
        .collect();

    Ok(KeyboardLayouts {
        layout_list: concat_layout_lists(layout_lists),
    })
}

/// Returns the list of user XKB rules paths to check for a given rules file.
fn user_xkb_rules_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Defaults to $HOME/.config/xkb/rules/ if XDG_CONFIG_HOME is not set
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg_config_home);
        path.push("xkb/rules");
        path.push(RULES_FILE);
        if path.exists() {
            paths.push(path);
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config/xkb/rules");
        path.push(RULES_FILE);
        if path.exists() {
            paths.push(path);
        }
    }

    // Second priority: $HOME/.xkb/rules/
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".xkb/rules");
        path.push(RULES_FILE);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

/// Fetches a list of keyboard layouts from the system rules files and user config paths,
/// merging them together.
pub fn all_keyboard_layouts() -> io::Result<KeyboardLayouts> {
    let base_rules = keyboard_layouts()?;
    let extras_rules = extra_keyboard_layouts()?;
    let user_rules = user_keyboard_layouts().unwrap_or(KeyboardLayouts {
        layout_list: LayoutList { layout: vec![] },
    });

    let layout_lists = vec![
        base_rules.layout_list,
        extras_rules.layout_list,
        user_rules.layout_list,
    ];

    Ok(KeyboardLayouts {
        layout_list: concat_layout_lists(layout_lists),
    })
}

fn merge_rules(base: KeyboardLayouts, extras: KeyboardLayouts) -> KeyboardLayouts {
    KeyboardLayouts {
        layout_list: concat_layout_lists(vec![base.layout_list, extras.layout_list])
    }
}

fn concat_layout_lists(layouts: Vec<LayoutList>) -> LayoutList {
    let mut new_layouts = vec![];
    for layout_list in layouts.into_iter() {
        new_layouts.extend(layout_list.layout);
    }
    return LayoutList { layout: new_layouts }
}
