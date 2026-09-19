//! The culling keymap: each action's default keys and the user's overrides
//! from the `shortcuts` settings key, merged so no key is bound twice. The
//! colour label actions' defaults follow the selected sidecar format.

use serde::Serialize;
use serde_json::{Map, Value};

use crate::sidecar::SidecarFormat;

/// Every action in the order the shortcuts panel shows them, with its
/// default keys under either format. A plain key is `event.key` lower-cased,
/// with `" "` as `"space"`. With a modifier held, the name is
/// `ctrl+alt+shift+meta+` (only the modifiers held, in that order) and the key
/// named from `event.code` (`Digit1` -> `1`, `Comma` -> `,`), as Option and
/// Shift change `event.key`.
const DEFAULTS: &[(&str, &[&str])] = &[
    ("previous", &["arrowleft", "arrowup", "w", "a", "h", "k"]),
    ("next", &["arrowright", "arrowdown", "s", "d", "j", "l"]),
    ("open", &["o"]),
    ("focus", &["f"]),
    ("zoom", &["space"]),
    ("rate1", &["1"]),
    ("rate2", &["2"]),
    ("rate3", &["3"]),
    ("rate4", &["4"]),
    ("rate5", &["5"]),
    ("reject", &["x"]),
    ("pick", &["p"]),
    ("unflag", &["u"]),
    ("clear", &["0"]),
];

/// The colour label actions, after `DEFAULTS`, with their default keys under
/// XMP (Lightroom's) and under `.dop` (PhotoLab's).
const LABEL_DEFAULTS: &[(&str, &[&str], &[&str])] = &[
    ("red", &["6"], &["ctrl+alt+1"]),
    ("orange", &[], &["ctrl+alt+2"]),
    ("yellow", &["7"], &["ctrl+alt+3"]),
    ("green", &["8"], &["ctrl+alt+4"]),
    ("blue", &["9"], &["ctrl+alt+5"]),
    ("pink", &[], &["ctrl+alt+6"]),
    ("purple", &["-"], &["ctrl+alt+7"]),
    ("clearlabel", &[], &["ctrl+alt+0"]),
];

fn default_keys(
    format: SidecarFormat,
) -> impl Iterator<Item = (&'static str, &'static [&'static str])> {
    DEFAULTS.iter().copied().chain(LABEL_DEFAULTS.iter().map(
        move |&(action, xmp, dop)| match format {
            SidecarFormat::Xmp => (action, xmp),
            SidecarFormat::Dop => (action, dop),
        },
    ))
}

/// The key reserved for `pick`.
const PICK_KEY: &str = "p";

/// macOS combinations owned by the app's menu (`app_menu::build` on top of
/// `Menu::default`).
const MACOS_MENU: &[&str] = &[
    "meta+,",
    "meta+z",
    "meta+q",
    "meta+h",
    "alt+meta+h",
    "meta+w",
    "meta+m",
    "ctrl+meta+f",
    "meta+x",
    "meta+c",
    "meta+v",
    "meta+a",
];

/// macOS combinations owned by the OS: app switcher, Spotlight, input
/// sources, window cycling, Force Quit, lock and log out, screenshots, Dock
/// and Mission Control.
const MACOS_SYSTEM: &[&str] = &[
    "meta+tab",
    "shift+meta+tab",
    "meta+space",
    "alt+meta+space",
    "ctrl+space",
    "meta+`",
    "shift+meta+`",
    "alt+meta+escape",
    "ctrl+meta+q",
    "shift+meta+q",
    "shift+meta+3",
    "shift+meta+4",
    "shift+meta+5",
    "alt+meta+d",
    "ctrl+meta+space",
    "ctrl+arrowleft",
    "ctrl+arrowright",
    "ctrl+arrowup",
    "ctrl+arrowdown",
];

/// Windows / Linux combinations owned by the app's menu.
const OTHER_MENU: &[&str] = &[
    "ctrl+,", "ctrl+z", "ctrl+x", "ctrl+c", "ctrl+v", "ctrl+a", "ctrl+m", "alt+f4",
];

/// Windows / Linux combinations owned by the OS; every `meta+` name is too,
/// as the shell owns the Windows / Super key.
const OTHER_SYSTEM: &[&str] = &[
    "alt+tab",
    "shift+alt+tab",
    "ctrl+escape",
    "ctrl+shift+escape",
];

/// Why `key` cannot be bound on macOS or elsewhere, if it cannot.
fn forbidden(key: &str, macos: bool) -> Option<&'static str> {
    let (menu, system) = if macos {
        (MACOS_MENU, MACOS_SYSTEM)
    } else {
        (OTHER_MENU, OTHER_SYSTEM)
    };
    if menu.contains(&key) {
        Some("is a menu accelerator")
    } else if system.contains(&key) || (!macos && key.contains("meta+")) {
        Some("is reserved by the system")
    } else {
        None
    }
}

