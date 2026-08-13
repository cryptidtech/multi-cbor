use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hint::black_box;

#[derive(Debug, Serialize, Deserialize)]
struct SmallStruct {
    id: u32,
    name: String,
    active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MediumStruct {
    id: u64,
    name: String,
    email: String,
    age: u32,
    tags: Vec<String>,
    metadata: BTreeMap<String, String>,
}

fn small_struct_deserialization(c: &mut Criterion) {
    let data = SmallStruct {
        id: 42,
        name: "test".to_string(),
        active: true,
    };
    let encoded = multi_cbor::to_vec(&data).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("small_struct", |b| {
        b.iter(|| multi_cbor::from_slice::<SmallStruct>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

fn medium_struct_deserialization(c: &mut Criterion) {
    let mut metadata = BTreeMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());

    let data = MediumStruct {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
        tags: vec!["rust".to_string(), "cbor".to_string(), "serde".to_string()],
        metadata,
    };
    let encoded = multi_cbor::to_vec(&data).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("medium_struct", |b| {
        b.iter(|| multi_cbor::from_slice::<MediumStruct>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

fn large_array_deserialization(c: &mut Criterion) {
    let data: Vec<u64> = (0..1000).collect();
    let encoded = multi_cbor::to_vec(&data).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("large_array", |b| {
        b.iter(|| multi_cbor::from_slice::<Vec<u64>>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

fn integer_deserialization(c: &mut Criterion) {
    let u8_data = multi_cbor::to_vec(&42u8).unwrap();
    let u32_data = multi_cbor::to_vec(&42u32).unwrap();
    let u64_data = multi_cbor::to_vec(&42u64).unwrap();
    let i64_data = multi_cbor::to_vec(&-42i64).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Elements(1));

    group.bench_function("u8", |b| {
        b.iter(|| multi_cbor::from_slice::<u8>(black_box(&u8_data)).unwrap());
    });
    group.bench_function("u32", |b| {
        b.iter(|| multi_cbor::from_slice::<u32>(black_box(&u32_data)).unwrap());
    });
    group.bench_function("u64", |b| {
        b.iter(|| multi_cbor::from_slice::<u64>(black_box(&u64_data)).unwrap());
    });
    group.bench_function("i64", |b| {
        b.iter(|| multi_cbor::from_slice::<i64>(black_box(&i64_data)).unwrap());
    });

    group.finish();
}

fn string_deserialization(c: &mut Criterion) {
    let short_str = "hello";
    let medium_str = "The quick brown fox jumps over the lazy dog";
    let long_str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";

    let short_data = multi_cbor::to_vec(&short_str).unwrap();
    let medium_data = multi_cbor::to_vec(&medium_str).unwrap();
    let long_data = multi_cbor::to_vec(&long_str).unwrap();

    let mut group = c.benchmark_group("deserialization");

    group.throughput(Throughput::Bytes(short_data.len() as u64));
    group.bench_function("string_short", |b| {
        b.iter(|| multi_cbor::from_slice::<&str>(black_box(&short_data)).unwrap());
    });

    group.throughput(Throughput::Bytes(medium_data.len() as u64));
    group.bench_function("string_medium", |b| {
        b.iter(|| multi_cbor::from_slice::<&str>(black_box(&medium_data)).unwrap());
    });

    group.throughput(Throughput::Bytes(long_data.len() as u64));
    group.bench_function("string_long", |b| {
        b.iter(|| multi_cbor::from_slice::<&str>(black_box(&long_data)).unwrap());
    });

    group.finish();
}

fn zero_copy_deserialization(c: &mut Criterion) {
    let data = "Hello, zero-copy world!";
    let encoded = multi_cbor::to_vec(&data).unwrap();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("zero_copy_str", |b| {
        b.iter(|| multi_cbor::from_slice::<&str>(black_box(&encoded)).unwrap());
    });

    group.bench_function("owned_string", |b| {
        b.iter(|| multi_cbor::from_slice::<String>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    small_struct_deserialization,
    medium_struct_deserialization,
    large_array_deserialization,
    integer_deserialization,
    string_deserialization,
    zero_copy_deserialization,
);
criterion_main!(benches);
