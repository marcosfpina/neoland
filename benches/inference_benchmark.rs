// NEOLAND Performance Benchmarks
// Criterion-based benchmarks for critical paths

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

// Note: These benchmarks require the actual NEOLAND code to be available
// They are placeholders showing the structure

fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    // Small payload
    let small_json = r#"{"messages": [{"role": "user", "content": "Hello"}]}"#;
    group.bench_with_input(
        BenchmarkId::new("small_payload", small_json.len()),
        &small_json,
        |b, json| {
            b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(json).unwrap()));
        },
    );

    // Medium payload
    let medium_json =
        format!(r#"{{"messages": [{{"role": "user", "content": "{}"}}]}}"#, "A".repeat(1000));
    group.bench_with_input(
        BenchmarkId::new("medium_payload", medium_json.len()),
        &medium_json,
        |b, json| {
            b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(json).unwrap()));
        },
    );

    // Large payload
    let large_json =
        format!(r#"{{"messages": [{{"role": "user", "content": "{}"}}]}}"#, "A".repeat(10000));
    group.bench_with_input(
        BenchmarkId::new("large_payload", large_json.len()),
        &large_json,
        |b, json| {
            b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(json).unwrap()));
        },
    );

    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // String concatenation
    group.bench_function("concat_small", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..100 {
                s.push_str(black_box(&format!("item_{}", i)));
            }
            black_box(s)
        });
    });

    // String formatting
    group.bench_function("format_macro", |b| {
        b.iter(|| {
            let result =
                format!("User {} sent message: {}", black_box("alice"), black_box("Hello, world!"));
            black_box(result)
        });
    });

    group.finish();
}

fn benchmark_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    // Vec allocation and push
    group.bench_function("vec_push_1000", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..1000 {
                v.push(black_box(i));
            }
            black_box(v)
        });
    });

    // Vec with capacity
    group.bench_function("vec_with_capacity_1000", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(1000);
            for i in 0..1000 {
                v.push(black_box(i));
            }
            black_box(v)
        });
    });

    // Vec iteration
    let data: Vec<i32> = (0..10000).collect();
    group.bench_function("vec_iter_sum", |b| {
        b.iter(|| {
            let sum: i32 = data.iter().sum();
            black_box(sum)
        });
    });

    group.finish();
}

fn benchmark_hash_operations(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("hash_operations");

    // HashMap insertion
    group.bench_function("hashmap_insert_1000", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..1000 {
                map.insert(black_box(i), black_box(i * 2));
            }
            black_box(map)
        });
    });

    // HashMap lookup
    let mut map = HashMap::new();
    for i in 0..10000 {
        map.insert(i, i * 2);
    }
    group.bench_function("hashmap_lookup", |b| {
        b.iter(|| {
            let value = map.get(&black_box(5000));
            black_box(value)
        });
    });

    group.finish();
}

fn benchmark_async_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_operations");
    group.measurement_time(Duration::from_secs(10));

    // Async task spawning
    group.bench_function("tokio_spawn_100", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let mut handles = Vec::new();
                for _ in 0..100 {
                    handles.push(tokio::spawn(async { black_box(42) }));
                }
                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            })
        });
    });

    // Channel operations
    group.bench_function("tokio_channel_1000", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let (tx, mut rx) = tokio::sync::mpsc::channel(1000);

                tokio::spawn(async move {
                    for i in 0..1000 {
                        tx.send(black_box(i)).await.unwrap();
                    }
                });

                let mut count = 0;
                while rx.recv().await.is_some() {
                    count += 1;
                    if count == 1000 {
                        break;
                    }
                }
                black_box(count)
            })
        });
    });

    group.finish();
}

fn benchmark_http_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_client");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10); // Fewer samples for network calls

    // Note: This requires a running server
    // Commented out to avoid failing when server isn't running
    /*
    group.bench_function("health_check", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = reqwest::Client::new();

        b.to_async(&runtime).iter(|| async {
            let response = client
                .get("http://localhost:3001/health")
                .send()
                .await;
            black_box(response)
        });
    });
    */

    group.finish();
}

criterion_group!(
    benches,
    benchmark_json_parsing,
    benchmark_string_operations,
    benchmark_vector_operations,
    benchmark_hash_operations,
    benchmark_async_operations,
    benchmark_http_client,
);

criterion_main!(benches);
