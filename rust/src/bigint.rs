// SPDX-License-Identifier: MIT
//! Arbitrary-precision integers for values past i64. Sign and magnitude,
//! base 2^32 limbs, least significant first.

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigInt {
    pub neg: bool,
    pub mag: Vec<u32>,
}

fn trim(v: &mut Vec<u32>) {
    while let Some(&0) = v.last() {
        v.pop();
    }
}

fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    for i in (0..a.len()).rev() {
        if a[i] != b[i] {
            return a[i].cmp(&b[i]);
        }
    }
    Ordering::Equal
}

fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (a, b) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    let mut out = Vec::with_capacity(a.len() + 1);
    let mut carry = 0u64;
    for i in 0..a.len() {
        let s = a[i] as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
        out.push(s as u32);
        carry = s >> 32;
    }
    if carry > 0 {
        out.push(carry as u32);
    }
    out
}

/// a - b with |a| >= |b|
fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for i in 0..a.len() {
        let mut d = a[i] as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
        if d < 0 {
            d += 1 << 32;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push(d as u32);
    }
    trim(&mut out);
    out
}

/// operands at least this long (in limbs) multiply by Karatsuba
const KARATSUBA_LIMBS: usize = 40;

fn mul_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut out = vec![0u32; a.len() + b.len()];
    mul_into(&mut out, a, b);
    trim(&mut out);
    out
}

/// out += a * b, where out has room for a.len() + b.len() limbs
fn mul_into(out: &mut [u32], a: &[u32], b: &[u32]) {
    let (a, b) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if b.len() < KARATSUBA_LIMBS {
        schoolbook_into(out, a, b);
    } else if a.len() >= 2 * b.len() {
        // unbalanced: b times b-sized pieces of a
        for (i, piece) in a.chunks(b.len()).enumerate() {
            let at = i * b.len();
            mul_into(&mut out[at..at + piece.len() + b.len()], piece, b);
        }
    } else {
        karatsuba_into(out, a, b);
    }
}

fn schoolbook_into(out: &mut [u32], a: &[u32], b: &[u32]) {
    for (i, &x) in b.iter().enumerate() {
        if x == 0 {
            continue;
        }
        let mut carry = 0u64;
        for (j, &y) in a.iter().enumerate() {
            let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
            out[i + j] = t as u32;
            carry = t >> 32;
        }
        add_at(out, i + a.len(), carry);
    }
}

/// adds a small value into out from limb `at` up
fn add_at(out: &mut [u32], mut at: usize, mut carry: u64) {
    while carry > 0 {
        let t = out[at] as u64 + carry;
        out[at] = t as u32;
        carry = t >> 32;
        at += 1;
    }
}

/// out += x, x no longer than out
fn add_into(out: &mut [u32], x: &[u32]) {
    let mut carry = 0u64;
    for (i, &v) in x.iter().enumerate() {
        let t = out[i] as u64 + v as u64 + carry;
        out[i] = t as u32;
        carry = t >> 32;
    }
    add_at(out, x.len(), carry);
}

/// out -= x, where the result stays non-negative
fn sub_into(out: &mut [u32], x: &[u32]) {
    let mut borrow = 0u64;
    for (i, o) in out.iter_mut().enumerate() {
        if i >= x.len() && borrow == 0 {
            break;
        }
        let t = (*o as u64).wrapping_sub(*x.get(i).unwrap_or(&0) as u64).wrapping_sub(borrow);
        *o = t as u32;
        borrow = (t >> 63) & 1;
    }
}

/// a * b = z2·B² + (z1 - z2 - z0)·B + z0 with z1 = (a1 + a0)(b1 + b0),
/// for b.len() <= a.len() < 2 * b.len()
fn karatsuba_into(out: &mut [u32], a: &[u32], b: &[u32]) {
    let half = a.len().div_ceil(2);
    let (a0, a1) = a.split_at(half);
    let (b0, b1) = b.split_at(half.min(b.len()));
    let mut z0 = vec![0u32; a0.len() + b0.len()];
    mul_into(&mut z0, a0, b0);
    let mut z2 = vec![0u32; a1.len() + b1.len()];
    mul_into(&mut z2, a1, b1);
    let sa = add_mag(a0, a1);
    let sb = add_mag(b0, b1);
    let mut z1 = vec![0u32; sa.len() + sb.len()];
    mul_into(&mut z1, &sa, &sb);
    sub_into(&mut z1, &z0);
    sub_into(&mut z1, &z2);
    trim(&mut z0);
    trim(&mut z1);
    trim(&mut z2);
    add_into(out, &z0);
    add_into(&mut out[half..], &z1);
    add_into(&mut out[2 * half..], &z2);
}

