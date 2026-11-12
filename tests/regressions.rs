//! Regression tests for reported issues, one section per issue.

use serde::{Serialize, Serializer};
use serde_evaluate::{
    EvaluateError, FieldExtractor, FieldScalarValue, ListFieldExtractor, NestedFieldExtractor,
    NestedListFieldExtractor,
};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};

fn nested_not_found(path: &[&str], index: usize) -> EvaluateError {
    EvaluateError::NestedFieldNotFound {
        path: path.iter().map(|s| s.to_string()).collect(),
        failed_at_index: Some(index),
    }
}

// =============================================================================
// #29: ListFieldExtractor returns Ok([]) for non-list targets
// =============================================================================

#[derive(Serialize)]
struct Scalars {
    id: u32,
    opt: Option<u32>,
    name: String,
    unit: (),
    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
    inner: Inner,
}

#[derive(Serialize)]
struct Inner {
    value: u32,
}

fn scalars() -> Scalars {
    Scalars {
        id: 7,
        opt: Some(3),
        name: "n".to_string(),
        unit: (),
        bytes: vec![1, 2],
        inner: Inner { value: 1 },
    }
}

#[test]
fn list_extractor_rejects_scalar_targets() {
    for field in ["id", "opt", "name", "bytes"] {
        let result = ListFieldExtractor::new(field).evaluate(&scalars());
        assert!(
            matches!(result, Err(EvaluateError::UnsupportedType { .. })),
            "field {field}: expected UnsupportedType, got {result:?}"
        );
    }
    let nested = NestedListFieldExtractor::new_from_path(&["inner", "value"]).unwrap();
    assert!(matches!(
        nested.evaluate(&scalars()),
        Err(EvaluateError::UnsupportedType { .. })
    ));
}

#[test]
fn list_extractor_treats_unit_and_null_as_empty() {
    assert_eq!(
        ListFieldExtractor::new("unit").evaluate(&scalars()),
        Ok(vec![])
    );
    let json = serde_json::json!({ "tags": null });
    assert_eq!(ListFieldExtractor::new("tags").evaluate(&json), Ok(vec![]));
}

// =============================================================================
// #30: unit enum variants are captured by name
// =============================================================================

#[derive(Serialize, Clone, Copy)]
enum Color {
    Red,
    Green,
}

#[test]
fn unit_variants_are_captured_as_strings() {
    #[derive(Serialize)]
    struct S {
        color: Color,
        colors: Vec<Color>,
    }
    let s = S {
        color: Color::Green,
        colors: vec![Color::Red, Color::Green],
    };
    assert_eq!(
        FieldExtractor::new("color").evaluate(&s),
        Ok(FieldScalarValue::String("Green".to_string()))
    );
    assert_eq!(
        ListFieldExtractor::new("colors").evaluate(&s),
        Ok(vec![
            FieldScalarValue::String("Red".to_string()),
            FieldScalarValue::String("Green".to_string()),
        ])
    );
}

// =============================================================================
// #31: a non-string map key is a non-match, not an error
// =============================================================================

#[derive(Serialize)]
struct Flattened {
    a: u32,
    #[serde(flatten)]
    extra: BTreeMap<(u32, u32), u32>,
    z: u32,
}

fn flattened() -> Flattened {
    let mut extra = BTreeMap::new();
    extra.insert((1, 2), 3);
    Flattened { a: 1, extra, z: 26 }
}

#[test]
fn flatten_with_unstringifiable_keys_finds_later_fields() {
    let record = flattened();
    assert_eq!(
        FieldExtractor::new("a").evaluate(&record),
        Ok(FieldScalarValue::U32(1))
    );
    assert_eq!(
        FieldExtractor::new("z").evaluate(&record),
        Ok(FieldScalarValue::U32(26))
    );
    assert_eq!(
        FieldExtractor::new("missing").evaluate(&record),
        Err(EvaluateError::FieldNotFound {
            field_name: "missing".to_string()
        })
    );
}

#[test]
fn root_map_with_unstringifiable_keys_is_not_found() {
    let mut map: HashMap<(u8, u8), u32> = HashMap::new();
    map.insert((1, 1), 1);
    assert_eq!(
        FieldExtractor::new("x").evaluate(&map),
        Err(EvaluateError::FieldNotFound {
            field_name: "x".to_string()
        })
    );
}

// =============================================================================
// #32: newtype, char, unit-variant and integer map keys
// =============================================================================

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Id(String);

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Red,
}

