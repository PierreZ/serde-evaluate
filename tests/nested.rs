use serde::Serialize;
use serde_evaluate::{error::EvaluateError, value::FieldScalarValue};

// Define nested structures for testing

#[derive(Serialize)]
struct Level3 {
    target_field: i32,
    other_field: String,
}

#[derive(Serialize)]
struct Level2 {
    level3: Level3,
    another_val: bool,
}

#[derive(Serialize)]
struct Level1 {
    level2: Level2,
    id: u64,
}

#[derive(Serialize)]
struct Inner {
    value: i32,
    another_value: Option<String>,
    nested_struct: InnerMost,
}

#[derive(Serialize)]
struct InnerMost {
    deep_value: bool,
}

#[derive(Serialize)]
struct Middle {
    inner: Inner,
    primitive: u64,
}

#[derive(Serialize)]
struct Outer {
    middle: Middle,
    name: String,
}

#[test]
fn test_extract_deeply_nested_field() {
    let data = Level1 {
        id: 101,
        level2: Level2 {
            another_val: true,
            level3: Level3 {
                target_field: 999,
                other_field: "hello".to_string(),
            },
        },
    };

    // Define the path to the target field
    let path = vec!["level2", "level3", "target_field"];

    // Create the extractor (this part should succeed)
    let extractor = serde_evaluate::NestedFieldExtractor::new_from_path(&path)
        .expect("Failed to create extractor");

    // Evaluate the extractor (this part will fail until evaluate is implemented)
    let result = extractor.evaluate(&data);
    assert_eq!(result, Ok(FieldScalarValue::I32(999)));
}

#[test]
fn test_extract_deeply_nested_field_comprehensive() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 123,
                another_value: Some("test".to_string()),
                nested_struct: InnerMost { deep_value: true },
            },
            primitive: 999,
        },
        name: "outer_name".to_string(),
    };

    // --- Valid Cases ---
    let extractor = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
        "middle", "inner", "value",
    ])
    .unwrap();
    let result = extractor.evaluate(&data);
    assert_eq!(result.unwrap(), FieldScalarValue::I32(123));

    let extractor_option = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
        "middle",
        "inner",
        "another_value",
    ])
    .unwrap();
    let result_option = extractor_option.evaluate(&data);
    assert_eq!(
        result_option.unwrap(),
        FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String("test".to_string()))))
    );

    let extractor_deepest = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
        "middle",
        "inner",
        "nested_struct",
        "deep_value",
    ])
    .unwrap();
    let result_deepest = extractor_deepest.evaluate(&data);
    assert_eq!(result_deepest.unwrap(), FieldScalarValue::Bool(true));

    // --- Error Cases ---

    // Non-existent intermediate field
    let extractor_bad_intermediate =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
            "middle",
            "non_existent",
            "value",
        ])
        .unwrap();
    let result_bad_intermediate = extractor_bad_intermediate.evaluate(&data);
    assert!(matches!(
        result_bad_intermediate,
        Err(EvaluateError::FieldNotFound { field_name })
        // Note: FieldNotFound error message should ideally reflect the full attempted path
        if field_name == "middle.non_existent.value"
    ));

    // Non-existent final field
    let extractor_bad_final =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
            "middle",
            "inner",
            "non_existent",
        ])
        .unwrap();
    let result_bad_final = extractor_bad_final.evaluate(&data);
    assert!(matches!(
        result_bad_final,
        Err(EvaluateError::FieldNotFound { field_name })
        if field_name == "middle.inner.non_existent"
    ));

    // Attempt to traverse through a non-struct primitive
    let extractor_traverse_primitive =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
            "middle",
            "primitive",
            "should_fail",
        ])
        .unwrap();
    let result_traverse_primitive = extractor_traverse_primitive.evaluate(&data);
    // Expect FieldNotFound because 'primitive' exists but we can't find 'should_fail' within it (as it's not a map/struct)
    assert!(
        matches!(
            result_traverse_primitive,
            Err(EvaluateError::FieldNotFound { ref field_name })
            if field_name == "middle.primitive.should_fail"
        ),
        "Expected FieldNotFound, got {:?}",
        result_traverse_primitive
    );

    // Attempt to extract a non-scalar (struct) as the final value
    let extractor_non_scalar =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
            "middle",
            "inner",
            "nested_struct",
        ])
        .unwrap();
    let result_non_scalar = extractor_non_scalar.evaluate(&data);
    assert!(matches!(
        result_non_scalar,
        Err(EvaluateError::UnsupportedType { .. })
    ));

    // Test path parsing errors handled by new_from_path
    assert!(
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&Vec::<&str>::new())
            .is_err()
    ); // Empty Vec
    assert!(
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&vec![
            "field1", "", "field3"
        ])
        .is_err()
    ); // Empty segment
}