const MACOS: bool = cfg!(target_os = "macos");

/// One action and the keys bound to it, as the `shortcuts` command returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Binding {
    pub action: &'static str,
    pub keys: Vec<String>,
}

/// The resolved keymap for one sidecar format, one entry per action in
/// `DEFAULTS` then `LABEL_DEFAULTS` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    format: SidecarFormat,
    bindings: Vec<Binding>,
    /// Stored overrides skipped for a key conflict under `format`, kept so
    /// saving does not drop them and another format can still apply them.
    inactive: Map<String, Value>,
}

impl Keymap {
    /// The default keys under `format`.
    pub fn defaults(format: SidecarFormat) -> Keymap {
        Keymap {
            format,
            bindings: default_keys(format)
                .map(|(action, keys)| Binding {
                    action,
                    keys: keys.iter().map(|k| k.to_string()).collect(),
                })
                .collect(),
            inactive: Map::new(),
        }
    }

    /// Merge the stored `shortcuts` value over the defaults. Every entry that
    /// cannot be used is logged and the action keeps its default; an override
    /// whose key is already bound to another action is skipped, so the
    /// earlier action wins. Conflicts are checked against `format`'s
    /// defaults, so one override can apply under one format only; a
    /// conflicting override is kept in the stored value (`overrides`).
    pub fn from_overrides(overrides: Option<&Value>, format: SidecarFormat) -> Keymap {
        let mut keymap = Keymap::defaults(format);
        let Some(overrides) = overrides else {
            return keymap;
        };
        let Some(overrides) = overrides.as_object() else {
            log::warn!("ignoring the shortcuts setting: not an object");
            return keymap;
        };
        for name in overrides.keys() {
            if !keymap.bindings.iter().any(|b| b.action == name) {
                log::warn!("ignoring the shortcut for an unknown action {name:?}");
            }
        }
        for i in 0..keymap.bindings.len() {
            let action = keymap.bindings[i].action;
            let Some(value) = overrides.get(action) else {
                continue;
            };
            let Some(keys) = parse_keys(value) else {
                log::warn!("ignoring the shortcut for {action}: not a non-empty list of keys");
                continue;
            };
            if action != "pick" && keys.iter().any(|k| k == PICK_KEY) {
                log::warn!("ignoring the shortcut for {action}: \"p\" is reserved for pick");
                continue;
            }
            if let Some((key, reason)) = keys
                .iter()
                .find_map(|k| forbidden(k, MACOS).map(|r| (k, r)))
            {
                log::warn!("ignoring the shortcut for {action}: {key:?} {reason}");
                continue;
            }
            let conflict = keys.iter().find_map(|key| {
                keymap
                    .bindings
                    .iter()
                    .find(|b| b.action != action && b.keys.contains(key))
                    .map(|b| (key, b.action))
            });
            if let Some((key, other)) = conflict {
                log::warn!("ignoring the shortcut for {action}: {key:?} is bound to {other}");
                keymap.inactive.insert(action.to_string(), value.clone());
                continue;
            }
            keymap.bindings[i].keys = keys;
        }
        keymap
    }

    /// The bindings in the order the shortcuts panel shows them.
    pub fn bindings(&self) -> Vec<Binding> {
        self.bindings.clone()
    }

    /// Add `key` to `action`'s keys, or explain why it cannot be: `pick` is
    /// not editable, `p` is reserved for pick, a key bound to another action
    /// is refused rather than moved, and a key the action holds is an error.
    pub fn add(&mut self, action: &str, key: &str) -> Result<(), String> {
        let i = self.check_bindable(action, key)?;
        if self.bindings[i].keys.iter().any(|k| k == key) {
            return Err(format!("{key:?} is already bound to {action}"));
        }
        self.bindings[i].keys.push(key.to_string());
        self.inactive.remove(action);
        Ok(())
    }

