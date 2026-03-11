use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use elliptic::{U256, GFp};

fn gen_test_values() -> Vec<(String, U256)> {
    vec![
        ("Small".to_string(), U256::from(4294836225u32)),
        ("Medium".to_string(), U256::from_hex_str("0xD5E9AE268C2992B9C3B3C7A3A9C5B7D4").unwrap()),
        ("Large".to_string(), U256([
            0xFFFFFFFC, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
            0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
        ])),
    ]
}

fn get_prime() -> U256 {
    U256([
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
    ])
}

fn benchmark_gfp(c: &mut Criterion) {
    let p = get_prime();
    let gfp = GFp::new(&p);
    let test_values = gen_test_values();

    let mut group = c.benchmark_group("U256 Modular Arithmetic");
    for (label, val) in &test_values {
        let val_mod = val.modulo(&p);

        let fe1 = gfp.create(&val_mod);
        let fe2 = gfp.create(&val_mod.add_mod(&U256::from(1u32), &p));

        group.throughput(Throughput::Bytes(32));

        // add_mod benchmarks
        group.bench_with_input(BenchmarkId::new("add_mod", label), &val_mod, |b, v| {
            b.iter(|| v.add_mod(v, &p));
        });

        // mul_mod benchmarks
        group.bench_with_input(BenchmarkId::new("mul_mod", label), &val_mod, |b, v| {
            b.iter(|| v.mul_mod(v, &p));
        });

        // exp_mod benchmarks
        let base = U256::from(65537u32);
        group.bench_with_input(BenchmarkId::new("exp_mod", label), &val_mod, |b, v| {
            b.iter(|| base.exp_mod(&v, &p));
        });

        // FieldElement division benchmark
        group.bench_with_input(
            BenchmarkId::new("FieldElement Div", label),
            &(fe1, fe2),
            |b, (x, y)| {
                b.iter(|| {
                    let _ = (*x).clone() / (*y).clone();
                });
            },
        );

        // FieldElement sqrt benchmark
        group.bench_with_input(BenchmarkId::new("FieldElement Sqrt", label), &fe1, |b, x| {
            b.iter(|| {
                let _ = x.sqrt();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_gfp);
criterion_main!(benches);
