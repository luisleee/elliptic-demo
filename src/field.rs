use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::U256;

// need to ensure p is prime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GFp {
    p: U256,
}

impl GFp {
    pub fn new(p: &U256) -> Self {
        GFp { p: p.clone() }
    }

    pub fn create(&self, val: &U256) -> FieldElement {
        FieldElement {
            val: val.modulo(&self.p),
            p: self.p.clone(),
        }
    }

    pub fn get_p(&self) -> U256 {
        self.p.clone()
    }

}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FieldElement {
    pub val: U256,
    p: U256,
}

impl std::fmt::Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl std::fmt::Debug for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl Add for FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: Self) -> Self::Output {
        if self.p != rhs.p {
            panic!("Cannot add elements from different fields");
        }

        FieldElement {
            val: self.val.add_mod(&rhs.val, &self.p),
            p: self.p.clone(),
        }
    }
}

impl Neg for FieldElement {
    type Output = FieldElement;
    fn neg(self) -> Self::Output {
        FieldElement {
            val: self.p.sub(&self.val).0,
            p: self.p.clone(),
        }
    }
}

impl Sub for FieldElement {
    type Output = FieldElement;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Mul for FieldElement {
    type Output = FieldElement;
    fn mul(self, rhs: Self) -> Self::Output {
        if self.p != rhs.p {
            panic!("Cannot add elements from different fields");
        }
        FieldElement {
            val: self.val.mul_mod(&rhs.val, &self.p),
            p: self.p.clone(),
        }
    }
}

impl Div for FieldElement {
    type Output = FieldElement;
    fn div(self, rhs: Self) -> Self::Output {
        if self.p != rhs.p {
            panic!("Cannot add elements from different fields");
        }

        if rhs.val.is_zero() {
            panic!("division by zero");
        }

        let p = &self.p;

        // a^p-1 = 1
        let inv = rhs.val.exp_mod(&p.sub(&U256::from(2u32)).0, p);
        FieldElement {
            val: self.val.mul_mod(&inv, &self.p),
            p: self.p.clone(),
        }
    }
}

impl FieldElement {
    pub fn get_p(&self) -> U256 {
        self.p.clone()
    }

    pub fn is_zero(&self) -> bool {
        self.val.is_zero()
    }

    pub fn show(&self) -> String {
        self.val.show()
    }

    pub fn pow(&self, other: &FieldElement) -> Self {
        FieldElement {
            val: self.val.exp_mod(&other.val, &self.p),
            p: self.p,
        }
    }

    pub fn legendre_symbol(&self) -> isize {
        if self.val.is_zero() {
            0
        } else {
            let gfp = GFp { p: self.p };
            let m = gfp.create(&self.p.sub(&U256::from(1u32)).0);

            let two = gfp.create(&U256::from(2u32));
            let one = gfp.create(&U256::from(1u32));

            let x = self.pow(&(m / two));

            if x == one {
                1
            } else {
                -1
            }
        }
    }