fn divmod_small(a: &[u32], d: u32) -> (Vec<u32>, u32) {
    let mut out = vec![0u32; a.len()];
    let mut rem = 0u64;
    for i in (0..a.len()).rev() {
        let cur = (rem << 32) | a[i] as u64;
        out[i] = (cur / d as u64) as u32;
        rem = cur % d as u64;
    }
    trim(&mut out);
    (out, rem as u32)
}

fn shl_mag(a: &[u32], n: usize) -> Vec<u32> {
    if a.is_empty() {
        return vec![];
    }
    let words = n / 32;
    let bits = n % 32;
    let mut out = vec![0u32; words];
    if bits == 0 {
        out.extend_from_slice(a);
    } else {
        let mut carry = 0u32;
        for &x in a {
            out.push((x << bits) | carry);
            carry = x >> (32 - bits);
        }
        if carry > 0 {
            out.push(carry);
        }
    }
    out
}

fn shr_mag(a: &[u32], n: usize) -> Vec<u32> {
    let words = n / 32;
    if words >= a.len() {
        return vec![];
    }
    let bits = n % 32;
    let mut out: Vec<u32> = a[words..].to_vec();
    if bits > 0 {
        for i in 0..out.len() {
            let hi = if i + 1 < out.len() { out[i + 1] << (32 - bits) } else { 0 };
            out[i] = (out[i] >> bits) | hi;
        }
    }
    trim(&mut out);
    out
}

fn bit_len_mag(a: &[u32]) -> usize {
    match a.last() {
        None => 0,
        Some(&top) => (a.len() - 1) * 32 + (32 - top.leading_zeros() as usize),
    }
}

/// long division, Knuth-style via shifting (simple binary schoolbook on limbs)
fn divmod_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if cmp_mag(a, b) == Ordering::Less {
        return (vec![], a.to_vec());
    }
    if b.len() == 1 {
        let (q, r) = divmod_small(a, b[0]);
        return (q, if r == 0 { vec![] } else { vec![r] });
    }
    // normalise so the divisor's top limb has its high bit set
    let s = b.last().unwrap().leading_zeros() as usize;
    let b = shl_mag(b, s);
    let alen = a.len();
    let mut a = shl_mag(a, s);
    if a.len() == alen {
        a.push(0);
    }
    let n = b.len();
    let m = a.len() - n;
    let mut q = vec![0u32; m];
    let btop = b[n - 1] as u64;
    let bnext = b[n - 2] as u64;
    for j in (0..m).rev() {
        let num = ((a[j + n] as u64) << 32) | a[j + n - 1] as u64;
        let mut qhat = num / btop;
        let mut rhat = num % btop;
        while qhat >= (1 << 32) || qhat * bnext > ((rhat << 32) | a[j + n - 2] as u64) {
            qhat -= 1;
            rhat += btop;
            if rhat >= (1 << 32) {
                break;
            }
        }
        // multiply and subtract
        let mut borrow = 0i64;
        let mut carry = 0u64;
        for i in 0..n {
            let p = qhat * b[i] as u64 + carry;
            carry = p >> 32;
            let t = a[i + j] as i64 - borrow - (p & 0xffff_ffff) as i64;
            a[i + j] = t as u32;
            borrow = if t < 0 { 1 } else { 0 };
        }
        let t = a[j + n] as i64 - borrow - carry as i64;
        a[j + n] = t as u32;
        if t < 0 {
            // add back
            qhat -= 1;
            let mut c = 0u64;
            for i in 0..n {
                let s2 = a[i + j] as u64 + b[i] as u64 + c;
                a[i + j] = s2 as u32;
                c = s2 >> 32;
            }
            a[j + n] = a[j + n].wrapping_add(c as u32);
        }
        q[j] = qhat as u32;
    }
    trim(&mut q);
    a.truncate(n);
    trim(&mut a);
    let r = shr_mag(&a, s);
    (q, r)
}

