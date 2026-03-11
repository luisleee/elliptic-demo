use crate::{FieldElement, GFp, Point, U256};

pub struct Curve {
    pub a: FieldElement,
    pub b: FieldElement,
    field: GFp,
}

impl std::fmt::Display for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl std::fmt::Debug for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl Curve {
    pub fn new(a: FieldElement, b: FieldElement, f: GFp) -> Self {
        Curve { a, b, field: f }
    }

    pub fn get_field(&self) -> GFp {
        self.field.clone()
    }

    pub fn show(&self) -> String {
        format!("p: {}\na: {}\nb: {}", self.field.get_p(), self.a, self.b)
    }

    pub fn get_points(&self) -> Vec<Point> {
        let mut points: Vec<Point> = vec![Point::Infinity];

        let p = self.field.get_p();
        let mut i = U256::new();
        let one = U256::from(1u32);

        while i < p {
            let x = self.field.create(&i);
            let s = x * x * x + self.a * x + self.b;

            match s.sqrt() {
                None => {}
                Some(y) => {
                    let point1 = Point::Coordinate { x, y };
                    points.push(point1);

                    if !y.is_zero() {
                        let point2 = Point::Coordinate { x, y: -y };
                        points.push(point2);
                    }
                }
            }

            i = i.add(&one).0;
        }

        points
    }
}

#[cfg(test)]
mod get_points_tests {
    use super::*;

    #[test]
    fn test_get_points_p7() {
        // y^2 = x^3 + x + 1 mod 7
        let p = U256::from(7u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(1u32));
        let b = field.create(&U256::from(1u32));
        let curve = Curve::new(a, b, field);

        let points = curve.get_points();

        assert_eq!(points.len(), 5);

        for point in &points {
            assert!(point.on_curve(&curve));
        }
    }

    #[test]
    fn test_get_points_p11() {
        // y^2 = x^3 + x + 6 mod 11
        let p = U256::from(11u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(1u32));
        let b = field.create(&U256::from(6u32));
        let curve = Curve::new(a, b, field);

        let points = curve.get_points();

        assert_eq!(points.len(), 13, "Should contain 13 points");

        for point in &points {
            assert!(point.on_curve(&curve));
        }
    }

    #[test]
    fn test_get_points_p23() {
        // y^2 = x^3 + x + 1 mod 23
        let p = U256::from(23u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(1u32));
        let b = field.create(&U256::from(1u32));
        let curve = Curve::new(a, b, field);

        let points = curve.get_points();

        assert_eq!(points.len(), 28, "Should contain 28 points");

        // 验证所有点
        for point in &points {
            assert!(point.on_curve(&curve));
        }

        // 检查包含无穷远点
        let has_infinity = points.iter().any(|p| matches!(p, Point::Infinity));
        assert!(has_infinity, "Points should include infinity");

        // 检查没有重复
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for point in &points {
            if let Point::Coordinate { x, y } = point {
                let key = (x.val.0[0], y.val.0[0]);
                assert!(!seen.contains(&key), "Duplicate point: {:?}", key);
                seen.insert(key);
            }
        }
    }

    #[test]
    fn test_get_points_includes_opposites() {
        let p = U256::from(13u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(0u32));
        let b = field.create(&U256::from(7u32));
        let curve = Curve::new(a, b, field);

        let points = curve.get_points();

        // 对于每个点 (x, y)，如果 y != 0，应该有对应的 (x, -y)
        use std::collections::HashMap;
        let mut by_x: HashMap<u32, Vec<u32>> = HashMap::new();

        for point in &points {
            if let Point::Coordinate { x, y } = point {
                by_x.entry(x.val.0[0])
                    .or_insert(Vec::new())
                    .push(y.val.0[0]);
            }
        }

        for (x, ys) in &by_x {
            if ys.len() == 2 {
                // 应该是 y 和 p-y
                let sum = (ys[0] + ys[1]) % 13;
                assert_eq!(sum, 0, "Points at x={} are not opposites: {:?}", x, ys);
            } else if ys.len() == 1 {
                // y = 0 的情况
                assert_eq!(ys[0], 0, "Single point at x={} should have y=0", x);
            }
        }
    }

    #[test]
    fn test_get_points_small_p5() {
        // 最小的测试 p = 5
        // y^2 = x^3 + x + 1 mod 5
        let p = U256::from(5u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(1u32));
        let b = field.create(&U256::from(1u32));
        let curve = Curve::new(a, b, field);

        let points = curve.get_points();

        // 期望的点（不包括无穷远点）
        let expected = vec![
            (0, 1),
            (0, 4),
            (2, 1),
            (2, 4),
            (3, 1),
            (3, 4),
            (4, 3),
            (4, 2),
        ];

        // 验证每个期望的点都在 points 中
        for (x_val, y_val) in expected {
            let found = points.iter().any(|point| {
                if let Point::Coordinate { x, y } = point {
                    x.val.0[0] == x_val && y.val.0[0] == y_val
                } else {
                    false
                }
            });
            assert!(found, "Point ({}, {}) should be in points", x_val, y_val);
        }

        // 验证包含无穷远点
        let has_infinity = points.iter().any(|p| matches!(p, Point::Infinity));
        assert!(has_infinity, "Points should include Infinity");

        // 验证总数
        assert_eq!(points.len(), 9, "Should have exactly 9 points");
    }
}
