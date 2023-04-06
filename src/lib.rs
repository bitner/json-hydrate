use serde_json::{Value};

pub fn hydrate(base: Value, item: &mut Value) {
    if item.is_null(){
        println!("item {} is empty base is {}", item, base);
        *item = base.clone();
    }
    else if item.is_object() {
        if item.as_object() == base.as_object(){
            println!("item {} == base {}", item, base);
            return;
        }
        if let Value::Object(item) = item {
            if let Value::Object(ref base) = base {
                for (key, baseval) in base {
                    if item.get(key).is_some(){
                        let itemval = item.get_mut(key).unwrap();
                        println!("key {} baseval {} itemval {}", key, baseval, itemval);
                        if itemval.as_str() == Some("𒍟※" ){
                            println!("REMOVE KEY {}", key);
                            item.remove(key);
                        }
                        else {
                            hydrate(baseval.clone(), itemval);
                            println!("BASE {} ITEM {}", baseval, itemval);
                        }

                    }
                    else {
                        println!("key {} is null in item", key);
                        item.entry(key).or_insert(baseval.clone());
                    }

                }
            }
        }
    }
    else if item.is_array(){
        if let Value::Array(item) = item {
            if let Value::Array(ref base) = base {
                if item.len() == base.len(){
                    println!("item len == base len");
                    for i in 0..item.len(){
                        println!("array element {} {}", item[i], base[i]);
                        hydrate(base[i].clone(), &mut item[i]);
                    }
                }
                else {
                    println!("item len != base len");
                }
            }
        }
    }

    println!("FINAL ITEM {} BASE {}", item, base);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json};

    #[test]
    fn test_equal_hydrate() {
        let base = json!({"a": "first", "b": "second", "c": "third"});
        let mut item = json!({"a": "first", "b": "second", "c": "third"});
        let target = json!({"a": "first", "b": "second", "c": "third"});
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_full_hydrate() {
        let base = json!({"a": "first", "b": "second", "c": "third"});
        let mut item = json!({});
        let target = json!({"a": "first", "b": "second", "c": "third"});
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_full_nested() {
        let base = json!({"a": "first", "b": "second", "c": {"d": "third"}});
        let mut item = json!({});
        let target = json!({"a": "first", "b": "second", "c": {"d": "third"}});
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_list_of_dicts_extra_keys() {
        let base = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        let mut item = json!({"a": [{"b3": 3}, {"c3": 3}]});
        let target = json!({
            "a": [{"b1": 1, "b2": 2, "b3": 3}, {"c1": 1, "c2": 2, "c3": 3}],
        });
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_unequal_len_list() {
        let base = json!({"a": [{"b1": 1}, {"c1": 1}, {"d1": 1}]});
        let mut item = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        let target = json!({"a": [{"b1": 1, "b2": 2}, {"c1": 1, "c2": 2}]});
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_deeply_nested_dict() {
        let base = json!({"a": {"b": {"c": {"d": "first", "d1": "second"}}}});
        let mut item = json!({"a": {"b": {"c": {"d2": "third"}}}});
        let target = json!({
            "a": {"b": {"c": {"d": "first", "d1": "second", "d2": "third"}}},
        });
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

    #[test]
    fn test_equal_list_of_non_dicts() {
        let base = json!({"assets": {"thumbnail": {"roles": ["thumbnail"]}}});
        let mut item = json!({"assets": {"thumbnail": {"href": "http://foo.com"}}});
        let target = json!({
            "assets": {"thumbnail": {"roles": ["thumbnail"], "href": "http://foo.com"}},
        });
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
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
        hydrate(base, &mut item);
        assert_eq!(item, target);
    }

}