/// 10**9, the base the decimal conversions work in
const DEC_BASE: u64 = 1_000_000_000;

/// the decimal digits of a magnitude
fn mag_to_decimal(mag: &[u32], out: &mut String) {
    let mut cache = std::collections::HashMap::new();
    let mut pieces = to_dec(mag, &mut cache);
    trim(&mut pieces);
    let mut it = pieces.iter().rev();
    match it.next() {
        Some(top) => out.push_str(&top.to_string()),
        None => out.push('0'),
    }
    for p in it {
        out.push_str(&format!("{:09}", p));
    }
}

/// a magnitude in base 10**9 limbs, least significant first. Short ones
/// are peeled by division by that constant (a multiply for the compiler);
/// long ones are hi * (2**32)**k + lo with the arithmetic done in base
/// 10**9, so the conversion costs a few balanced Karatsuba products
fn to_dec(mag: &[u32], pows: &mut std::collections::HashMap<usize, Vec<u32>>) -> Vec<u32> {
    if mag.len() <= 2 * KARATSUBA_LIMBS {
        let mut pieces = vec![];
        let mut cur = mag.to_vec();
        trim(&mut cur);
        while !cur.is_empty() {
            let mut rem = 0u64;
            for x in cur.iter_mut().rev() {
                let t = (rem << 32) | *x as u64;
                *x = (t / DEC_BASE) as u32;
                rem = t % DEC_BASE;
            }
            trim(&mut cur);
            pieces.push(rem as u32);
        }
        return pieces;
    }
    // split at a power of two, so the powers are shared
    let k = 1usize << (usize::BITS - 1 - (mag.len() - 1).leading_zeros());
    let lo = to_dec(&mag[..k], pows);
    let hi = to_dec(&mag[k..], pows);
    let p = dec_pow_limbs(k, pows);
    let mut out = vec![0u32; hi.len() + p.len() + 1];
    dec_mul_into(&mut out, &hi, &p);
    dec_add_into(&mut out, &lo);
    trim(&mut out);
    out
}

/// (2**32)**k in base 10**9, for k a power of two
fn dec_pow_limbs(k: usize, pows: &mut std::collections::HashMap<usize, Vec<u32>>) -> Vec<u32> {
    if let Some(p) = pows.get(&k) {
        return p.clone();
    }
    let p = if k == 1 {
        let b = 1u64 << 32;
        vec![(b % DEC_BASE) as u32, (b / DEC_BASE) as u32]
    } else {
        let h = dec_pow_limbs(k / 2, pows);
        let mut sq = vec![0u32; 2 * h.len()];
        dec_mul_into(&mut sq, &h, &h);
        trim(&mut sq);
        sq
    };
    pows.insert(k, p.clone());
    p
}

/// out += a * b in base 10**9; out has room for a.len() + b.len() limbs
fn dec_mul_into(out: &mut [u32], a: &[u32], b: &[u32]) {
    let (a, b) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    if b.len() < KARATSUBA_LIMBS {
        for (i, &x) in b.iter().enumerate() {
            if x == 0 {
                continue;
            }
            let mut carry = 0u64;
            for (j, &y) in a.iter().enumerate() {
                let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
                out[i + j] = (t % DEC_BASE) as u32;
                carry = t / DEC_BASE;
            }
            dec_add_at(out, i + a.len(), carry);
        }
    } else if a.len() >= 2 * b.len() {
        for (i, piece) in a.chunks(b.len()).enumerate() {
            let at = i * b.len();
            dec_mul_into(&mut out[at..at + piece.len() + b.len()], piece, b);
        }
    } else {
        let half = a.len().div_ceil(2);
        let (a0, a1) = a.split_at(half);
        let (b0, b1) = b.split_at(half.min(b.len()));
        let mut z0 = vec![0u32; a0.len() + b0.len()];
        dec_mul_into(&mut z0, a0, b0);
        let mut z2 = vec![0u32; a1.len() + b1.len()];
        dec_mul_into(&mut z2, a1, b1);
        let mut sa = vec![0u32; half + 1];
        dec_add_into(&mut sa, a0);
        dec_add_into(&mut sa, a1);
        let mut sb = vec![0u32; half + 1];
        dec_add_into(&mut sb, b0);
        dec_add_into(&mut sb, b1);
        trim(&mut sa);
        trim(&mut sb);
        let mut z1 = vec![0u32; sa.len() + sb.len()];
        dec_mul_into(&mut z1, &sa, &sb);
        dec_sub_into(&mut z1, &z0);
        dec_sub_into(&mut z1, &z2);
        trim(&mut z0);
        trim(&mut z1);
        trim(&mut z2);
        dec_add_into(out, &z0);
        dec_add_into(&mut out[half..], &z1);
        dec_add_into(&mut out[2 * half..], &z2);
    }
}

