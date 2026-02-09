use serde::Serialize;
use serde_evaluate::{CompositeFieldExtractor, EvaluateError, FieldScalarValue};
use std::collections::BTreeMap;

#[derive(Serialize)]
struct Record {
    name: String,
    age: u32,
    score: f64,
    active: bool,
}

fn sample_record() -> Record {
    Record {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    }
}

// =============================================================================
// Happy path
// =============================================================================

#[test]
fn basic_top_level_fields() {
    let extractor = CompositeFieldExtractor::new(&["name", "age", "score"]).unwrap();
    let values = extractor.evaluate(&sample_record()).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::String("Alice".to_string()),
            FieldScalarValue::U32(30),
            FieldScalarValue::F64(95.5),
        ]
    );
}

#[test]
fn preserves_field_order() {
    let extractor = CompositeFieldExtractor::new(&["score", "name"]).unwrap();
    let values = extractor.evaluate(&sample_record()).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::F64(95.5),
            FieldScalarValue::String("Alice".to_string()),
        ]
    );
}

#[test]
fn single_field() {
    let extractor = CompositeFieldExtractor::new(&["active"]).unwrap();
    let values = extractor.evaluate(&sample_record()).unwrap();
    assert_eq!(values, vec![FieldScalarValue::Bool(true)]);
}

#[test]
fn nested_paths() {
    #[derive(Serialize)]
    struct Outer {
        label: String,
        inner: Inner,
    }

    #[derive(Serialize)]
    struct Inner {
        value: i32,
    }

    let record = Outer {
        label: "test".to_string(),
        inner: Inner { value: 42 },
    };

    let extractor =
        CompositeFieldExtractor::new_from_paths(&[&["label"], &["inner", "value"]]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::String("test".to_string()),
            FieldScalarValue::I32(42),
        ]
    );
}

#[test]
fn mixed_scalar_types() {
    #[derive(Serialize)]
    struct Mixed {
        a_bool: bool,
        a_i64: i64,
        a_u8: u8,
        a_f32: f32,
        a_char: char,
        a_string: String,
    }

    let record = Mixed {
        a_bool: false,
        a_i64: -100,
        a_u8: 255,
        a_f32: 1.5,
        a_char: 'Z',
        a_string: "hello".to_string(),
    };

    let extractor =
        CompositeFieldExtractor::new(&["a_bool", "a_i64", "a_u8", "a_f32", "a_char", "a_string"])
            .unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::Bool(false),
            FieldScalarValue::I64(-100),
            FieldScalarValue::U8(255),
            FieldScalarValue::F32(1.5),
            FieldScalarValue::Char('Z'),
            FieldScalarValue::String("hello".to_string()),
        ]
    );
}

#[test]
fn option_fields() {
    #[derive(Serialize)]
    struct WithOptions {
        present: Option<i32>,
        absent: Option<i32>,
    }

    let record = WithOptions {
        present: Some(42),
        absent: None,
    };

    let extractor = CompositeFieldExtractor::new(&["present", "absent"]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::I32(42)))),
            FieldScalarValue::Option(None),
        ]
    );
}

#[test]
fn map_traversal() {
    #[derive(Serialize)]
    struct WithMap {
        id: u64,
        attrs: BTreeMap<String, i32>,
    }

    let mut attrs = BTreeMap::new();
    attrs.insert("priority".to_string(), 5);

    let record = WithMap { id: 1, attrs };

    let extractor =
        CompositeFieldExtractor::new_from_paths(&[&["id"], &["attrs", "priority"]]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![FieldScalarValue::U64(1), FieldScalarValue::I32(5)]
    );
}

#[test]
fn deeply_nested_paths() {
    #[derive(Serialize)]
    struct L0 {
        l1: L1,
        top: String,
    }

    #[derive(Serialize)]
    struct L1 {
        l2: L2,
    }

    #[derive(Serialize)]
    struct L2 {
        leaf: bool,
    }

    let record = L0 {
        l1: L1 {
            l2: L2 { leaf: true },
        },
        top: "root".to_string(),
    };

    let extractor =
        CompositeFieldExtractor::new_from_paths(&[&["top"], &["l1", "l2", "leaf"]]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::String("root".to_string()),
            FieldScalarValue::Bool(true),
        ]
    );
}