#[test]
fn map_keys_are_matched_by_string_form() {
    #[derive(Serialize)]
    struct S {
        by_id: BTreeMap<Id, u32>,
        by_color: BTreeMap<Key, u32>,
        by_char: BTreeMap<char, u32>,
        by_int: BTreeMap<u32, Inner>,
    }
    let s = S {
        by_id: [(Id("k".to_string()), 1)].into(),
        by_color: [(Key::Red, 2)].into(),
        by_char: [('c', 3)].into(),
        by_int: [(5, Inner { value: 4 })].into(),
    };
    let get = |path: &[&str]| {
        NestedFieldExtractor::new_from_path(path)
            .unwrap()
            .evaluate(&s)
    };
    assert_eq!(get(&["by_id", "k"]), Ok(FieldScalarValue::U32(1)));
    assert_eq!(get(&["by_color", "Red"]), Ok(FieldScalarValue::U32(2)));
    assert_eq!(get(&["by_char", "c"]), Ok(FieldScalarValue::U32(3)));
    assert_eq!(get(&["by_int", "5", "value"]), Ok(FieldScalarValue::U32(4)));
}

// =============================================================================
// #33: fields skipped by skip_serializing_if
// =============================================================================

#[derive(Serialize)]
struct Skipping {
    #[serde(skip_serializing_if = "Option::is_none")]
    v: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    l: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inner: Option<Inner>,
}

#[test]
fn skipped_fields_are_absent_not_missing() {
    let s = Skipping {
        v: None,
        l: vec![],
        inner: None,
    };
    assert_eq!(
        FieldExtractor::new("v").evaluate(&s),
        Ok(FieldScalarValue::Option(None))
    );
    assert_eq!(ListFieldExtractor::new("l").evaluate(&s), Ok(vec![]));
    // A typo is still reported as missing.
    assert!(matches!(
        FieldExtractor::new("vv").evaluate(&s),
        Err(EvaluateError::FieldNotFound { .. })
    ));
    // Traversing through a skipped field fails after that field.
    let path = ["inner", "value"];
    assert_eq!(
        NestedFieldExtractor::new_from_path(&path)
            .unwrap()
            .evaluate(&s),
        Err(nested_not_found(&path, 1))
    );
}

// =============================================================================
// #34: fixed-size arrays
// =============================================================================

#[test]
fn list_extractor_accepts_fixed_size_arrays() {
    #[derive(Serialize)]
    struct S {
        arr: [u32; 3],
    }
    assert_eq!(
        ListFieldExtractor::new("arr").evaluate(&S { arr: [1, 2, 3] }),
        Ok(vec![
            FieldScalarValue::U32(1),
            FieldScalarValue::U32(2),
            FieldScalarValue::U32(3),
        ])
    );
}

// =============================================================================
// #35: failed_at_index points at the first unmatched segment
// =============================================================================

#[derive(Serialize)]
struct Deep {
    a: A,
}

#[derive(Serialize)]
struct A {
    m: BTreeMap<String, u32>,
    s: u32,
    o: Option<Inner>,
    b: B,
}

#[derive(Serialize)]
struct B {
    c: u32,
}

fn deep() -> Deep {
    Deep {
        a: A {
            m: [("y".to_string(), 1)].into(),
            s: 1,
            o: None,
            b: B { c: 1 },
        },
    }
}

#[test]
fn failed_at_index_is_exact() {
    let cases: &[(&[&str], usize)] = &[
        (&["x"], 0),
        (&["a", "x"], 1),
        (&["a", "m", "x", "c"], 2),
        (&["a", "s", "c"], 2),
        (&["a", "o", "c"], 2),
        (&["a", "b", "c", "d"], 3),
    ];
    for (path, index) in cases {
        let result = NestedFieldExtractor::new_from_path(path)
            .unwrap()
            .evaluate(&deep());
        assert_eq!(result, Err(nested_not_found(path, *index)), "path {path:?}");
    }
}

#[test]
fn failed_at_index_on_json_value() {
    let json = serde_json::json!({ "a": { "b": 1 } });
    let path = ["a", "x", "y"];
    assert_eq!(
        NestedFieldExtractor::new_from_path(&path)
            .unwrap()
            .evaluate(&json),
        Err(nested_not_found(&path, 1))
    );
    assert_eq!(
        nested_not_found(&path, 1).to_string(),
        "Nested field path 'a.x.y' not found (failed at segment 1: 'x')"
    );
}

// =============================================================================
// #38: empty field names are rejected
// =============================================================================

#[test]
fn empty_field_name_is_invalid() {
    assert!(matches!(
        FieldExtractor::new("").evaluate(&scalars()),
        Err(EvaluateError::InvalidPath(_))
    ));
    assert!(matches!(
        ListFieldExtractor::new("").evaluate(&scalars()),
        Err(EvaluateError::InvalidPath(_))
    ));
}

// =============================================================================
// #39: values that are not on the path are not serialized
// =============================================================================

struct Counted<'a>(&'a Cell<usize>);

impl Serialize for Counted<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.set(self.0.get() + 1);
        serializer.serialize_u32(0)
    }
}

#[test]
fn unmatched_map_values_are_not_serialized() {
    let calls = Cell::new(0);
    let map: BTreeMap<String, Counted> = (0..10)
        .map(|i| (format!("k{i}"), Counted(&calls)))
        .collect();
    assert_eq!(
        FieldExtractor::new("k3").evaluate(&map),
        Ok(FieldScalarValue::U32(0))
    );
    assert_eq!(calls.get(), 1);
}