fn dec_add_at(out: &mut [u32], mut at: usize, mut carry: u64) {
    while carry > 0 {
        let t = out[at] as u64 + carry;
        out[at] = (t % DEC_BASE) as u32;
        carry = t / DEC_BASE;
        at += 1;
    }
}

fn dec_add_into(out: &mut [u32], x: &[u32]) {
    let mut carry = 0u64;
    for (i, &v) in x.iter().enumerate() {
        let t = out[i] as u64 + v as u64 + carry;
        out[i] = (t % DEC_BASE) as u32;
        carry = t / DEC_BASE;
    }
    dec_add_at(out, x.len(), carry);
}

/// out -= x in base 10**9, where the result stays non-negative
fn dec_sub_into(out: &mut [u32], x: &[u32]) {
    let mut borrow = 0u64;
    for (i, o) in out.iter_mut().enumerate() {
        if i >= x.len() && borrow == 0 {
            break;
        }
        let sub = *x.get(i).unwrap_or(&0) as u64 + borrow;
        if (*o as u64) >= sub {
            *o = (*o as u64 - sub) as u32;
            borrow = 0;
        } else {
            *o = (*o as u64 + DEC_BASE - sub) as u32;
            borrow = 1;
        }
    }
}

/// decimal digits to a magnitude, halving recursively: hi * 10**len(lo) + lo,
/// so the products are large and balanced (Karatsuba)
fn decimal_to_mag(digits: &[u32]) -> Vec<u32> {
    fn pow10(n: usize, cache: &mut std::collections::HashMap<usize, Vec<u32>>) -> Vec<u32> {
        if let Some(p) = cache.get(&n) {
            return p.clone();
        }
        let p = if n <= 9 {
            vec![10u32.pow(n as u32)]
        } else {
            let h = pow10(n / 2, cache);
            let sq = mul_mag(&h, &h);
            if n % 2 == 1 { mul_mag(&sq, &[10]) } else { sq }
        };
        cache.insert(n, p.clone());
        p
    }
    fn go(d: &[u32], cache: &mut std::collections::HashMap<usize, Vec<u32>>) -> Vec<u32> {
        if d.len() <= 1000 {
            let mut mag: Vec<u32> = vec![];
            for chunk in d.chunks(9) {
                let mul = 10u64.pow(chunk.len() as u32);
                let mut carry = chunk.iter().fold(0u64, |acc, &x| acc * 10 + x as u64);
                for x in mag.iter_mut() {
                    let t = *x as u64 * mul + carry;
                    *x = t as u32;
                    carry = t >> 32;
                }
                if carry > 0 {
                    mag.push(carry as u32);
                }
            }
            return mag;
        }
        let lo_len = d.len() / 2;
        let (hi, lo) = d.split_at(d.len() - lo_len);
        let hi = go(hi, cache);
        let lo = go(lo, cache);
        let p = pow10(lo_len, cache);
        add_mag(&mul_mag(&hi, &p), &lo)
    }
    let mut cache = std::collections::HashMap::new();
    let mut m = go(digits, &mut cache);
    trim(&mut m);
    m
}

/// exact comparison of an int with a float; None when f is a NaN
pub fn cmp_i64_f64(x: i64, f: f64) -> Option<Ordering> {
    if x.unsigned_abs() < 1 << 53 {
        // exactly representable
        return (x as f64).partial_cmp(&f);
    }
    BigInt::from_i64(x).cmp_f64(f)
}

