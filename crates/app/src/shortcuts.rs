//! The culling keymap: each action's default keys and the user's overrides
//! from the `shortcuts` settings key, merged so no key is bound twice. The
//! colour label actions' defaults follow the selected sidecar format.

use serde::Serialize;
use serde_json::Value;

use crate::sidecar::SidecarFormat;

/// Every action in the order the shortcuts panel shows them, with its
/// default keys under either format. Key names are `event.key` lower-cased,
/// with `" "` as `"space"`, or `ctrl+alt+` and the key named from
/// `event.code` (`Digit1` -> `1`), as Option changes `event.key` on macOS.
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
        }
    }

    /// Merge the stored `shortcuts` value over the defaults. Every entry that
    /// cannot be used is logged and the action keeps its default; an override
    /// whose key is already bound to another action is skipped, so the
    /// earlier action wins. Conflicts are checked against `format`'s
    /// defaults, so one override can apply under one format only.
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
            let conflict = keys.iter().find_map(|key| {
                keymap
                    .bindings
                    .iter()
                    .find(|b| b.action != action && b.keys.contains(key))
                    .map(|b| (key, b.action))
            });
            if let Some((key, other)) = conflict {
                log::warn!("ignoring the shortcut for {action}: {key:?} is bound to {other}");
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

    /// Bind `action` to `key` alone, or explain why it cannot be: `pick` is
    /// not editable, `p` is reserved for pick, and a key bound to another
    /// action is refused rather than moved.
    pub fn rebind(&mut self, action: &str, key: &str) -> Result<(), String> {
        let i = self.index(action)?;
        if action == "pick" {
            return Err("pick is not editable".to_string());
        }
        if key == PICK_KEY {
            return Err("\"p\" is reserved for pick".to_string());
        }
        if let Some(other) = self
            .bindings
            .iter()
            .find(|b| b.action != action && b.keys.iter().any(|k| k == key))
        {
            return Err(format!("{key:?} is bound to {}", other.action));
        }
        self.bindings[i].keys = vec![key.to_string()];
        Ok(())
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
        Ok(())
    }

    /// Restore every action's default keys.
    pub fn reset_all(&mut self) {
        *self = Keymap::defaults(self.format);
    }

    /// The actions whose keys differ from the default, as stored under the
    /// `shortcuts` settings key. Compared with the current format's defaults.
    pub fn overrides(&self) -> Value {
        let defaults = Keymap::defaults(self.format);
        Value::Object(
            self.bindings
                .iter()
                .zip(&defaults.bindings)
                .filter(|(b, d)| b.keys != d.keys)
                .map(|(b, _)| (b.action.to_string(), Value::from(b.keys.clone())))
                .collect(),
        )
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
        assert_eq!(keymap, Keymap::defaults(XMP));
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
            Keymap::from_overrides(Some(&overrides), XMP),
            Keymap::defaults(XMP)
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
        assert_eq!(keymap, Keymap::defaults(DOP));
    }

    #[test]
    fn only_plain_p_is_reserved() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["ctrl+alt+p"]})), XMP);
        assert_eq!(keys_of(&keymap, "reject"), vec!["ctrl+alt+p"]);
        let mut keymap = Keymap::defaults(DOP);
        assert!(keymap.rebind("unflag", "p").is_err());
        keymap.rebind("unflag", "ctrl+alt+p").unwrap();
        assert_eq!(keymap.overrides(), json!({"unflag": ["ctrl+alt+p"]}));
    }

    #[test]
    fn overrides_and_reset_use_the_current_formats_defaults() {
        let mut keymap = Keymap::defaults(DOP);
        keymap.rebind("red", "6").unwrap();
        assert_eq!(keymap.overrides(), json!({"red": ["6"]}));
        keymap.reset("red").unwrap();
        assert_eq!(keys_of(&keymap, "red"), vec!["ctrl+alt+1"]);
        keymap.reset_all();
        assert_eq!(keymap, Keymap::defaults(DOP));
    }

    #[test]
    fn a_rebind_is_the_only_override() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.rebind("reject", "r").unwrap();
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
        assert_eq!(keymap.overrides(), json!({"reject": ["r"]}));
    }

    #[test]
    fn rebinding_back_to_the_default_removes_the_override() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.rebind("reject", "r").unwrap();
        keymap.rebind("reject", "x").unwrap();
        assert_eq!(keymap.overrides(), json!({}));
    }

    #[test]
    fn rebind_rejects_a_key_bound_to_another_action() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.rebind("reject", "j"),
            Err("\"j\" is bound to next".to_string())
        );
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn rebind_rejects_p_and_pick() {
        let mut keymap = Keymap::defaults(XMP);
        assert_eq!(
            keymap.rebind("unflag", "p"),
            Err("\"p\" is reserved for pick".to_string())
        );
        assert_eq!(
            keymap.rebind("pick", "q"),
            Err("pick is not editable".to_string())
        );
        assert!(keymap.rebind("nope", "q").is_err());
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn reset_restores_the_defaults() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.rebind("reject", "r").unwrap();
        keymap.rebind("clear", "c").unwrap();
        keymap.reset("reject").unwrap();
        assert_eq!(keymap.overrides(), json!({"clear": ["c"]}));
        keymap.reset_all();
        assert_eq!(keymap.overrides(), json!({}));
        assert_eq!(keymap, Keymap::defaults(XMP));
    }

    #[test]
    fn reset_refuses_a_default_key_now_bound_elsewhere() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.rebind("reject", "r").unwrap();
        keymap.rebind("clear", "x").unwrap();
        assert_eq!(
            keymap.reset("reject"),
            Err("\"x\" is bound to clear".to_string())
        );
    }

    #[test]
    fn overrides_round_trip() {
        let mut keymap = Keymap::defaults(XMP);
        keymap.rebind("previous", "q").unwrap();
        keymap.rebind("reject", "r").unwrap();
        keymap.rebind("zoom", "z").unwrap();
        assert_eq!(
            Keymap::from_overrides(Some(&keymap.overrides()), XMP),
            keymap
        );
    }
}
