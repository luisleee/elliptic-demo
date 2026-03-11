use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U256(pub [u32; 8]);

impl From<u32> for U256 {
    fn from(value: u32) -> Self {
        let mut arr = [0u32; 8];
        arr[0] = value;
        U256(arr)
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        let mut arr = [0u32; 8];
        arr[0] = value as u32;
        arr[1] = (value >> 32) as u32;
        U256(arr)
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        let mut arr = [0u32; 8];
        arr[0] = value as u32;
        arr[1] = (value >> 32) as u32;
        arr[2] = (value >> 64) as u32;
        arr[3] = (value >> 96) as u32;
        U256(arr)
    }
}

impl From<[u8; 32]> for U256 {
    fn from(bytes: [u8; 32]) -> Self {
        let mut arr = [0u32; 8];
        for i in 0..8 {
            let offset = i * 4;
            arr[i] = (bytes[offset] as u32)
                | ((bytes[offset + 1] as u32) << 8)
                | ((bytes[offset + 2] as u32) << 16)
                | ((bytes[offset + 3] as u32) << 24);
        }
        U256(arr)
    }
}

impl From<[u32; 8]> for U256 {
    fn from(qbytes: [u32; 8]) -> Self {
        U256(qbytes)
    }
}

impl Ord for U256 {
    fn cmp(&self, other: &U256) -> Ordering {
        for i in (0..8).rev() {
            if self.0[i] < other.0[i] {
                return Ordering::Less;
            } else if self.0[i] > other.0[i] {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl std::fmt::Debug for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl U256 {
    pub fn new() -> Self {
        U256([0u32; 8])
        // low end: 0
        // high end: 7
    }

    fn one() -> Self {
        U256([1, 0, 0, 0, 0, 0, 0, 0])
    }

    pub fn show(&self) -> String {
        let parts: Vec<String> = (0..8).rev().map(|i| format!("{:08x}", self.0[i])).collect();

        format!("0x{}", parts.join(" "))
    }

    pub fn from_hex_str(s: &str) -> Result<Self, String> {
        let s = s.trim_start_matches("0x").trim();

        if s.len() > 64 {
            return Err("Hex string too long for U256".to_owned());
        }

        let mut data = [0u32; 8];

        let mut hex_iter = s.chars().rev();
        for i in 0..8 {
            let mut val = 0u32;
            for j in 0..8 {
                if let Some(c) = hex_iter.next() {
                    let digit = c.to_digit(16).ok_or(format!("Invalid hex digit {}", c))?;
                    val |= digit << (4 * j);
                } else {
                    break;
                }
            }
            data[i] = val;
        }

        Ok(U256(data))
    }

    pub fn add(&self, other: &U256) -> (Self, u64) {
        let mut result = [0u32; 8];
        let mut carry = 0u64;

        for i in 0..8 {
            let sum = (self.0[i] as u64 + other.0[i] as u64) + carry;
            result[i] = (sum & 0xFFFFFFFF) as u32;
            carry = sum >> 32;
        }

        (U256(result), carry)
    }

    pub fn sub(&self, other: &Self) -> (Self, bool) {
        let mut result = [0u32; 8];
        let mut borrow = 0u64;
        for i in 0..8 {
            let sub = (self.0[i] as u64).wrapping_sub(other.0[i] as u64 + borrow);
            result[i] = sub as u32;
            borrow = if sub >> 63 == 1 { 1 } else { 0 };
        }
        (U256(result), borrow != 0)
    }

    pub fn shl(&self, shift: usize) -> Self {
        if shift == 0 {
            return self.clone();
        }

        let mut result = [0u32; 8];

        let words = shift / 32;
        let bits = shift % 32;

        for i in words..8 {
            result[i] = self.0[i - words];
        }

        if bits > 0 {
            for i in (1..8).rev() {
                result[i] = (result[i] << bits) | (result[i - 1] >> (32 - bits));
            }
            result[0] <<= bits;
        }

        U256(result)
    }

    pub fn shr(&self, shift: usize) -> Self {
        if shift == 0 {
            return self.clone();
        }

        let mut result = [0u32; 8];
        let words = shift / 32;
        let bits = shift % 32;

        for i in 0..8 - words {
            result[i] = self.0[i + words];
        }

        if bits > 0 {
            for i in 0..7 {
                result[i] = (result[i] >> bits) | (result[i + 1] << (32 - bits));
            }
            result[7] >>= bits;
        }

        U256(result)
    }

    pub fn unchecked_mul(&self, other: &U256) -> Self {
        let mut result = [0u32; 8];

        for i in 0..8 {
            let mut carry = 0u64;
            for j in 0..8 - i {
                let prod = (self.0[i] as u64 * other.0[j] as u64) + carry + result[j + i] as u64;

                result[j + i] = (prod & 0xFFFFFFFF) as u32;
                carry = prod >> 32;
            }
        }
        U256(result)
    }

    pub fn leading_zeros(&self) -> usize {
        for i in (0..8).rev() {
            if self.0[i] != 0 {
                return (7 - i) * 32 + self.0[i].leading_zeros() as usize;
            }
        }
        256
    }

    pub fn bit_length(&self) -> usize {
        256 - self.leading_zeros()
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0u32; 8]
    }

    pub fn quotient(&self, modulus: &U256) -> (Self, Self) {
        if modulus.is_zero() {
            panic!("division by zero");
        }
        if self < modulus {
            return (U256::new(), self.clone());
        }
        let shift = modulus.leading_zeros() - self.leading_zeros();
        let mut q = [0u32; 8];
        let mut s = self.clone();

        for i in (0..shift + 1).rev() {
            let shifted = modulus.shl(i);
            if shifted > s {
                continue;
            }
            s = s.sub(&shifted).0;
            let words = i / 32;
            let bits = i % 32;
            q[words] |= 1 << bits;
        }

        (U256(q), s)
    }

    pub fn div(&self, modulus: &U256) -> Self {
        self.quotient(modulus).0
    }

    pub fn modulo(&self, modulus: &U256) -> Self {
        self.quotient(modulus).1
    }

    pub fn add_mod(&self, other: &U256, modulus: &U256) -> Self {
        let a = self.modulo(modulus);
        let b = other.modulo(modulus);

        let (sum, carry) = a.add(&b);
        if carry == 0 {
            if sum < *modulus {
                sum
            } else {
                sum.sub(modulus).0
            }
        } else {
            sum.sub(modulus).0
        }
    }

    pub fn and(&self, other: &U256) -> Self {
        let mut result = [0u32; 8];
        for i in 0..8 {
            result[i] = self.0[i] & other.0[i];
        }
        U256(result)
    }

    pub fn or(&self, other: &U256) -> Self {
        let mut result = [0u32; 8];
        for i in 0..8 {
            result[i] = self.0[i] | other.0[i];
        }
        U256(result)
    }

    pub fn mul_mod(&self, other: &U256, modulus: &U256) -> Self {
        let mut a = self.clone();
        let mut b = other.clone();
        let mut result = U256::new();

        while !b.is_zero() {
            if !b.and(&U256::one()).is_zero() {
                result = result.add_mod(&a, modulus)
            }
            a = a.add_mod(&a, modulus);
            b = b.shr(1);
        }

        result
    }

    pub fn exp_mod(&self, other: &U256, modulus: &U256) -> Self {
        let mut a = self.clone();
        let mut b = other.clone();
        let mut result = U256::one();

        while !b.is_zero() {
            if !b.and(&U256::one()).is_zero() {
                result = result.mul_mod(&a, modulus)
            }
            a = a.mul_mod(&a, modulus);
            b = b.shr(1);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_new() {
        let u = U256::new();
        assert_eq!(u.0, [0u32; 8]);
    }

    #[test]
    fn test_add() {
        // 基本加法
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([2, 0, 0, 0, 0, 0, 0, 0]);
        let (sum, carry) = a.add(&b);
        assert_eq!(sum.0[0], 3);
        assert_eq!(carry, 0);

        // 进位测试
        let a = U256([0xFFFFFFFF, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let (sum, carry) = a.add(&b);
        assert_eq!(sum.0[0], 0);
        assert_eq!(sum.0[1], 1);
        assert_eq!(carry, 0);

        // 多进位测试
        let a = U256([
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0xFFFFFFFF,
        ]);
        let b = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let (sum, carry) = a.add(&b);
        assert_eq!(sum.0, [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(carry, 1);
    }

    #[test]
    fn test_cmp() {
        let a = U256([1, 2, 3, 4, 5, 6, 7, 8]);
        let b = U256([1, 2, 3, 4, 5, 6, 7, 8]);
        let c = U256([2, 2, 3, 4, 5, 6, 7, 8]);
        let d = U256([1, 2, 3, 4, 5, 6, 7, 7]);

        assert_eq!(a.cmp(&b), Ordering::Equal);
        assert_eq!(a.cmp(&c), Ordering::Less);
        assert_eq!(c.cmp(&a), Ordering::Greater);
        assert_eq!(a.cmp(&d), Ordering::Greater);
    }

    #[test]
    fn test_sub() {
        // 基本减法
        let a = U256([5, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([3, 0, 0, 0, 0, 0, 0, 0]);
        let (diff, borrow) = a.sub(&b);
        assert_eq!(diff.0[0], 2);
        assert_eq!(borrow, false);

        // 借位测试
        let a = U256([0, 1, 0, 0, 0, 0, 0, 0]);
        let b = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let (diff, borrow) = a.sub(&b);
        assert_eq!(diff.0[0], 0xFFFFFFFF);
        assert_eq!(diff.0[1], 0);
        assert_eq!(borrow, false);

        // 下溢测试
        let a = U256([0, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let (diff, borrow) = a.sub(&b);
        assert_eq!(diff.0[0], 0xFFFFFFFF);
        assert_eq!(
            diff.0[1..],
            [0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF]
        );
        assert_eq!(borrow, true);
    }

    #[test]
    fn test_shl() {
        // 左移0位
        let a = U256([1, 2, 3, 4, 5, 6, 7, 8]);
        let result = a.shl(0);
        assert_eq!(result.0, a.0);

        // 左移1位
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shl(1);
        assert_eq!(result.0[0], 2);

        // 左移31位
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shl(31);
        assert_eq!(result.0[0], 0x80000000);
        assert_eq!(result.0[1], 0);

        // 左移32位
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shl(32);
        assert_eq!(result.0[0], 0);
        assert_eq!(result.0[1], 1);

        // 左移33位
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shl(33);
        assert_eq!(result.0[0], 0);
        assert_eq!(result.0[1], 2);

        // 左移超过256位
        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shl(300);
        assert_eq!(result.0, [0u32; 8]);
    }

    #[test]
    fn test_shr() {
        // 右移0位
        let a = U256([1, 2, 3, 4, 5, 6, 7, 8]);
        let result = a.shr(0);
        assert_eq!(result.0, a.0);

        // 右移1位
        let a = U256([2, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shr(1);
        assert_eq!(result.0[0], 1);

        // 右移31位
        let a = U256([0x80000000, 0, 0, 0, 0, 0, 0, 0]);
        let result = a.shr(31);
        assert_eq!(result.0[0], 1);

        // 右移32位（刚好一个word）
        let a = U256([0, 1, 0, 0, 0, 0, 0, 0]);
        let result = a.shr(32);
        assert_eq!(result.0[0], 1);
        assert_eq!(result.0[1], 0);

        // 右移33位
        let a = U256([0, 2, 0, 0, 0, 0, 0, 0]);
        let result = a.shr(33);
        assert_eq!(result.0[0], 1);
        assert_eq!(result.0[1], 0);

        // 更复杂的跨word移位
        let a = U256([0x12345678, 0x9ABCDEF0, 0, 0, 0, 0, 0, 0]);
        let result = a.shr(16);
        assert_eq!(result.0[0], 0xDEF01234);
        assert_eq!(result.0[1], 0x00009ABC);
    }

    #[test]
    fn test_is_zero() {
        let zero = U256([0u32; 8]);
        assert!(zero.is_zero(), "All zero array should be zero");

        // 在最低位设置1，应该返回 false
        let mut arr = [0u32; 8];
        arr[0] = 1;
        let val = U256(arr);
        assert!(
            !val.is_zero(),
            "Value with lowest bit set should NOT be zero"
        );

        // 在最高位设置1，应该返回 false
        let mut arr = [0u32; 8];
        arr[7] = 1;
        let val = U256(arr);
        assert!(
            !val.is_zero(),
            "Value with highest bit set should NOT be zero"
        );

        // 中间几位非零
        let mut arr = [0u32; 8];
        arr[3] = 123456;
        arr[5] = 987654321;
        let val = U256(arr);
        assert!(
            !val.is_zero(),
            "Value with multiple bits set should NOT be zero"
        );
    }

    #[test]
    fn test_quotient() {
        let zero = U256([0; 8]);

        // 1. self < modulus, e.g. 5 < 10
        let a = U256::from(5u64);
        let b = U256::from(10u64);
        let (q, r) = a.quotient(&b);
        assert_eq!(q, zero);
        assert_eq!(r, a);

        // 2. Exact division, e.g. 20 / 5 = 4 remainder 0
        let a = U256::from(20u64);
        let b = U256::from(5u64);
        let (q, r) = a.quotient(&b);
        let expected_q = U256::from(4u64);
        assert_eq!(q, expected_q);
        assert_eq!(r, zero);

        // 3. Division with remainder, e.g. 22 / 5 = 4 remainder 2
        let a = U256::from(22u64);
        let b = U256::from(5u64);
        let (q, r) = a.quotient(&b);
        let expected_q = U256::from(4u64);
        let expected_r = U256::from(2u64);
        assert_eq!(q, expected_q);
        assert_eq!(r, expected_r);

        // 4. Large number division:
        // Construct a = 0x00000001_00000000_00000000_00000000 (bit 96 set)
        let mut a_arr = [0u32; 8];
        a_arr[3] = 1; // index 3 corresponds to 32*3=96 bit position low to high
        let a = U256(a_arr);

        // divisor b = 2
        let b = U256::from(2u64);

        let (q, r) = a.quotient(&b);

        // q should be a >> 1 (i.e. bit 95 set)
        let mut expected_q_arr = [0u32; 8];
        expected_q_arr[2] = 0x80000000; // 2^95 = 1 << 31 in word 2 (since word 3 bit 0 >> 1 = word 2 bit 31)
        let expected_q = U256(expected_q_arr);

        let expected_r = zero; // remainder 0 since even division

        assert_eq!(q, expected_q);
        assert_eq!(r, expected_r);
    }

    #[test]
    fn test_unchecked_mul() {
        // 基本乘法
        let a = U256([5, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([3, 0, 0, 0, 0, 0, 0, 0]);
        let prod = a.unchecked_mul(&b);
        assert_eq!(prod.0[0], 15);

        // 大数乘法
        let a = U256([0xFFFFFFFF, 0, 0, 0, 0, 0, 0, 0]);
        let b = U256([2, 0, 0, 0, 0, 0, 0, 0]);
        let prod = a.unchecked_mul(&b);
        assert_eq!(prod.0[0], 0xFFFFFFFE);
        assert_eq!(prod.0[1], 1);

        // 多字乘法
        let a = U256([0xFFFFFFFF, 0xFFFFFFFF, 0, 0, 0, 0, 0, 0]);
        let b = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        let prod = a.unchecked_mul(&b);
        assert_eq!(prod.0[0], 0xFFFFFFFF);
        assert_eq!(prod.0[1], 0xFFFFFFFF);

        // 创建一个会导致连续进位的乘法
        let a = U256([0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0, 0, 0, 0, 0]);
        let b = U256([0xFFFFFFFF, 0, 0, 0, 0, 0, 0, 0]);

        let prod = a.unchecked_mul(&b);

        // 验证进位正确传播
        assert_eq!(prod.0[0], 1);
        assert_eq!(prod.0[1], 0xFFFFFFFF);
        assert_eq!(prod.0[2], 0xFFFFFFFF);
        assert_eq!(prod.0[3], 0xFFFFFFFE);
    }

    #[test]
    fn test_leading_zeros() {
        let a = U256([0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.leading_zeros(), 256);

        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.leading_zeros(), 256 - 1); // 因为最低位是1，所以前面有255个0

        let a = U256([0, 0, 0, 0, 0, 0, 0, 0x80000000]);
        assert_eq!(a.leading_zeros(), 0);

        let a = U256([0, 0, 0, 0, 0, 0, 0, 0x0F000000]);
        assert_eq!(a.leading_zeros(), 4);
    }

    #[test]
    fn test_bit_length() {
        let a = U256([0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.bit_length(), 0);

        let a = U256([1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.bit_length(), 1);

        let a = U256([0, 0, 0, 0, 0, 0, 0, 0x80000000]);
        assert_eq!(a.bit_length(), 256);

        let a = U256([
            0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
            0x7FFFFFFF,
        ]);
        assert_eq!(a.bit_length(), 255);
    }

    #[test]
    fn test_edge_cases() {
        // 最大数
        let max = U256([0xFFFFFFFF; 8]);

        // max + 1 = overflow
        let (sum, carry) = max.add(&U256([1, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(sum.0, [0u32; 8]);
        assert_eq!(carry, 1);

        // max - max = 0
        let (diff, borrow) = max.sub(&max);
        assert_eq!(diff.0, [0u32; 8]);
        assert_eq!(borrow, false);

        // max * 1 = max
        let prod = max.unchecked_mul(&U256([1, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(prod.0, max.0);
    }

    #[test]
    fn test_properties() {
        // 测试交换律：a + b = b + a
        let a = U256([123, 456, 789, 101, 112, 131, 415, 161]);
        let b = U256([718, 192, 021, 222, 324, 252, 627, 282]);
        let (sum1, carry1) = a.add(&b);
        let (sum2, carry2) = b.add(&a);
        assert_eq!(sum1.0, sum2.0);
        assert_eq!(carry1, carry2);

        // 测试结合律：(a + b) + c = a + (b + c)
        let c = U256([303, 234, 353, 637, 383, 940, 414, 243]);
        let (sum1, _carry1) = a.add(&b);
        let (sum1, carry1) = sum1.add(&c);
        let (sum2, _carry2) = b.add(&c);
        let (sum2, carry2) = a.add(&sum2);
        assert_eq!(sum1.0, sum2.0);
        assert_eq!(carry1, carry2);
    }

    #[test]
    fn test_add_mod() {
        let modulus = U256::from(10u64);

        // 两个加数均小于模且和小于模，结果应该是加法结果本身
        let a = U256::from(3u64);
        let b = U256::from(4u64);
        let r = a.add_mod(&b, &modulus);
        assert_eq!(r, U256::from(7u64));

        // 和等于模，结果应当是0
        let a = U256::from(7u64);
        let b = U256::from(3u64);
        let r = a.add_mod(&b, &modulus);
        assert_eq!(r, U256::from(0u64));

        // 和大于模，结果应当是和减模
        let a = U256::from(8u64);
        let b = U256::from(5u64);
        let r = a.add_mod(&b, &modulus);
        // 8+5=13, 13 mod 10=3
        assert_eq!(r, U256::from(3u64));

        // 模数是很大的数字
        let mut modulus_arr = [0u32; 8];
        modulus_arr[7] = 1; // 模数 = 2^(32*7) = 2^224
        let modulus = U256(modulus_arr);

        // 加数a = 2^(223)
        let mut a_arr = [0u32; 8];
        a_arr[6] = 1 << 31; // 位于第6个元素最高位
        let a = U256(a_arr);

        // 加数b = 2^(223)
        let b = a.clone();
        let r = a.add_mod(&b, &modulus);

        // 2^(223) + 2^(223) = 2^(224) == 模数，所以余数为0
        assert_eq!(r, U256([0; 8]));

        let max = U256([0xffffffff; 8]);
        let k = max.add_mod(&max, &U256::from(2u64));
        assert_eq!(k, U256::new());
    }

    #[test]
    fn test_mul_mod() {
        let modulus = U256::from(10u64);

        // 2 * 3 % 10 = 6
        let a = U256::from(2u64);
        let b = U256::from(3u64);
        let r = a.mul_mod(&b, &modulus);
        assert_eq!(r, U256::from(6u64));

        // 7 * 5 % 10 = 5 (35 % 10 =5)
        let a = U256::from(7u64);
        let b = U256::from(5u64);
        let r = a.mul_mod(&b, &modulus);
        assert_eq!(r, U256::from(5u64));

        // 0 * any = 0
        let a = U256::from(0u64);
        let b = U256::from(123456u64);
        let r = a.mul_mod(&b, &modulus);
        assert_eq!(r, U256::from(0u64));

        // 模数 2^128 + 1 (简化写法)
        let mut modulo_arr = [0u32; 8];
        modulo_arr[4] = 1;
        modulo_arr[0] = 1;
        let modulus = U256(modulo_arr);

        // a = 2^64
        let mut a_arr = [0u32; 8];
        a_arr[2] = 1 << 0; // bit 64-95 的最低位
        let a = U256(a_arr);

        // b = 2^64
        let b = a.clone();

        let r = a.mul_mod(&b, &modulus);

        // 2^64 * 2^64 = 2^128, 2^128 mod (2^128 + 1) = (2^128)
        let mut expected_arr = [0u32; 8];
        expected_arr[4] = 1;
        let expected = U256(expected_arr);

        assert_eq!(r, expected);
    }

    #[test]
    fn test_exp_mod() {
        let modulus = U256::from(1000u64);

        // 2^10 mod 1000 = 1024 mod 1000 = 24
        let base = U256::from(2u64);
        let exponent = U256::from(10u64);
        let result = base.exp_mod(&exponent, &modulus);
        assert_eq!(result, U256::from(24u64));

        // 3^0 mod 1000 = 1
        let base = U256::from(3u64);
        let exponent = U256::from(0u64);
        let result = base.exp_mod(&exponent, &modulus);
        assert_eq!(result, U256::from(1u64));

        // 10^5 mod 1000 = 0
        let base = U256::from(10u64);
        let exponent = U256::from(5u64);
        let result = base.exp_mod(&exponent, &modulus);
        let expected = U256::from(0u64);
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_quotient_modulus_zero() {
        let a = U256::from(1u64);
        let zero_mod = U256([0u32; 8]);
        // 会 panicq
        let _ = a.quotient(&zero_mod);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_add_mod_modulus_zero() {
        let a = U256::from(1u64);
        let b = U256::from(1u64);
        let zero_mod = U256([0u32; 8]);
        // 会 panic
        let _ = a.add_mod(&b, &zero_mod);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_mul_mod_modulus_zero() {
        let a = U256::from(1u64);
        let b = U256::from(1u64);
        let zero_mod = U256([0u32; 8]);
        // 会 panic
        let _ = a.mul_mod(&b, &zero_mod);
    }
}