impl BigInt {
    pub fn zero() -> BigInt {
        BigInt { neg: false, mag: vec![] }
    }
    pub fn from_i64(v: i64) -> BigInt {
        let neg = v < 0;
        let mut m = v.unsigned_abs();
        let mut mag = Vec::new();
        while m > 0 {
            mag.push(m as u32);
            m >>= 32;
        }
        BigInt { neg, mag }
    }
    pub fn from_u128(v: u128, neg: bool) -> BigInt {
        let mut m = v;
        let mut mag = Vec::new();
        while m > 0 {
            mag.push(m as u32);
            m >>= 32;
        }
        BigInt { neg: neg && !mag.is_empty(), mag }
    }
    pub fn from_i128(v: i128) -> BigInt {
        BigInt::from_u128(v.unsigned_abs(), v < 0)
    }
    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }
    pub fn to_i64(&self) -> Option<i64> {
        if self.mag.len() > 2 {
            return None;
        }
        let mut m: u64 = 0;
        for (i, &x) in self.mag.iter().enumerate() {
            m |= (x as u64) << (32 * i);
        }
        if self.neg {
            if m <= (i64::MAX as u64) + 1 {
                Some((m as i128).wrapping_neg() as i64)
            } else {
                None
            }
        } else if m <= i64::MAX as u64 {
            Some(m as i64)
        } else {
            None
        }
    }
    pub fn to_f64(&self) -> f64 {
        // round correctly enough: use the top 64 bits and an exponent
        let bl = bit_len_mag(&self.mag);
        let v = if bl <= 64 {
            let mut m: u64 = 0;
            for (i, &x) in self.mag.iter().enumerate() {
                m |= (x as u64) << (32 * i);
            }
            m as f64
        } else {
            let shift = bl - 64;
            let top = shr_mag(&self.mag, shift);
            let mut m: u64 = 0;
            for (i, &x) in top.iter().enumerate() {
                m |= (x as u64) << (32 * i);
            }
            // sticky bit for rounding
            let sticky = self.mag.iter().take(shift / 32).any(|&x| x != 0) || (shift % 32 != 0 && self.mag[shift / 32] & ((1u32 << (shift % 32)) - 1) != 0);
            let m2 = if sticky { m | 1 } else { m };
            (m2 as f64) * 2f64.powi(shift as i32)
        };
        if self.neg {
            -v
        } else {
            v
        }
    }
    pub fn from_f64(f: f64) -> Option<BigInt> {
        if !f.is_finite() {
            return None;
        }
        let f = f.trunc();
        let neg = f < 0.0;
        let a = f.abs();
        if a < 1.8e19 {
            return Some(BigInt::from_u128(a as u128, neg));
        }
        let bits = a.to_bits();
        let exp = ((bits >> 52) & 0x7ff) as i64 - 1075;
        let mant = (bits & ((1 << 52) - 1)) | (1 << 52);
        let m = BigInt::from_u128(mant as u128, false);
        let mut r = if exp >= 0 { BigInt { neg: false, mag: shl_mag(&m.mag, exp as usize) } } else { BigInt { neg: false, mag: shr_mag(&m.mag, (-exp) as usize) } };
        r.neg = neg && !r.mag.is_empty();
        Some(r)
    }
    /// exact comparison with a float; None when f is a NaN
    pub fn cmp_f64(&self, f: f64) -> Option<Ordering> {
        if f.is_nan() {
            return None;
        }
        if f.is_infinite() {
            return Some(if f > 0.0 { Ordering::Less } else { Ordering::Greater });
        }
        // compare with the integer part, then let a fraction break a tie
        let fl = f.floor();
        Some(self.cmp(&BigInt::from_f64(fl)?).then(if f > fl { Ordering::Less } else { Ordering::Equal }))
    }
    pub fn from_radix(s: &str, radix: u32) -> Option<BigInt> {
        let s = s.trim();
        let (neg, digits) = if let Some(r) = s.strip_prefix('-') {
            (true, r)
        } else if let Some(r) = s.strip_prefix('+') {
            (false, r)
        } else {
            (false, s)
        };
        if digits.is_empty() {
            return None;
        }
        let mut vals = Vec::with_capacity(digits.len());
        for c in digits.chars() {
            if c != '_' {
                vals.push(c.to_digit(radix)?);
            }
        }
        let mut mag = if radix == 10 && vals.len() > 2000 {
            decimal_to_mag(&vals)
        } else {
            // mag = mag * radix**k + chunk, a few digits at a time
            let per = (32.0 / (radix as f64).log2()).floor() as usize;
            let mut mag: Vec<u32> = vec![];
            for chunk in vals.chunks(per) {
                let mul = (radix as u64).pow(chunk.len() as u32);
                let mut carry = chunk.iter().fold(0u64, |acc, &d| acc * radix as u64 + d as u64);
                for x in mag.iter_mut() {
                    let t = *x as u64 * mul + carry;
                    *x = t as u32;
                    carry = t >> 32;
                }
                if carry > 0 {
                    mag.push(carry as u32);
                }
            }
            mag
        };
        trim(&mut mag);
        Some(BigInt { neg: neg && !mag.is_empty(), mag })
    }
    pub fn to_radix(&self, radix: u32) -> String {
        if self.mag.is_empty() {
            return "0".into();
        }
        if radix == 10 {
            let mut out = String::with_capacity(self.mag.len() * 10 + 1);
            if self.neg {
                out.push('-');
            }
            mag_to_decimal(&self.mag, &mut out);
            return out;
        }
        let mut digits = Vec::new();
        let mut cur = self.mag.clone();
        // peel several digits at a time
        let (chunk, per) = match radix {
            16 => (1 << 28, 7),
            8 => (1 << 30, 10),
            2 => (1 << 31, 31),
            _ => (radix, 1),
        };
        while !cur.is_empty() {
            let (q, mut r) = divmod_small(&cur, chunk);
            cur = q;
            for _ in 0..per {
                digits.push(std::char::from_digit(r % radix, radix).unwrap());
                r /= radix;
                if cur.is_empty() && r == 0 {
                    break;
                }
            }
        }
        while digits.len() > 1 && *digits.last().unwrap() == '0' {
            digits.pop();
        }
        if self.neg {
            digits.push('-');
        }
        digits.iter().rev().collect()
    }
    pub fn neg(&self) -> BigInt {
        BigInt { neg: !self.neg && !self.mag.is_empty(), mag: self.mag.clone() }
    }
    pub fn abs(&self) -> BigInt {
        BigInt { neg: false, mag: self.mag.clone() }
    }
    pub fn add(&self, o: &BigInt) -> BigInt {
        if self.neg == o.neg {
            return BigInt { neg: self.neg, mag: add_mag(&self.mag, &o.mag) };
        }
        match cmp_mag(&self.mag, &o.mag) {
            Ordering::Equal => BigInt::zero(),
            Ordering::Greater => BigInt { neg: self.neg, mag: sub_mag(&self.mag, &o.mag) },
            Ordering::Less => BigInt { neg: o.neg, mag: sub_mag(&o.mag, &self.mag) },
        }
    }
    pub fn sub(&self, o: &BigInt) -> BigInt {
        self.add(&o.neg())
    }
    pub fn mul(&self, o: &BigInt) -> BigInt {
        let mag = mul_mag(&self.mag, &o.mag);
        BigInt { neg: (self.neg != o.neg) && !mag.is_empty(), mag }
    }
    /// floor division and modulo, Python semantics; None on division by zero
    pub fn divmod(&self, o: &BigInt) -> Option<(BigInt, BigInt)> {
        if o.mag.is_empty() {
            return None;
        }
        let (q, r) = divmod_mag(&self.mag, &o.mag);
        let mut q = BigInt { neg: (self.neg != o.neg) && !q.is_empty(), mag: q };
        let mut r = BigInt { neg: self.neg && !r.is_empty(), mag: r };
        if !r.mag.is_empty() && (r.neg != o.neg) {
            q = q.sub(&BigInt::from_i64(1));
            r = r.add(o);
        }
        Some((q, r))
    }
    pub fn pow(&self, mut e: u64) -> BigInt {
        let mut base = self.clone();
        let mut acc = BigInt::from_i64(1);
        while e > 0 {
            if e & 1 == 1 {
                acc = acc.mul(&base);
            }
            e >>= 1;
            if e > 0 {
                base = base.mul(&base);
            }
        }
        acc
    }
    pub fn cmp(&self, o: &BigInt) -> Ordering {
        match (self.neg, o.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => cmp_mag(&self.mag, &o.mag),
            (true, true) => cmp_mag(&o.mag, &self.mag),
        }
    }
    pub fn shl(&self, n: usize) -> BigInt {
        BigInt { neg: self.neg, mag: shl_mag(&self.mag, n) }
    }
    pub fn shr(&self, n: usize) -> BigInt {
        if !self.neg {
            return BigInt { neg: false, mag: shr_mag(&self.mag, n) };
        }
        // floor for negatives: -((|x| - 1) >> n) - 1
        let one = BigInt::from_i64(1);
        let m = self.abs().sub(&one);
        let s = BigInt { neg: false, mag: shr_mag(&m.mag, n) };
        s.add(&one).neg()
    }
    pub fn bit_length(&self) -> usize {
        bit_len_mag(&self.mag)
    }
    pub fn is_odd(&self) -> bool {
        self.mag.first().map(|x| x & 1 == 1).unwrap_or(false)
    }
    /// two's-complement limbs, `n` limbs long
    fn twos(&self, n: usize) -> Vec<u32> {
        let mut v = self.mag.clone();
        v.resize(n, 0);
        if self.neg {
            for x in v.iter_mut() {
                *x = !*x;
            }
            for x in v.iter_mut() {
                let (s, c) = x.overflowing_add(1);
                *x = s;
                if !c {
                    break;
                }
            }
        }
        v
    }
    fn from_twos(mut v: Vec<u32>) -> BigInt {
        let neg = v.last().map(|x| x >> 31 == 1).unwrap_or(false);
        if neg {
            for x in v.iter_mut() {
                *x = !*x;
            }
            for x in v.iter_mut() {
                let (s, c) = x.overflowing_add(1);
                *x = s;
                if !c {
                    break;
                }
            }
        }
        trim(&mut v);
        BigInt { neg: neg && !v.is_empty(), mag: v }
    }
    pub fn bitop(&self, o: &BigInt, op: char) -> BigInt {
        let n = self.mag.len().max(o.mag.len()) + 1;
        let a = self.twos(n);
        let b = o.twos(n);
        let r: Vec<u32> = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| match op {
                '&' => x & y,
                '|' => x | y,
                _ => x ^ y,
            })
            .collect();
        BigInt::from_twos(r)
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_radix(10))
    }
}

