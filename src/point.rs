use core::panic;

use crate::{Curve, FieldElement, U256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Point {
    Infinity,
    Coordinate { x: FieldElement, y: FieldElement },
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Infinity => "O".to_string(),
            Self::Coordinate { x, y } => {
                format!("x: {}\ny: {}", x, y)
            }
        };
        write!(f, "{}", s)
    }
}

impl Point {
    pub fn on_curve(&self, curve: &Curve) -> bool {
        match *self {
            Self::Infinity => true,
            Self::Coordinate { x, y } => {
                let p1 = x.get_p();
                let p2 = y.get_p();
                let fp = curve.get_field().get_p();
                if p1 == p2 && p2 == fp {
                    y * y == x * x * x + curve.a * x + curve.b
                } else {
                    panic!("Cannot add elements from different fields");
                }
            }
        }
    }

    pub fn add(&self, other: &Self, curve: &Curve) -> Self {
        if !self.on_curve(curve) || !other.on_curve(curve) {
            panic!("point not on curve!");
        }

        match self {
            Self::Infinity => other.clone(),
            Self::Coordinate { x: x1, y: y1 } => match other {
                Self::Infinity => self.clone(),
                Self::Coordinate { x: x2, y: y2 } => {
                    if x1 == x2 && *y1 == -*y2 {
                        return Self::Infinity;
                    }

                    let lambda = if x1 == x2 && y1 == y2 {
                        // double
                        let f = curve.get_field();
                        let three = f.create(&U256::from(3u32));
                        let two = f.create(&U256::from(2u32));

                        (three * (*x1) * (*x1) + curve.a) / (two * (*y1))
                    } else {
                        // add

                        (*y2 - *y1) / (*x2 - *x1)
                    };
                    let x3 = lambda * lambda - *x1 - *x2;
                    let y3 = lambda * (*x1 - x3) - *y1;
                    Self::Coordinate { x: x3, y: y3 }
                }
            },
        }
    }

    pub fn mul(&self, k: U256, curve: &Curve) -> Self {
        if !self.on_curve(curve) {
            panic!("point not on curve!");
        }

        let mut result = Self::Infinity;
        let mut pt = self.clone();
        let mut k = k;

        while !k.is_zero() {
            if !k.and(&U256::from(1u32)).is_zero() {
                result = result.add(&pt, curve);
            }

            pt = pt.add(&pt, curve);
            k = k.shr(1);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{GFp, U256};

    use super::*;

    #[test]
    fn test_on_curve_basic() {
        // 使用小素数 p = 17
        let p = U256::from(17u32);
        let field = GFp::new(&p);

        // 曲线 y^2 = x^3 + 2x + 2 mod 17
        let a = field.create(&U256::from(2u32));
        let b = field.create(&U256::from(2u32));
        let curve = Curve::new(a, b, field);

        // 无穷远点
        assert!(Point::Infinity.on_curve(&curve));

        // 有效点 (5, 1): 1^2 = 1, 5^3 + 2*5 + 2 = 137 = 1 mod 17
        let x = field.create(&U256::from(5u32));
        let y = field.create(&U256::from(1u32));
        let valid_point = Point::Coordinate { x, y };
        assert!(valid_point.on_curve(&curve));

        // 无效点 (1, 1): 1^2 = 1, 1^3 + 2*1 + 2 = 5 ≠ 1 mod 17
        let x = field.create(&U256::from(1u32));
        let y = field.create(&U256::from(1u32));
        let invalid_point = Point::Coordinate { x, y };
        assert!(!invalid_point.on_curve(&curve));
    }

    #[test]
    fn test_point_add_and_mul() {
        // 取小素数域 p = 17
        let p = U256::from(17u32);
        let field = GFp::new(&p);
        let a = field.create(&U256::from(2u32));
        let b = field.create(&U256::from(2u32));
        let curve = Curve::new(a, b, field.clone());

        // 点 P = (5, 1)，验证在曲线上
        let x = field.create(&U256::from(5u32));
        let y = field.create(&U256::from(1u32));
        let p1 = Point::Coordinate {
            x: x.clone(),
            y: y.clone(),
        };
        assert!(p1.on_curve(&curve));

        // 测试 P + Infinity = P
        assert_eq!(p1.add(&Point::Infinity, &curve), p1);
        assert_eq!(Point::Infinity.add(&p1, &curve), p1);

        // 计算 2P = P + P
        let p2 = p1.add(&p1, &curve);
        assert!(p2.on_curve(&curve));

        // 计算 3P = 2P + P
        let p3 = p2.add(&p1, &curve);
        assert!(p3.on_curve(&curve));

        // 数乘测试： k = 3
        let p3_mul = p1.mul(U256::from(3u32), &curve);
        assert_eq!(p3, p3_mul, "3P from add and mul should be equal");

        // 数乘 k = 0 => Infinity
        let inf = p1.mul(U256::from(0u32), &curve);
        assert!(matches!(inf, Point::Infinity));

        // 数乘 k = 1 => 本身
        let p1_mul = p1.mul(U256::from(1u32), &curve);
        assert_eq!(p1, p1_mul);

        // 数乘 k = 4
        let p4 = p1.mul(U256::from(4u32), &curve);
        assert!(p4.on_curve(&curve));
    }

    #[test]
    fn test_nist_p256_point_add_and_mul() {
        // 构造NIST P-256定义的p
        let p = U256([
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
            0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
        ]);

        let field = GFp::new(&p);

        // a 值（p256标准的a）
        let a_val = U256([
            0xFFFFFFFC, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
            0x00000000, 0x00000000, 0x00000001, 0xFFFFFFFF,
        ]);
        let a = field.create(&a_val);

        // b 值
        let b_val = U256([
            0x27D2604B, 0x3BCE3C3E, 0xCC53B0F6, 0x651D06B0, 
            0x769886BC, 0xB3EBBD55, 0xAA3A93E7, 0x5AC635D8
        ]);
        let b = field.create(&b_val);

        let curve = Curve::new(a, b, field.clone());

        // G 点x坐标
        let gx_val = U256([
            0xD898C296, 0xF4A13945, 0x2DEB33A0, 0x77037D81,
            0x63A440F2, 0xF8BCE6E5, 0xE12C4247, 0x6B17D1F2,
        ]);
        let gx = field.create(&gx_val);

        // G 点y坐标
        let gy_val = U256([
            0x37BF51F5, 0xCBB64068, 0x6B315ECE, 0x2BCE3357,
            0x7C0F9E16, 0x8EE7EB4A, 0xFE1A7F9B, 0x4FE342E2,
        ]);
        let gy = field.create(&gy_val);

        let g = Point::Coordinate { x: gx, y: gy };

        assert!(g.on_curve(&curve), "G should be on the curve");

        // 测试 2G = G + G
        let g2 = g.add(&g, &curve);
        assert!(g2.on_curve(&curve));

        // 测试数乘 kG，k=3
        let g3 = g.mul(U256::from(3u32), &curve);
        let g3_expected = g2.add(&g, &curve);
        assert_eq!(g3, g3_expected);

        // 测试数乘 kG，k=0，应该是无穷远点
        let inf = g.mul(U256::from(0u32), &curve);
        assert!(matches!(inf, Point::Infinity));

        let n = U256([
            0xFC632551, 0xF3B9CAC2, 0xA7179E84, 0xBCE6FAAD,
            0xFFFFFFFF, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF
        ]);
        let inf = g.mul(n, &curve);
        assert!(matches!(inf, Point::Infinity), "nG should be O");
    }

}