    /// Remove `key` from `action`'s keys. `pick` is not editable, and the
    /// last key cannot be removed, as an empty list does not load back.
    pub fn remove(&mut self, action: &str, key: &str) -> Result<(), String> {
        let i = self.index(action)?;
        if action == "pick" {
            return Err("pick is not editable".to_string());
        }
        let keys = &mut self.bindings[i].keys;
        let Some(at) = keys.iter().position(|k| k == key) else {
            return Err(format!("{key:?} is not bound to {action}"));
        };
        if keys.len() == 1 {
            return Err(format!("{key:?} is the only key for {action}; use Reset"));
        }
        keys.remove(at);
        self.inactive.remove(action);
        Ok(())
    }

    fn check_bindable(&self, action: &str, key: &str) -> Result<usize, String> {
        let i = self.index(action)?;
        if action == "pick" {
            return Err("pick is not editable".to_string());
        }
        if key == PICK_KEY {
            return Err("\"p\" is reserved for pick".to_string());
        }
        if let Some(reason) = forbidden(key, MACOS) {
            return Err(format!("{key:?} {reason}"));
        }
        if let Some(other) = self
            .bindings
            .iter()
            .find(|b| b.action != action && b.keys.iter().any(|k| k == key))
        {
            return Err(format!("{key:?} is bound to {}", other.action));
        }
        Ok(i)
    }

    /// Restore `action`'s default keys, or fail if one of them has since been
    /// bound to another action.
    pub fn reset(&mut self, action: &str) -> Result<(), String> {
        let i = self.index(action)?;
        let defaults = Keymap::defaults(self.format).bindings.swap_remove(i).keys;
        if let Some((key, other)) = defaults.iter().find_map(|key| {
            self.bindings
                .iter()
                .find(|b| b.action != action && b.keys.contains(key))
                .map(|b| (key, b.action))
        }) {
            return Err(format!("{key:?} is bound to {other}"));
        }
        self.bindings[i].keys = defaults;
        self.inactive.remove(action);
        Ok(())
    }

    /// Restore every action's default keys.
    pub fn reset_all(&mut self) {
        *self = Keymap::defaults(self.format);
    }

    /// The actions whose keys differ from the default, as stored under the
    /// `shortcuts` settings key. Compared with the current format's defaults;
    /// a stored override skipped for a key conflict is kept as it was.
    pub fn overrides(&self) -> Value {
        let defaults = Keymap::defaults(self.format);
        let mut overrides = self.inactive.clone();
        overrides.extend(
            self.bindings
                .iter()
                .zip(&defaults.bindings)
                .filter(|(b, d)| b.keys != d.keys)
                .map(|(b, _)| (b.action.to_string(), Value::from(b.keys.clone()))),
        );
        Value::Object(overrides)
    }

    fn index(&self, action: &str) -> Result<usize, String> {
        self.bindings
            .iter()
            .position(|b| b.action == action)
            .ok_or_else(|| format!("unknown action {action:?}"))
    }
}

