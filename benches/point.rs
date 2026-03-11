use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use elliptic::{Curve, Point, U256, GFp};

fn prepare_nist_p256() -> (Curve, Point) {
    let p = U256([
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
    ]);

    let field = GFp::new(&p);

    let a_val = U256([
        0xFFFFFFFC, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
    ]);
    let a = field.create(&a_val);

    let b_val = U256([
        0x27D2604B, 0x3BCE3C3E, 0xCC53B0F6, 0x651D06B0, 
        0x769886BC, 0xB3EBBD55, 0xAA3A93E7, 0x5AC635D8
    ]);
    let b = field.create(&b_val);

    let curve = Curve::new(a, b, field.clone());

    let gx_val = U256([
        0xD898C296, 0xF4A13945, 0x2DEB33A0, 0x77037D81,
        0x63A440F2, 0xF8BCE6E5, 0xE12C4247, 0x6B17D1F2,
    ]);
    let gx = field.create(&gx_val);

    let gy_val = U256([
        0x37BF51F5, 0xCBB64068, 0x6B315ECE, 0x2BCE3357,
        0x7C0F9E16, 0x8EE7EB4A, 0xFE1A7F9B, 0x4FE342E2,
    ]);
    let gy = field.create(&gy_val);

    let g = Point::Coordinate { x: gx, y: gy };

    (curve, g)
}

fn bench_point_mul(c: &mut Criterion) {
    let (curve, g) = prepare_nist_p256();

    // 选择不同大小的标量 k，测试点乘耗时差异
    let test_scalars = vec![
        ("Small", U256::from(12345u32)),
        ("Medium", U256([
            0xFFFFFFFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
        ])),
        ("Large", U256([
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        ])),
    ];

    let mut group = c.benchmark_group("Elliptic Curve Point Multiplication");

    for (label, k) in test_scalars {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("mul", label), &k, |b, k| {
            b.iter(|| {
                let _res = g.mul(*k, &curve);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_point_mul);
criterion_main!(benches);
