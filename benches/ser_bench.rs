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

fn small_struct_serialization(c: &mut Criterion) {
    let data = SmallStruct {
        id: 42,
        name: "test".to_string(),
        active: true,
    };

    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1));

    group.bench_function("small_struct", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&data)).unwrap());
    });

    group.finish();
}

fn medium_struct_serialization(c: &mut Criterion) {
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

    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1));

    group.bench_function("medium_struct", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&data)).unwrap());
    });

    group.finish();
}

fn large_array_serialization(c: &mut Criterion) {
    let data: Vec<u64> = (0..1000).collect();

    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("large_array", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&data)).unwrap());
    });

    group.finish();
}

fn integer_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1));

    group.bench_function("u8", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&42u8)).unwrap());
    });
    group.bench_function("u32", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&42u32)).unwrap());
    });
    group.bench_function("u64", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&42u64)).unwrap());
    });
    group.bench_function("i64", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&-42i64)).unwrap());
    });

    group.finish();
}

fn string_serialization(c: &mut Criterion) {
    let short_str = "hello";
    let medium_str = "The quick brown fox jumps over the lazy dog";
    let long_str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";

    let mut group = c.benchmark_group("serialization");

    group.throughput(Throughput::Bytes(short_str.len() as u64));
    group.bench_function("string_short", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&short_str)).unwrap());
    });

    group.throughput(Throughput::Bytes(medium_str.len() as u64));
    group.bench_function("string_medium", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&medium_str)).unwrap());
    });

    group.throughput(Throughput::Bytes(long_str.len() as u64));
    group.bench_function("string_long", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&long_str)).unwrap());
    });

    group.finish();
}

fn packed_format_serialization(c: &mut Criterion) {
    let data = MediumStruct {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: 30,
        tags: vec!["rust".to_string(), "cbor".to_string()],
        metadata: BTreeMap::new(),
    };

    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Elements(1));

    group.bench_function("packed_format", |b| {
        b.iter(|| {
            use multi_cbor::ser::{IoWrite, Serializer};
            let mut buf = Vec::new();
            let mut ser = Serializer::new(IoWrite::new(&mut buf)).packed_format();
            black_box(&data).serialize(&mut ser).unwrap();
            buf
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    small_struct_serialization,
    medium_struct_serialization,
    large_array_serialization,
    integer_serialization,
    string_serialization,
    packed_format_serialization,
);
criterion_main!(benches);