    pub fn sqrt(&self) -> Option<FieldElement> {
        if self.is_zero() {
            return Some(self.clone());
        }

        if self.legendre_symbol() != 1 {
            return None;
        }

        let p = &self.p;
        let field = GFp::new(p);

        // p = 3 (mod 4)
        // x = n^((p+1)/4) mod p
        let four = U256::from(4u32);
        if p.modulo(&four) == U256::from(3u32) {
            let exp = p.add(&U256::from(1u32)).0.shr(2); // (p+1)/4
            let result = self.val.exp_mod(&exp, p);
            return Some(field.create(&result));
        }

        // Tonelli-Shanks
        // p-1 = Q * 2^S
        let mut q = p.sub(&U256::from(1u32)).0;
        let mut s = 0u32;
        while q.modulo(&U256::from(2u32)).is_zero() {
            q = q.shr(1);
            s += 1;
        }

        // non-residue z
        let mut z = U256::from(2u32);
        loop {
            let z_elem = field.create(&z);
            if z_elem.legendre_symbol() == -1 {
                break;
            }
            z = z.add(&U256::from(1u32)).0;
        }

        let mut m = s;
        let mut c = z.exp_mod(&q, p);
        let mut t = self.val.exp_mod(&q, p);
        let mut r = self.val.exp_mod(&q.add(&U256::from(1u32)).0.shr(1), p);

        let one = U256::from(1u32);

        loop {
            if t.is_zero() {
                return Some(field.create(&U256::new()));
            }
            if t == one {
                return Some(field.create(&r));
            }

            // min i, t^(2^i) = 1
            let mut i = 1u32;
            let mut temp = t.mul_mod(&t, p);
            while temp != one && i < m {
                temp = temp.mul_mod(&temp, p);
                i += 1;
            }

            let exp = U256::from(1u32).shl((m - i - 1) as usize);
            let b = c.exp_mod(&exp, p);
            m = i;
            c = b.mul_mod(&b, p);
            t = t.mul_mod(&c, p);
            r = r.mul_mod(&b, p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 方便构造 U256
    fn u256(n: u32) -> U256 {
        U256::from(n)
    }

    #[test]
    fn test_add_same_field() {
        let p = u256(17);
        let field = GFp::new(&p);
        let a = field.create(&u256(5));
        let b = field.create(&u256(14));
        let c = a + b;

        assert_eq!(c.p, p);
        // 5 + 14 = 19 mod 17 = 2
        assert_eq!(c.val, u256(2));
    }

    #[test]
    #[should_panic(expected = "Cannot add elements from different fields")]
    fn test_add_diff_field_panic() {
        let field1 = GFp::new(&u256(17));
        let field2 = GFp::new(&u256(19));
        let a = field1.create(&u256(5));
        let b = field2.create(&u256(5));
        let _ = a + b;
    }

    #[test]
    fn test_neg() {
        let p = u256(17);
        let field = GFp::new(&p);
        let a = field.create(&u256(3));
        let neg_a = -a;

        // -3 mod 17 = 14
        assert_eq!(neg_a.val, u256(14));
        assert_eq!(neg_a.p, p);
    }

    #[test]
    fn test_mul() {
        let p = u256(17);
        let field = GFp::new(&p);
        let a = field.create(&u256(4));
        let b = field.create(&u256(5));
        let c = a * b;

        // 4 * 5 = 20 mod 17 = 3
        assert_eq!(c.val, u256(3));
    }

    #[test]
    #[should_panic(expected = "Cannot add elements from different fields")]
    fn test_mul_diff_field_panic() {
        let field1 = GFp::new(&u256(17));
        let field2 = GFp::new(&u256(19));
        let a = field1.create(&u256(5));
        let b = field2.create(&u256(5));
        let _ = a * b;
    }

    #[test]
    fn test_div() {
        let p = u256(17);
        let field = GFp::new(&p);
        let a = field.create(&u256(8));
        let b = field.create(&u256(5));

        let c = a / b;

        // verify a * b^{-1} mod p
        // 5^{-1} mod 17 = 7 since 5*7=35=1 mod17
        // 8*7=56=5 mod17
        assert_eq!(c.val, u256(5));
    }

    #[test]
    fn test_legendre_zero() {
        let p = u256(17);
        let field = GFp::new(&p);
        let zero = field.create(&u256(0));

        assert_eq!(zero.legendre_symbol(), 0);
    }

    #[test]
    fn test_legendre_quadratic_residue() {
        let p = u256(17);
        let field = GFp::new(&p);

        // 4 = 2^2 mod 17, 是二次剩余
        let a = field.create(&u256(4));
        assert_eq!(a.legendre_symbol(), 1);

        // 9 = 3^2 mod 17, 是二次剩余
        let b = field.create(&u256(9));
        assert_eq!(b.legendre_symbol(), 1);

        // 16 = 4^2 mod 17, 是二次剩余
        let c = field.create(&u256(16));
        assert_eq!(c.legendre_symbol(), 1);
    }

    #[test]
    fn test_legendre_non_residue() {
        let p = u256(17);
        let field = GFp::new(&p);

        // 3 不是模 17 的二次剩余
        let a = field.create(&u256(3));
        assert_eq!(a.legendre_symbol(), -1);

        // 5 不是模 17 的二次剩余
        let b = field.create(&u256(5));
        assert_eq!(b.legendre_symbol(), -1);
    }

    #[test]
    fn test_legendre_mod_7() {
        let p = u256(7);
        let field = GFp::new(&p);

        // 模 7 的二次剩余: 1, 2, 4
        assert_eq!(field.create(&u256(1)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(2)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(4)).legendre_symbol(), 1);

        // 模 7 的非二次剩余: 3, 5, 6
        assert_eq!(field.create(&u256(3)).legendre_symbol(), -1);
        assert_eq!(field.create(&u256(5)).legendre_symbol(), -1);
        assert_eq!(field.create(&u256(6)).legendre_symbol(), -1);

        // 0
        assert_eq!(field.create(&u256(0)).legendre_symbol(), 0);
    }

    #[test]
    fn test_legendre_mod_11() {
        let p = u256(11);
        let field = GFp::new(&p);

        // 模 11 的二次剩余: 1, 3, 4, 5, 9
        assert_eq!(field.create(&u256(1)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(3)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(4)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(5)).legendre_symbol(), 1);
        assert_eq!(field.create(&u256(9)).legendre_symbol(), 1);

        // 模 11 的非二次剩余: 2, 6, 7, 8, 10
        assert_eq!(field.create(&u256(2)).legendre_symbol(), -1);
        assert_eq!(field.create(&u256(6)).legendre_symbol(), -1);
        assert_eq!(field.create(&u256(7)).legendre_symbol(), -1);
    }

    #[test]
    fn test_sqrt_zero() {
        let p = u256(17);
        let field = GFp::new(&p);
        let zero = field.create(&u256(0));

        let sqrt = zero.sqrt().unwrap();
        assert_eq!(sqrt.val, u256(0));
    }

    #[test]
    fn test_sqrt_perfect_square() {
        let p = u256(17);
        let field = GFp::new(&p);

        // 4 = 2^2 mod 17
        let a = field.create(&u256(4));
        let sqrt = a.sqrt().unwrap();
        assert!(sqrt.val == u256(2) || sqrt.val == u256(15)); // ±2 mod 17

        // 验证结果
        let squared = sqrt * sqrt;
        assert_eq!(squared.val, u256(4));
    }

    #[test]
    fn test_sqrt_non_residue() {
        let p = u256(17);
        let field = GFp::new(&p);

        // 3 不是二次剩余
        let a = field.create(&u256(3));
        assert!(a.sqrt().is_none());
    }

    #[test]
    fn test_sqrt_mod_7() {
        let p = u256(7);
        let field = GFp::new(&p);

        // 2 是二次剩余，3^2 = 9 = 2 mod 7
        let a = field.create(&u256(2));
        let sqrt = a.sqrt().unwrap();
        assert!(sqrt.val == u256(3) || sqrt.val == u256(4)); // ±3 mod 7

        // 验证
        let squared = sqrt * sqrt;
        assert_eq!(squared.val, u256(2));
    }

    #[test]
    fn test_sqrt_p_mod_4_eq_3() {
        // p = 11 ≡ 3 (mod 4)，使用快速路径
        let p = u256(11);
        let field = GFp::new(&p);

        // 5 是二次剩余，4^2 = 16 = 5 mod 11
        let a = field.create(&u256(5));
        let sqrt = a.sqrt().unwrap();
        assert!(sqrt.val == u256(4) || sqrt.val == u256(7)); // ±4 mod 11

        // 验证
        let squared = sqrt * sqrt;
        assert_eq!(squared.val, u256(5));
    }

    #[test]
    fn test_sqrt_secp256k1_prime() {
        // secp256k1 的素数 p = 2^256 - 2^32 - 977
        // p = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
        let p = U256([
            0xFFFFFC2F, 0xFFFFFFFE, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0xFFFFFFFF,
        ]);

        let field = GFp::new(&p);

        // 测试一个已知的非二次剩余
        let n = field.create(&U256::from(3u32));
        let l = n.legendre_symbol();

        assert_eq!(l, -1, "3 is non-residue");

        let n = field.create(&U256::from(2u32));
        let l = n.legendre_symbol();
        assert_eq!(l, 1, "2 is residue");

        let n = field.create(&U256::from(8u32));
        let l = n.legendre_symbol();
        assert_eq!(l, 1, "8 is residue");
    }

    #[test]
    fn test_sqrt_secp256k1_known_square() {
        // secp256k1 素数
        let p = U256([
            0xFFFFFC2F, 0xFFFFFFFE, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0xFFFFFFFF,
        ]);

        let field = GFp::new(&p);

        // 取一个数 x，计算 x^2，然后对 x^2 开方应该得到 ±x
        let x = field.create(&U256([
            0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222, 0x33333333, 0x44444444, 0x55555555,
            0x66666666,
        ]));

        let x_squared = x * x;

        let sqrt = x_squared.sqrt().unwrap();

        // sqrt 应该等于 x 或 -x
        assert!(sqrt == x || sqrt == -x);

        // 验证 sqrt^2 = x^2
        let verify = sqrt * sqrt;
        assert_eq!(verify, x_squared);
    }

    #[test]
    fn test_sqrt_p256_prime() {
        // NIST P-256 曲线的素数
        // p = 2^256 - 2^224 + 2^192 + 2^96 - 1
        // p = 0xFFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF
        let p = U256([
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
            0xFFFFFFFF,
        ]);

        let field = GFp::new(&p);

        // 测试一个大数
        let n = field.create(&U256([
            0xDEADBEEF, 0xCAFEBABE, 0x12345678, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210, 0xAAAAAAAA,
            0x55555555,
        ]));

        let legendre = n.legendre_symbol();
        assert_eq!(legendre, 1, "n is residue");

        let sqrt = n.sqrt().unwrap();
        let squared = sqrt * sqrt;
        assert_eq!(squared, n);
    }

    #[test]
    fn test_sqrt_large_perfect_square() {
        // 使用一个较小但仍然很大的素数
        // p = 2^128 - 159 (这是一个素数)
        let p = U256([0xFFFFFF61, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0, 0, 0, 0]);

        let field = GFp::new(&p);

        // 构造一个已知的平方数
        let base = field.create(&U256([
            0x87654321, 0xFEDCBA98, 0x13579BDF, 0x2468ACE0, 0, 0, 0, 0,
        ]));

        let square = base * base;

        // 对平方数开方
        let sqrt = square.sqrt().unwrap();

        // 应该得到 ±base
        assert!(sqrt == base || sqrt == -base);

        // 验证
        let verify = sqrt * sqrt;
        assert_eq!(verify, square);
    }

    #[test]
    fn test_sqrt_edge_case_one() {
        let p = U256([
            0xFFFFFC2F, 0xFFFFFFFE, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0xFFFFFFFF,
        ]);

        let field = GFp::new(&p);
        let one = field.create(&U256::from(1u32));

        // 1 的平方根是 ±1
        let sqrt = one.sqrt().unwrap();
        assert!(sqrt.val == U256::from(1u32) || sqrt.val == p.sub(&U256::from(1u32)).0);

        let squared = sqrt * sqrt;
        assert_eq!(squared, one);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_div_by_zero_panic() {
        let p = u256(17);
        let field = GFp::new(&p);
        let a = field.create(&u256(8));
        let zero = field.create(&u256(0));
        let _ = a / zero;
    }

    #[test]
    #[should_panic(expected = "Cannot add elements from different fields")]
    fn test_div_diff_field_panic() {
        let field1 = GFp::new(&u256(17));
        let field2 = GFp::new(&u256(19));
        let a = field1.create(&u256(5));
        let b = field2.create(&u256(5));
        let _ = a / b;
    }
}
