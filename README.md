# Serde Evaluate

[![Crates.io](https://img.shields.io/crates/v/serde_evaluate.svg)](https://crates.io/crates/serde_evaluate)
[![Docs.rs](https://docs.rs/serde_evaluate/badge.svg)](https://docs.rs/serde_evaluate)

Extract single scalar field values from Serializable structs without full deserialization.

## Overview

This library provides a mechanism to extract the value of a single field
from any struct that implements `serde::Serialize` **without** needing to
deserialize the entire struct. The extraction happens at **runtime** by
intercepting the serialization process.
It's particularly useful when you only need one specific piece of data
from a potentially large or complex structure, potentially residing
within nested structs or maps.

The extracted value is returned as a `FieldScalarValue` enum, which covers
common scalar types (integers, floats, bool, string, char, bytes, unit, and options of these).

## Basic Usage

For extracting simple top-level fields, use `FieldExtractor`:

```rust
use serde::Serialize;
use serde_evaluate::{extractor::FieldExtractor, value::FieldScalarValue, EvaluateError};

#[derive(Serialize)]
struct UserProfile {
    user_id: u64,
    username: String,
    is_active: bool,
}

fn main() -> Result<(), EvaluateError> {
    let profile = UserProfile {
        user_id: 9876,
        username: "tester".to_string(),
        is_active: true,
    };

    // Extract the 'username' field (top-level)
    let extractor = FieldExtractor::new("username");
    let username_value = extractor.evaluate(&profile)?;
    assert_eq!(username_value, FieldScalarValue::String("tester".to_string()));

    // Extract the 'is_active' field (top-level)
    let active_extractor = FieldExtractor::new("is_active");
    let active_value = active_extractor.evaluate(&profile)?;
    assert_eq!(active_value, FieldScalarValue::Bool(true));

    Ok(())
}
```

For nested fields (within structs or maps), use `NestedFieldExtractor`. Here's an example demonstrating how to extract a nested field from a `HashMap`:

```rust
use serde::Serialize;
use std::collections::HashMap;
use serde_evaluate::{extractor::NestedFieldExtractor, value::FieldScalarValue, EvaluateError};

#[derive(Serialize)]
struct Config {
    port: u16,
    settings: HashMap<String, Detail>,
}

#[derive(Serialize)]
struct Detail {
    enabled: bool,
    level: String,
}

fn main() -> Result<(), EvaluateError> {
    let mut settings_map = HashMap::new();
    settings_map.insert("feature_x".to_string(), Detail { enabled: true, level: "debug".to_string() });
    settings_map.insert("feature_y".to_string(), Detail { enabled: false, level: "info".to_string() });

    let config = Config {
        port: 8080,
        settings: settings_map,
    };

    // Extract 'settings[feature_x].level'
    // The path components are: "settings", "feature_x", "level"
    let extractor = NestedFieldExtractor::new_from_path(&["settings", "feature_x", "level"])?;
    let level_value = extractor.evaluate(&config)?;
    assert_eq!(level_value, FieldScalarValue::String("debug".to_string()));

    // Extract 'settings[feature_y].enabled'
    // The path components are: "settings", "feature_y", "enabled"
    let extractor_enabled = NestedFieldExtractor::new_from_path(&["settings", "feature_y", "enabled"])?;
    let enabled_value = extractor_enabled.evaluate(&config)?;
    assert_eq!(enabled_value, FieldScalarValue::Bool(false));

    Ok(())
}