//! `DoS` Protection Tests
//!
//! These tests verify that the deserializer properly enforces limits to prevent
//! denial of service attacks via maliciously crafted CBOR data.

use multi_cbor::config::DeserializerConfig;
use multi_cbor::de::Deserializer;
use multi_cbor::error::Category;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Helper function to create CBOR-encoded array with specified number of elements
fn create_cbor_array(size: usize) -> Vec<u8> {
    let arr: Vec<u32> = (0..u32::try_from(size).unwrap()).collect();
    multi_cbor::to_vec(&arr).unwrap()
}

/// Helper function to create CBOR-encoded map with specified number of entries
fn create_cbor_map(size: usize) -> Vec<u8> {
    let map: HashMap<String, u32> = (0..u32::try_from(size).unwrap())
        .map(|i| (format!("key{i}"), i))
        .collect();
    multi_cbor::to_vec(&map).unwrap()
}

/// Helper function to create indefinite-length CBOR array
fn create_indefinite_array(size: usize) -> Vec<u8> {
    let mut result = vec![0x9f]; // Indefinite array start
    for i in 0..u32::try_from(size).unwrap() {
        // Append each element
        let elem_bytes = multi_cbor::to_vec(&i).unwrap();
        result.extend_from_slice(&elem_bytes);
    }
    result.push(0xff); // Break marker
    result
}

/// Helper function to create indefinite-length CBOR map
fn create_indefinite_map(size: usize) -> Vec<u8> {
    let mut result = vec![0xbf]; // Indefinite map start
    for i in 0..u32::try_from(size).unwrap() {
        // Append key
        let key = format!("k{i}");
        let key_bytes = multi_cbor::to_vec(&key).unwrap();
        result.extend_from_slice(&key_bytes);
        // Append value
        let val_bytes = multi_cbor::to_vec(&i).unwrap();
        result.extend_from_slice(&val_bytes);
    }
    result.push(0xff); // Break marker
    result
}

#[test]
fn test_array_size_limit_not_exceeded() {
    let config = DeserializerConfig::strict().max_array_size(100);
    let cbor = create_cbor_array(50);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let arr = result.unwrap();
    assert_eq!(arr.len(), 50);
}

#[test]
fn test_array_size_limit_exceeded() {
    let config = DeserializerConfig::strict().max_array_size(100);
    let cbor = create_cbor_array(1000);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.classify(), Category::Syntax);
    assert!(err.to_string().contains("array size limit exceeded"));
}

#[test]
fn test_array_size_exactly_at_limit() {
    let config = DeserializerConfig::default().max_array_size(100);
    let cbor = create_cbor_array(100);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let arr = result.unwrap();
    assert_eq!(arr.len(), 100);
}

#[test]
fn test_array_size_one_over_limit() {
    let config = DeserializerConfig::default().max_array_size(100);
    let cbor = create_cbor_array(101);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("array size limit exceeded"));
}

#[test]
fn test_map_size_limit_not_exceeded() {
    let config = DeserializerConfig::strict().max_map_size(100);
    let cbor = create_cbor_map(50);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<HashMap<String, u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let map = result.unwrap();
    assert_eq!(map.len(), 50);
}

#[test]
fn test_map_size_limit_exceeded() {
    let config = DeserializerConfig::strict().max_map_size(100);
    let cbor = create_cbor_map(1000);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<HashMap<String, u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.classify(), Category::Syntax);
    assert!(err.to_string().contains("map size limit exceeded"));
}

#[test]
fn test_map_size_exactly_at_limit() {
    let config = DeserializerConfig::default().max_map_size(100);
    let cbor = create_cbor_map(100);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<HashMap<String, u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let map = result.unwrap();
    assert_eq!(map.len(), 100);
}

#[test]
fn test_indefinite_array_limit_not_exceeded() {
    let config = DeserializerConfig::strict().max_indefinite_iterations(100);
    let cbor = create_indefinite_array(50);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let arr = result.unwrap();
    assert_eq!(arr.len(), 50);
}

#[test]
fn test_indefinite_array_limit_exceeded() {
    let config = DeserializerConfig::strict().max_indefinite_iterations(100);
    let cbor = create_indefinite_array(1000);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.classify(), Category::Syntax);
    assert!(err
        .to_string()
        .contains("indefinite-length iteration limit exceeded"));
}

#[test]
fn test_indefinite_map_limit_not_exceeded() {
    let config = DeserializerConfig::strict().max_indefinite_iterations(100);
    let cbor = create_indefinite_map(50);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<HashMap<String, u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let map = result.unwrap();
    assert_eq!(map.len(), 50);
}

