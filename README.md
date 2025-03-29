# serde-evaluate

Extract single scalar field values from Serializeable structs without full deserialization.

## Usage Example

```rust
use serde::Serialize;
use serde_evaluate::{FieldExtractor, FieldScalarValue, EvaluateError};

#[derive(Serialize)]
struct MyData {
    id: u32,
    name: String,
    active: bool,
    score: Option<f64>,
    #[serde(with = "serde_bytes")]
    raw_data: Vec<u8>,
    nested: InnerData, // Unsupported type for direct extraction
}

#[derive(Serialize)]
struct InnerData {
    value: i32,
}

fn main() -> Result<(), EvaluateError> {
    let data = MyData {
        id: 101,
        name: "Example".to_string(),
        active: true,
        score: Some(95.5),
        raw_data: vec![1, 2, 3, 4],
        nested: InnerData { value: -5 },
    };

    // Extract the 'name' field
    let name_value = FieldExtractor::new("name").evaluate(&data)?;
    assert_eq!(name_value, FieldScalarValue::String("Example".to_string()));

    // Extract the 'active' field
    let active_value = FieldExtractor::new("active").evaluate(&data)?;
    assert_eq!(active_value, FieldScalarValue::Bool(true));

    // Extract the 'score' field (Option<f64>)
    let score_value = FieldExtractor::new("score").evaluate(&data)?;
    assert_eq!(score_value, FieldScalarValue::Option(Some(Box::new(FieldScalarValue::F64(95.5)))));

    // Extract the 'raw_data' field (Vec<u8> handled via serde_bytes)
    let bytes_value = FieldExtractor::new("raw_data").evaluate(&data)?;
    assert_eq!(bytes_value, FieldScalarValue::Bytes(vec![1, 2, 3, 4]));

    // Trying to extract a non-existent field returns FieldNotFound
    let missing_result = FieldExtractor::new("address").evaluate(&data);
    assert!(matches!(missing_result, Err(EvaluateError::FieldNotFound { .. })));

    // Trying to extract a non-scalar field (struct) returns UnsupportedType
    let nested_result = FieldExtractor::new("nested").evaluate(&data);
    assert!(matches!(nested_result, Err(EvaluateError::UnsupportedType { type_name }) if type_name == "struct"));

    Ok(())
}
```

See the [crate documentation](https://docs.rs/serde_evaluate) for more details and the full API.

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.