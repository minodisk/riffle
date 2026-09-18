//! The culling keymap: each action's default keys and the user's overrides
//! from the `shortcuts` settings key, merged so no key is bound twice.

use serde::Serialize;
use serde_json::Value;

/// Every action in the order the shortcuts panel shows them, with its
/// default keys. Key names are `event.key` lower-cased, with `" "` as
/// `"space"`.
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

/// The key reserved for `pick`.
const PICK_KEY: &str = "p";

/// One action and the keys bound to it, as the `shortcuts` command returns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Binding {
    pub action: &'static str,
    pub keys: Vec<String>,
}

/// The resolved keymap, one entry per action in `DEFAULTS` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap(Vec<Binding>);

impl Default for Keymap {
    fn default() -> Self {
        Keymap(
            DEFAULTS
                .iter()
                .map(|(action, keys)| Binding {
                    action,
                    keys: keys.iter().map(|k| k.to_string()).collect(),
                })
                .collect(),
        )
    }
}

impl Keymap {
    /// Merge the stored `shortcuts` value over the defaults. Every entry that
    /// cannot be used is logged and the action keeps its default; an override
    /// whose key is already bound to another action is skipped, so the
    /// earlier action in `DEFAULTS` order wins.
    pub fn from_overrides(overrides: Option<&Value>) -> Keymap {
        let mut keymap = Keymap::default();
        let Some(overrides) = overrides else {
            return keymap;
        };
        let Some(overrides) = overrides.as_object() else {
            log::warn!("ignoring the shortcuts setting: not an object");
            return keymap;
        };
        for name in overrides.keys() {
            if !DEFAULTS.iter().any(|(action, _)| action == name) {
                log::warn!("ignoring the shortcut for an unknown action {name:?}");
            }
        }
        for i in 0..keymap.0.len() {
            let action = keymap.0[i].action;
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
                    .0
                    .iter()
                    .find(|b| b.action != action && b.keys.contains(key))
                    .map(|b| (key, b.action))
            });
            if let Some((key, other)) = conflict {
                log::warn!("ignoring the shortcut for {action}: {key:?} is bound to {other}");
                continue;
            }
            keymap.0[i].keys = keys;
        }
        keymap
    }

    /// The bindings in the order the shortcuts panel shows them.
    pub fn bindings(&self) -> Vec<Binding> {
        self.0.clone()
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
        assert_eq!(Keymap::from_overrides(None), Keymap::default());
        assert_eq!(Keymap::from_overrides(Some(&json!({}))), Keymap::default());
    }

    #[test]
    fn a_valid_override_replaces_only_that_action() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["r"]})));
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
        let mut expected = Keymap::default();
        expected
            .0
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
                Keymap::from_overrides(Some(&value)),
                Keymap::default(),
                "{value}"
            );
        }
    }

    #[test]
    fn an_unknown_action_does_not_block_the_others() {
        let keymap = Keymap::from_overrides(Some(&json!({"nope": ["q"], "reject": ["r"]})));
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
    }

    #[test]
    fn p_on_another_action_is_skipped() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["p"]})));
        assert_eq!(keymap, Keymap::default());
    }

    #[test]
    fn a_collision_with_another_default_is_skipped() {
        let keymap = Keymap::from_overrides(Some(&json!({"reject": ["j"]})));
        assert_eq!(keymap, Keymap::default());
    }

    #[test]
    fn colliding_overrides_keep_the_first_in_action_order() {
        let keymap = Keymap::from_overrides(Some(&json!({"clear": ["r"], "reject": ["r"]})));
        assert_eq!(keys_of(&keymap, "reject"), vec!["r"]);
        assert_eq!(keys_of(&keymap, "clear"), vec!["0"]);
    }

    #[test]
    fn the_defaults_bind_no_key_twice() {
        let mut keys: Vec<&str> = DEFAULTS
            .iter()
            .flat_map(|(_, keys)| keys.iter().copied())
            .collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), total);
    }
}