#[test]
fn test_indefinite_map_limit_exceeded() {
    let config = DeserializerConfig::strict().max_indefinite_iterations(100);
    let cbor = create_indefinite_map(1000);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<HashMap<String, u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.classify(), Category::Syntax);
    assert!(err
        .to_string()
        .contains("indefinite-length iteration limit exceeded"));
}

#[test]
fn test_recursion_depth_limit() {
    // Create deeply nested array: [[[[...]]]]
    let config = DeserializerConfig::default().max_recursion_depth(32);

    let mut cbor = Vec::new();
    // Create 50 levels of nesting (should exceed limit of 32)
    cbor.extend(std::iter::repeat_n(0x81u8, 50)); // Array of length 1
    cbor.push(0x00); // Final value: 0

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<multi_cbor::Value, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("recursion limit exceeded"));
}

#[test]
fn test_recursion_depth_within_limit() {
    // Create moderately nested array
    let config = DeserializerConfig::default().max_recursion_depth(32);

    let mut cbor = Vec::new();
    // Create 20 levels of nesting (within limit of 32)
    cbor.extend(std::iter::repeat_n(0x81u8, 20)); // Array of length 1
    cbor.push(0x00); // Final value: 0

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<multi_cbor::Value, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
}

#[test]
fn test_unlimited_config_allows_large_arrays() {
    let config = DeserializerConfig::unlimited();
    let cbor = create_cbor_array(10000);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_ok());
    let arr = result.unwrap();
    assert_eq!(arr.len(), 10000);
}

#[test]
fn test_default_config_has_reasonable_limits() {
    let config = DeserializerConfig::default();

    // Should accept reasonably large collections
    assert_eq!(config.get_max_array_size(), Some(100_000));
    assert_eq!(config.get_max_map_size(), Some(100_000));
    assert_eq!(config.get_max_recursion_depth(), 128);
    assert_eq!(config.get_max_indefinite_iterations(), Some(100_000));
}

#[test]
fn test_strict_config_has_tight_limits() {
    let config = DeserializerConfig::strict();

    // Should have conservative limits
    assert_eq!(config.get_max_array_size(), Some(1_000));
    assert_eq!(config.get_max_map_size(), Some(1_000));
    assert_eq!(config.get_max_recursion_depth(), 32);
    assert_eq!(config.get_max_indefinite_iterations(), Some(1_000));
}

#[test]
fn test_nested_arrays_respect_individual_limits() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Container {
        arrays: Vec<Vec<u32>>,
    }

    // Test that nested arrays each respect the size limit independently
    let config = DeserializerConfig::default().max_array_size(100);

    // Create array of arrays: [[0,1,2,...,99], [0,1,2,...,99]]
    let data = Container {
        arrays: vec![(0..99).collect(), (0..99).collect()],
    };

    let cbor = multi_cbor::to_vec(&data).unwrap();
    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Container, _> = serde::Deserialize::deserialize(&mut de);

    // Should succeed - each individual array is within limit
    assert!(result.is_ok());
}

#[test]
fn test_nested_array_exceeds_limit() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Container {
        arrays: Vec<Vec<u32>>,
    }

    let config = DeserializerConfig::default().max_array_size(100);

    let data = Container {
        arrays: vec![
            (0..150).collect(), // This inner array exceeds limit
        ],
    };

    let cbor = multi_cbor::to_vec(&data).unwrap();
    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Container, _> = serde::Deserialize::deserialize(&mut de);

    // Should fail - inner array exceeds limit
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("array size limit exceeded"));
}

#[test]
fn test_custom_config_builder() {
    let config = DeserializerConfig::default()
        .max_array_size(500)
        .max_map_size(250)
        .max_recursion_depth(64)
        .max_indefinite_iterations(1000);

    assert_eq!(config.get_max_array_size(), Some(500));
    assert_eq!(config.get_max_map_size(), Some(250));
    assert_eq!(config.get_max_recursion_depth(), 64);
    assert_eq!(config.get_max_indefinite_iterations(), Some(1000));
}

#[test]
fn test_error_offset_reported_correctly() {
    let config = DeserializerConfig::strict().max_array_size(10);

    // Create CBOR with a large array
    let cbor = create_cbor_array(100);

    let mut de = Deserializer::from_slice(&cbor).config(config);
    let result: Result<Vec<u32>, _> = serde::Deserialize::deserialize(&mut de);

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Error should have an offset
    assert!(err.offset() > 0);
}