#[test]
fn duplicate_field() {
    let extractor = CompositeFieldExtractor::new(&["name", "age", "name"]).unwrap();
    let values = extractor.evaluate(&sample_record()).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::String("Alice".to_string()),
            FieldScalarValue::U32(30),
            FieldScalarValue::String("Alice".to_string()),
        ]
    );
}

#[test]
fn bytes_field() {
    #[derive(Serialize)]
    struct WithBytes {
        id: u32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    let record = WithBytes {
        id: 7,
        data: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };

    let extractor = CompositeFieldExtractor::new(&["id", "data"]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::U32(7),
            FieldScalarValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ]
    );
}

#[test]
fn nested_options() {
    #[derive(Serialize)]
    struct WithNestedOpts {
        label: String,
        double_some: Option<Option<i32>>,
        some_none: Option<Option<i32>>,
        none: Option<Option<i32>>,
    }

    let record = WithNestedOpts {
        label: "x".to_string(),
        double_some: Some(Some(99)),
        some_none: Some(None),
        none: None,
    };

    let extractor =
        CompositeFieldExtractor::new(&["label", "double_some", "some_none", "none"]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::String("x".to_string()),
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::I32(99)
            )))))),
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::Option(None)))),
            FieldScalarValue::Option(None),
        ]
    );
}

// =============================================================================
// Error cases
// =============================================================================

#[test]
fn error_empty_field_list() {
    let fields: &[&str] = &[];
    let err = CompositeFieldExtractor::new(fields).unwrap_err();
    assert!(matches!(err, EvaluateError::InvalidPath(_)));
}

#[test]
fn error_empty_paths_list() {
    let paths: &[&[&str]] = &[];
    let err = CompositeFieldExtractor::new_from_paths(paths).unwrap_err();
    assert!(matches!(err, EvaluateError::InvalidPath(_)));
}

#[test]
fn error_empty_field_name() {
    let err = CompositeFieldExtractor::new(&[""]).unwrap_err();
    assert!(matches!(err, EvaluateError::InvalidPath(_)));
}

#[test]
fn error_empty_path_segment() {
    let err = CompositeFieldExtractor::new_from_paths(&[&[""]]).unwrap_err();
    assert!(matches!(err, EvaluateError::InvalidPath(_)));
}

#[test]
fn error_empty_inner_path() {
    let empty: &[&str] = &[];
    let err = CompositeFieldExtractor::new_from_paths(&[empty]).unwrap_err();
    assert!(matches!(err, EvaluateError::InvalidPath(_)));
}

#[test]
fn error_field_not_found() {
    let extractor = CompositeFieldExtractor::new(&["name", "nonexistent"]).unwrap();
    let err = extractor.evaluate(&sample_record()).unwrap_err();
    assert!(matches!(
        err,
        EvaluateError::NestedFieldNotFound { ref path, .. } if path == &vec!["nonexistent".to_string()]
    ));
}

#[test]
fn error_unsupported_type() {
    #[derive(Serialize)]
    struct HasStruct {
        label: String,
        nested: Inner,
    }

    #[derive(Serialize)]
    struct Inner {
        value: i32,
    }

    let record = HasStruct {
        label: "ok".to_string(),
        nested: Inner { value: 1 },
    };

    // Targeting the struct itself (not a scalar within it) should fail
    let extractor = CompositeFieldExtractor::new_from_paths(&[&["label"], &["nested"]]).unwrap();
    let err = extractor.evaluate(&record).unwrap_err();
    assert!(matches!(err, EvaluateError::UnsupportedType { .. }));
}

#[test]
fn error_first_field_fails_fast() {
    let extractor = CompositeFieldExtractor::new(&["missing", "name"]).unwrap();
    let err = extractor.evaluate(&sample_record()).unwrap_err();
    // Should fail on "missing" before even trying "name"
    assert!(matches!(
        err,
        EvaluateError::NestedFieldNotFound { ref path, .. } if path == &vec!["missing".to_string()]
    ));
}