/// 2**e as an f64, for e in the normal range
fn pow2(e: i64) -> f64 {
    f64::from_bits(((e + 1023) as u64) << 52)
}

/// `a / b` correctly rounded, after CPython's long_true_divide; None when
/// the result is too large for a float. `b` must be non-zero.
pub fn true_divide(a: &BigInt, b: &BigInt) -> Option<f64> {
    const MANT: i64 = 53;
    const MIN_EXP: i64 = -1021;
    const MAX_EXP: i64 = 1024;
    let neg = a.neg != b.neg;
    let sign = |x: f64| if neg { -x } else { x };
    if a.is_zero() {
        return Some(sign(0.0));
    }
    let (aa, bb) = (a.abs(), b.abs());
    let (asz, bsz) = (aa.bit_length() as i64, bb.bit_length() as i64);
    if asz <= MANT && bsz <= MANT {
        return Some(sign(aa.to_f64() / bb.to_f64()));
    }
    let diff = asz - bsz;
    if diff > MAX_EXP {
        return None;
    }
    if diff < MIN_EXP - MANT - 1 {
        return Some(sign(0.0));
    }
    let shift = diff.max(MIN_EXP) - MANT - 2;
    let (x, d) = if shift <= 0 { (aa.shl((-shift) as usize), bb) } else { (aa, bb.shl(shift as usize)) };
    let (q, r) = x.divmod(&d).unwrap();
    let inexact = !r.is_zero();
    let mut qv: u64 = q.mag.iter().rev().fold(0u64, |acc, &w| (acc << 32) | w as u64);
    let qsz = q.bit_length() as i64;
    let extra = qsz.max(MIN_EXP - shift) - MANT;
    let mask = 1u64 << (extra - 1);
    let low = qv | inexact as u64;
    if low & mask != 0 && low & (3 * mask - 1) != 0 {
        qv += mask;
    }
    qv &= !(2 * mask - 1);
    let dx = qv as f64;
    if shift + qsz >= MAX_EXP && (shift + qsz > MAX_EXP || dx == pow2(qsz)) {
        return None;
    }
    // dx is exact, so the scaling below does not round
    let mut s = shift;
    let mut v = dx;
    while s < -1000 {
        v *= pow2(-1000);
        s += 1000;
    }
    while s > 1000 {
        v *= pow2(1000);
        s -= 1000;
    }
    Some(sign(v * pow2(s)))
}
