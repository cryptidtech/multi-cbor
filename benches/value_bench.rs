use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use multi_cbor::Value;
use std::collections::BTreeMap;
use std::hint::black_box;

fn value_serialization(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    map.insert(
        Value::Text("name".to_string()),
        Value::Text("John".to_string()),
    );
    map.insert(Value::Text("age".to_string()), Value::Integer(30));
    map.insert(Value::Text("active".to_string()), Value::Bool(true));
    let value = Value::Map(map);

    let mut group = c.benchmark_group("value");
    group.throughput(Throughput::Elements(1));

    group.bench_function("serialize", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&value)).unwrap());
    });

    group.finish();
}

fn value_deserialization(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    map.insert(
        Value::Text("name".to_string()),
        Value::Text("John".to_string()),
    );
    map.insert(Value::Text("age".to_string()), Value::Integer(30));
    map.insert(Value::Text("active".to_string()), Value::Bool(true));
    let value = Value::Map(map);
    let encoded = multi_cbor::to_vec(&value).unwrap();

    let mut group = c.benchmark_group("value");
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("deserialize", |b| {
        b.iter(|| multi_cbor::from_slice::<Value>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

fn value_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("value");
    group.throughput(Throughput::Elements(1));

    group.bench_function("construct_integer", |b| {
        b.iter(|| Value::Integer(black_box(42)));
    });

    group.bench_function("construct_text", |b| {
        b.iter(|| Value::Text(black_box("hello".to_string())));
    });

    group.bench_function("construct_array", |b| {
        b.iter(|| {
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        });
    });

    group.bench_function("construct_map", |b| {
        b.iter(|| {
            let mut map = BTreeMap::new();
            map.insert(Value::Text("key".to_string()), Value::Integer(42));
            Value::Map(map)
        });
    });

    group.finish();
}

fn large_value_operations(c: &mut Criterion) {
    // Create a large nested structure
    let mut large_map = BTreeMap::new();
    for i in 0..100 {
        let mut inner_map = BTreeMap::new();
        for j in 0..10 {
            inner_map.insert(Value::Text(format!("key{j}")), Value::Integer(i * 10 + j));
        }
        large_map.insert(Value::Text(format!("outer{i}")), Value::Map(inner_map));
    }
    let large_value = Value::Map(large_map);

    let mut group = c.benchmark_group("value");

    group.bench_function("large_value_serialize", |b| {
        b.iter(|| multi_cbor::to_vec(black_box(&large_value)).unwrap());
    });

    let encoded = multi_cbor::to_vec(&large_value).unwrap();
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("large_value_deserialize", |b| {
        b.iter(|| multi_cbor::from_slice::<Value>(black_box(&encoded)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    value_serialization,
    value_deserialization,
    value_construction,
    large_value_operations,
);
criterion_main!(benches);
