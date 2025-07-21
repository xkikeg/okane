//! Module defining misc utilities.
//! Eventually each can be individually published as a crate.

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

/// Structured configuration holding overrides with default values.
#[derive(Debug, Default)]
pub struct ConfigResolver<Key, ConfigValue> {
    base: ConfigValue,
    overrides: HashMap<Key, ConfigValue>,
}

impl<Key, ConfigValue> ConfigResolver<Key, ConfigValue> {
    /// Create a new instance of [`Config`].
    pub fn new(base: ConfigValue, overrides: HashMap<Key, ConfigValue>) -> Self {
        Self { base, overrides }
    }

    /// Returns an always-existing field for the given `key`.
    /// Note field is `FnOnce` so it can be arbitrary function as long as it makes sense.
    pub fn get<Q, F, Out>(&self, key: &Q, field: F) -> Out
    where
        Key: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&ConfigValue) -> Out,
    {
        match self.overrides.get(key) {
            None => field(&self.base),
            Some(set) => field(set),
        }
    }

    /// Returns an optional field for the given `key`.
    /// Note field as `FnMut` returning any type of [`Option`].
    pub fn get_opt<Q, F, Out>(&self, key: &Q, mut field: F) -> Option<Out>
    where
        Key: Borrow<Q> + Eq + Hash,
        Q: Hash + Eq + ?Sized,
        F: FnMut(&ConfigValue) -> Option<Out>,
    {
        match self.overrides.get(key) {
            None => field(&self.base),
            Some(set) => field(set).or_else(|| field(&self.base)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[derive(Debug)]
    struct TestStruct {
        int_field: i32,
        str_field: &'static str,
    }

    impl TestStruct {
        fn str_opt(&self) -> Option<&'static str> {
            if self.str_field.is_empty() {
                None
            } else {
                Some(self.str_field)
            }
        }
    }

    fn input() -> ConfigResolver<&'static str, TestStruct> {
        ConfigResolver::new(
            TestStruct {
                int_field: 1,
                str_field: "base",
            },
            hashmap! {
                "key1" => TestStruct {
                    int_field: 0,
                    str_field: "",
                },
                "key2" => TestStruct {
                    int_field: 2,
                    str_field: "non-empty",
                },
            },
        )
    }

    #[test]
    fn get_returns_base_value_if_no_overrides() {
        assert_eq!(1, input().get("not_existing", |x| x.int_field));
    }

    #[test]
    fn get_returns_override_value_if_key_exists() {
        assert_eq!(2, input().get("key2", |x| x.int_field));
    }

    #[test]
    fn get_opt_returns_base_value_if_no_overrides() {
        assert_eq!(
            Some("base"),
            input().get_opt("not_existing", TestStruct::str_opt)
        );
    }

    #[test]
    fn get_opt_returns_base_value_if_overrides_returns_none() {
        assert_eq!(Some("base"), input().get_opt("key1", TestStruct::str_opt));
    }

    #[test]
    fn get_opt_returns_override_value() {
        assert_eq!(
            Some("non-empty"),
            input().get_opt("key2", TestStruct::str_opt)
        );
    }
}