fn parse_keys(value: &Value) -> Option<Vec<String>> {
    let keys = value.as_array()?;
    if keys.is_empty() {
        return None;
    }
    keys.iter()
        .map(|k| k.as_str().filter(|k| !k.is_empty()).map(str::to_string))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const XMP: SidecarFormat = SidecarFormat::Xmp;
    const DOP: SidecarFormat = SidecarFormat::Dop;

    fn keys_of(keymap: &Keymap, action: &str) -> Vec<String> {
        keymap
            .bindings()
            .into_iter()
            .find(|b| b.action == action)
            .unwrap()
            .keys
    }

    #[test]
    fn no_overrides_equals_the_defaults() {
        assert_eq!(Keymap::from_overrides(None, XMP), Keymap::defaults(XMP));
        assert_eq!(
            Keymap::from_overrides(Some(&json!({})), XMP),
            Keymap::defaults(XMP)
        );
    }

    #[test]
    fn a_valid_override_replaces_only_that_action() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["r"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
        let mut expected = Keymap::defaults(XMP);
        expected
            .bindings
            .iter_mut()
            .find(|b| b.action == "reject")
            .unwrap()
            .keys = vec!["r".into()];
        assert_eq!(keymap, expected);
    }

    #[test]
    fn invalid_shapes_keep_the_defaults() {
        for value in [
            json!(["r"]),
            json!("r"),
            json!({"nope": ["r"]}),
            json!({"reject": "r"}),
            json!({"reject": []}),
            json!({"reject": [""]}),
            json!({"reject": ["r", 1]}),
            json!({"reject": null}),
        ] {
            assert_eq!(
                Keymap::from_overrides(Some(&value), XMP),
                Keymap::defaults(XMP),
                "{value}"
            );
        }
    }

    #[test]
    fn an_unknown_action_does_not_block_the_others() {
        let keymap = Keymap::from_overrides(Some(&json!({"nope": ["q"], "reject": ["r"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
    }

    #[test]
    fn p_on_another_action_is_skipped() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["p"]})), XMP);
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn a_collision_with_another_default_is_skipped() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["j"]})), XMP);
        assert_eq!(keymap.bindings(), Keymap::defaults(XMP).bindings());
    }

    #[test]
    fn colliding_overrides_keep_the_first_in_action_order() {
        let keymap = Keymap::from_overrides(Some(&json!({"clear": ["r"], "reject": ["r"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
        assert_eq!(keys_of(&keymap, "clear"), vec!["0"]);
    }

    #[test]
    fn the_defaults_bind_no_key_twice() {
        for format in [XMP, DOP] {
            let mut keys: Vec<String> = Keymap::defaults(format)
                .bindings()
                .into_iter()
                .flat_map(|b| b.keys)
                .collect();
            let total = keys.len();
            keys.sort();
            keys.dedup();
            assert_eq!(keys.len(), total, "{format:?}");
        }
    }

    #[test]
    fn the_label_defaults_follow_the_format() {
        let xmp = Keymap::defaults(XMP);
        let dop = Keymap::defaults(DOP);
        for (action, x, d) in [
            ("red", vec!["6"], vec!["ctrl+alt+1"]),
            ("orange", vec![], vec!["ctrl+alt+2"]),
            ("yellow", vec!["7"], vec!["ctrl+alt+3"]),
            ("green", vec!["8"], vec!["ctrl+alt+4"]),
            ("blue", vec!["9"], vec!["ctrl+alt+5"]),
            ("pink", vec![], vec!["ctrl+alt+6"]),
            ("purple", vec!["-"], vec!["ctrl+alt+7"]),
            ("clearlabel", vec![], vec!["ctrl+alt+0"]),
        ] {
            assert_eq!(keys_of(&xmp, action), x, "{action}");
            assert_eq!(keys_of(&dop, action), d, "{action}");
        }
        assert_eq!(keys_of(&dop, "clear"), vec!["0"]);
        let actions: Vec<_> = xmp.bindings().into_iter().map(|b| b.action).collect();
        assert_eq!(
            &actions[13..],
            [
                "clear",
                "red",
                "orange",
                "yellow",
                "green",
                "blue",
                "pink",
                "purple",
                "clearlabel"
            ]
        );
    }

    #[test]
    fn a_label_override_applies_under_both_formats() {
        let overrides = json!({"red": ["r"]});
        for format in [XMP, DOP] {
            assert_eq!(
                keys_of(&Keymap::from_overrides(Some(&overrides), format), "red"),
                vec!["r"]
            );
        }
    }

    #[test]
    fn a_collision_is_checked_against_the_current_formats_defaults() {
        let overrides = json!({"reject": ["6"]});
        assert_eq!(
            Keymap::from_overrides(Some(&overrides), XMP).bindings(),
            Keymap::defaults(XMP).bindings()
        );
        assert_eq!(
            keys_of(&Keymap::from_overrides(Some(&overrides), DOP), "reject"),
            vec!["6"]
        );
    }

    #[test]
    fn ctrl_alt_keys_are_bindable() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["ctrl+alt+1"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["ctrl+alt+1"]);
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["ctrl+alt+1"]})), DOP);
        assert_eq!(keymap.bindings(), Keymap::defaults(DOP).bindings());
    }

    #[test]
    fn only_plain_p_is_reserved() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["ctrl+alt+p"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["ctrl+alt+p"]);
        let mut keymap = Keymap::defaults(DOP);
        assert!(keymap.add("unflag", "p").is_err());
        keymap.add("unflag", "ctrl+alt+p").unwrap();
        assert_eq!(keymap.overrides(), json!({"unflag": ["u", "ctrl+alt+p"]}));
    }

    #[test]
    fn overrides_and_reset_use_the_current_formats_defaults() {
        let mut keymap = Keymap::defaults(DOP);
        keymap.add("red", "6").unwrap();
        assert_eq!(keymap.overrides(), json!({"red": ["ctrl+alt+1", "6"]}));
        keymap.reset("red").unwrap();
        assert_eq!(keys_of(&keymap, "red"), vec!["ctrl+alt+1"]);
        keymap.add("red", "6").unwrap();
        keymap.remove("red", "ctrl+alt+1").unwrap();
        keymap.reset_all();
        assert_eq!(keymap, Keymap::defaults(DOP));
    }

    #[test]
    fn add_keeps_the_existing_keys() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.add("reject", "r").unwrap();
        assert_eq!(keys_of(&keymap, "reject"), vec!["x", "r"]);
        assert_eq!(keymap.overrides(), json!({"reject": ["x", "r"]}));
    }

    #[test]
    fn add_rejects_a_key_the_action_holds() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.add("reject", "x"),
            Err("\"x\" is already bound to reject".to_string())
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn add_rejects_a_key_bound_to_another_action() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.add("reject", "j"),
            Err("\"j\" is bound to next".to_string())
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn add_rejects_p_and_pick() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.add("unflag", "p"),
            Err("\"p\" is reserved for pick".to_string())
        );
        assert_eq!(
            keymap.add("pick", "q"),
            Err("pick is not editable".to_string())
        );
        assert!(keymap.add("nope", "q").is_err());
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn remove_drops_one_key() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.remove("previous", "h").unwrap();
        assert_eq!(
            keys_of(&keymap, "previous"),
            vec!["arrowleft", "arrowup", "w", "a", "k"]
        );
    }

    #[test]
    fn remove_refuses_the_last_key() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.remove("reject", "x"),
            Err("\"x\" is the only key for reject; use Reset".to_string())
        );
        assert_eq!(
            keymap.remove("pick", "p"),
            Err("pick is not editable".to_string())
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn remove_rejects_a_key_not_on_the_action() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.remove("reject", "j"),
            Err("\"j\" is not bound to reject".to_string())
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn adding_then_removing_returns_to_the_default() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.add("reject", "r").unwrap();
        keymap.remove("reject", "r").unwrap();
        assert_eq!(keymap.overrides(), json!({}));
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn reset_restores_the_defaults() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.add("reject", "r").unwrap();
        keymap.add("clear", "c").unwrap();
        keymap.remove("previous", "h").unwrap();
        keymap.reset("reject").unwrap();
        assert_eq!(
            keymap.overrides(),
            json!({"clear": ["0", "c"], "previous": ["arrowleft", "arrowup", "w", "a", "k"]})
        );
        keymap.reset_all();
        assert_eq!(keymap.overrides(), json!({}));
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn reset_refuses_a_default_key_now_bound_elsewhere() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.add("reject", "r").unwrap();
        keymap.remove("reject", "x").unwrap();
        keymap.add("clear", "x").unwrap();
        assert_eq!(
            keymap.reset("reject"),
            Err("\"x\" is bound to clear".to_string())
        );
    }

    #[test]
    fn overrides_round_trip() {
        for format in [XMP, DOP] {
            let mut keymap = Keymap::defaults(format);
            keymap.add("previous", "q").unwrap();
            keymap.remove("previous", "h").unwrap();
            keymap.add("reject", "r").unwrap();
            keymap.remove("reject", "x").unwrap();
            keymap.add("zoom", "z").unwrap();
            assert_eq!(
                Keymap::from_overrides(Some(&keymap.overrides()), format),
                keymap,
                "{format:?}"
            );
        }
    }

    #[test]
    fn forbidden_follows_the_platform() {
        assert_eq!(forbidden("meta+q", true), Some("is a menu accelerator"));
        assert_eq!(forbidden("meta+,", true), Some("is a menu accelerator"));
        assert_eq!(
            forbidden("shift+meta+4", true),
            Some("is reserved by the system")
        );
        assert_eq!(forbidden("meta+k", true), None);
        assert_eq!(forbidden("meta+arrowleft", true), None);
        assert_eq!(forbidden("shift+meta+z", true), None);
        assert_eq!(forbidden("ctrl+z", true), None);
        assert_eq!(forbidden("ctrl+z", false), Some("is a menu accelerator"));
        assert_eq!(forbidden("alt+f4", false), Some("is a menu accelerator"));
        assert_eq!(
            forbidden("alt+tab", false),
            Some("is reserved by the system")
        );
        assert_eq!(
            forbidden("meta+k", false),
            Some("is reserved by the system")
        );
        assert_eq!(
            forbidden("ctrl+alt+shift+meta+k", false),
            Some("is reserved by the system")
        );
        for key in ["shift+j", "ctrl+j", "alt+j", "ctrl+alt+1"] {
            assert_eq!(forbidden(key, true), None, "{key}");
            assert_eq!(forbidden(key, false), None, "{key}");
        }
    }

    #[test]
    fn add_refuses_a_forbidden_key() {
        let key = if MACOS { "meta+z" } else { "ctrl+z" };
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.add("reject", key),
            Err(format!("{key:?} is a menu accelerator"))
        );
        let key = if MACOS { "meta+tab" } else { "alt+tab" };
        assert_eq!(
            keymap.add("reject", key),
            Err(format!("{key:?} is reserved by the system"))
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn a_forbidden_override_is_skipped() {
        let key = if MACOS { "meta+q" } else { "ctrl+c" };
        let keymap = Keymap::from_overrides(Some(&json!({"reject": [key]})), XMP);
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn modified_keys_are_bindable() {
        let mut keymap = Keymap::defaults(XMP);
        let mut keys = vec!["shift+j", "ctrl+j", "alt+j", "ctrl+alt+1"];
        if MACOS {
            keys.push("meta+k");
        }
        for key in &keys {
            keymap.add("reject", key).unwrap();
        }
        let expected: Vec<String> = std::iter::once("x").chain(keys).map(String::from).collect();
        assert_eq!(keys_of(&keymap, "reject"), expected);
    }

    #[test]
    fn an_inactive_override_survives_unrelated_edits() {
        let stored = json!({"reject": ["6"]});
        let mut keymap = Keymap::from_overrides(Some(&stored), XMP);
        assert_eq!(keymap.overrides(), stored);
        keymap.add("zoom", "z").unwrap();
        assert_eq!(
            keymap.overrides(),
            json!({"reject": ["6"], "zoom": ["space", "z"]})
        );
        keymap.remove("zoom", "space").unwrap();
        keymap.reset("zoom").unwrap();
        assert_eq!(keymap.overrides(), stored);
        let saved = keymap.overrides();
        assert_eq!(
            keys_of(&Keymap::from_overrides(Some(&saved), DOP), "reject"),
            vec!["6"]
        );
    }

    #[test]
    fn editing_or_resetting_an_action_drops_its_inactive_override() {
        let stored = json!({"reject": ["6"], "clear": ["j"]});
        let mut keymap = Keymap::from_overrides(Some(&stored), XMP);
        keymap.add("reject", "r").unwrap();
        assert_eq!(
            keymap.overrides(),
            json!({"reject": ["x", "r"], "clear": ["j"]})
        );
        let mut keymap = Keymap::from_overrides(Some(&stored), XMP);
        keymap.add("reject", "r").unwrap();
        keymap.remove("reject", "r").unwrap();
        assert_eq!(keymap.overrides(), json!({"clear": ["j"]}));
        let mut keymap = Keymap::from_overrides(Some(&stored), XMP);
        keymap.reset("reject").unwrap();
        assert_eq!(keymap.overrides(), json!({"clear": ["j"]}));
        keymap.reset_all();
        assert_eq!(keymap.overrides(), json!({}));
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn inactive_overrides_round_trip() {
        let stored = json!({"reject": ["6"], "red": ["r"]});
        for format in [XMP, DOP] {
            let keymap = Keymap::from_overrides(Some(&stored), format);
            let reloaded = Keymap::from_overrides(Some(&keymap.overrides()), format);
            assert_eq!(reloaded.bindings(), keymap.bindings(), "{format:?}");
        }
        let xmp = Keymap::from_overrides(Some(&stored), XMP);
        assert_eq!(keys_of(&xmp, "reject"), vec!["x"]);
        let dop = Keymap::from_overrides(Some(&xmp.overrides()), DOP);
        assert_eq!(keys_of(&dop, "reject"), vec!["6"]);
        assert_eq!(keys_of(&dop, "red"), vec!["r"]);
    }
}
