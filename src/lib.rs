use serde_json::{Map, Value};
use thiserror::Error;

const MAGIC_MARKER: &str = "𒍟※";

pub trait Hydrate {
    fn hydrate(&mut self, base: Self) -> Result<(), Error>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("type mismatch")]
    TypeMismatch(Value, Value),
}

impl Hydrate for Value {
    fn hydrate(&mut self, base: Self) -> Result<(), Error> {
        match self {
            Value::Object(item) => match base {
                Value::Object(base) => item.hydrate(base),
                _ => Err(Error::TypeMismatch(self.clone(), base)),
            },
            Value::Array(item) => match base {
                Value::Array(base) => item.hydrate(base),
                _ => Err(Error::TypeMismatch(self.clone(), base)),
            },
            _ => Ok(()),
        }
    }
}

impl Hydrate for Vec<Value> {
    fn hydrate(&mut self, base: Self) -> Result<(), Error> {
        for (item, base) in self.iter_mut().zip(base.into_iter()) {
            item.hydrate(base)?;
        }
        Ok(())
    }
}

impl Hydrate for Map<String, Value> {
    fn hydrate(&mut self, base: Self) -> Result<(), Error> {
        for (key, base_value) in base {
            if self
                .get(&key)
                .and_then(|value| value.as_str())
                .map(|s| s == MAGIC_MARKER)
                .unwrap_or(false)
            {
                self.remove(&key);
            } else if let Some(self_value) = self.get_mut(&key) {
                self_value.hydrate(base_value)?;
            } else {
                self.insert(key, base_value);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Hydrate;
    use serde_json::json;

    #[test]
    fn test_equal_hydrate() {
        let base = json!({"a": "first", "b": "second", "c": "third"});
        let mut item = json!({"a": "first", "b": "second", "c": "third"});
        let target = json!({"a": "first", "b": "second", "c": "third"});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_full_hydrate() {
        let base = json!({"a": "first", "b": "second", "c": "third"});
        let mut item = json!({});
        let target = json!({"a": "first", "b": "second", "c": "third"});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_full_nested() {
        let base = json!({"a": "first", "b": "second", "c": {"d": "third"}});
        let mut item = json!({});
        let target = json!({"a": "first", "b": "second", "c": {"d": "third"}});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_nested_extra_keys() {
        let base = json!({"a": "first", "b": "second", "c": {"d": "third"}});
        let mut item = json!({"c": {"e": "fourth", "f": "fifth"}});
        let target = json!({
            "a": "first",
            "b": "second",
            "c": {"d": "third", "e": "fourth", "f": "fifth"},
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_list_of_dicts_extra_keys() {
        let base = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        let mut item = json!({"a": [{"b3": 3}, {"c3": 3}]});
        let target = json!({
            "a": [{"b1": 1, "b2": 2, "b3": 3}, {"c1": 1, "c2": 2, "c3": 3}],
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_equal_len_list_of_mixed_types() {
        let base = json!({"a": [{"b1": 1, "b2": 2}, "foo", {"c1": 1, "c2": 2}, "bar"]});
        let mut item = json!({"a": [{"b3": 3}, "far", {"c3": 3}, "boo"]});
        let target = json!({
            "a": [
                {"b1": 1, "b2": 2, "b3": 3},
                "far",
                {"c1": 1, "c2": 2, "c3": 3},
                "boo",
            ],
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_unequal_len_list() {
        let base = json!({"a": [{"b1": 1}, {"c1": 1}, {"d1": 1}]});
        let mut item = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        let target = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_marked_non_merged_fields() {
        let base = json!({
            "a": "first",
            "b": "second",
            "c": {"d": "third", "e": "fourth"},
        });
        let mut item = json!({"c": {"e": "𒍟※", "f": "fifth"}});
        let target = json!({
            "a": "first",
            "b": "second",
            "c": {"d": "third", "f": "fifth"},
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_marked_non_merged_fields_in_list() {
        let base = json!({
            "a": [{"b": "first", "d": "third"}, {"c": "second", "e": "fourth"}],
        });
        let mut item = json!({
            "a": [
                {"d": "𒍟※"},
                {"e": "𒍟※", "f": "fifth"},
            ],
        });
        let target = json!({"a": [{"b": "first"}, {"c": "second", "f": "fifth"}]});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_deeply_nested_dict() {
        let base = json!({"a": {"b": {"c": {"d": "first", "d1": "second"}}}});
        let mut item = json!({"a": {"b": {"c": {"d2": "third"}}}});
        let target = json!({
            "a": {"b": {"c": {"d": "first", "d1": "second", "d2": "third"}}},
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_equal_list_of_non_dicts() {
        let base = json!({"assets": {"thumbnail": {"roles": ["thumbnail"]}}});
        let mut item = json!({"assets": {"thumbnail": {"href": "http://foo.com"}}});
        let target = json!({
            "assets": {"thumbnail": {"roles": ["thumbnail"], "href": "http://foo.com"}},
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_invalid_assets_removed() {
        let base = json!({
            "type": "Feature",
            "assets": {
                "asset1": {"name": "Asset one"},
                "asset2": {"name": "Asset two"},
            },
        });
        let mut item = json!({
            "assets": {
                "asset1": {"href": "http://foo.com"},
                "asset2": "𒍟※",
            },
        });
        let target = json!({
            "type": "Feature",
            "assets": {"asset1": {"name": "Asset one", "href": "http://foo.com"}},
        });
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }

    #[test]
    fn test_top_level_base_keys_marked() {
        let base = json!({
            "single": "Feature",
            "double": {"nested": "value"},
            "triple": {"nested": {"deep": "value"}},
            "included": "value",
        });
        let mut item = json!({
            "single": "𒍟※",
            "double": "𒍟※",
            "triple": "𒍟※",
            "unique": "value",
        });
        let target = json!({"included": "value", "unique": "value"});
        item.hydrate(base).unwrap();
        assert_eq!(item, target);
    }
}
