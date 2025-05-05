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