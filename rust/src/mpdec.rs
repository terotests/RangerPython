// SPDX-License-Identifier: MIT
//! mpdec: the decimal arithmetic behind the native _decimal module, after
//! libmpdec, the library CPython's C decimal is built on. The operations
//! follow libmpdec's step by step (the same special cases, the same
//! status flags, the same finalizing), so results and signals match the
//! C version's. A coefficient is a non-negative BigInt with its digit
//! count beside it; exponents are i64 as in a 64-bit libmpdec build.
//! exp, ln, log10 and non-integer powers use the integer kernels of the
//! pure-Python decimal (_dexp, _dlog, ...) inside libmpdec's loop for
//! correct rounding.

use crate::bigint::BigInt;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

pub const MAX_PREC: i64 = 999_999_999_999_999_999;
pub const MAX_EMAX: i64 = 999_999_999_999_999_999;
pub const MIN_EMIN: i64 = -999_999_999_999_999_999;
pub const MIN_ETINY: i64 = MIN_EMIN - (MAX_PREC - 1);
/// exponents past these give an infinity or zero when finalized
pub const EXP_INF: i64 = 2_000_000_000_000_000_001;
pub const EXP_CLAMP: i64 = -4_000_000_000_000_000_001;
/// a coefficient longer than this is where libmpdec's allocation would
/// fail: the operation reports MallocError instead of trying
const MAX_ALLOC: i64 = 1 << 26;

// the status flags, with libmpdec's values
pub const CLAMPED: u32 = 0x1;
pub const CONVERSION_SYNTAX: u32 = 0x2;
pub const DIVISION_BY_ZERO: u32 = 0x4;
pub const DIVISION_IMPOSSIBLE: u32 = 0x8;
pub const DIVISION_UNDEFINED: u32 = 0x10;
pub const FPU_ERROR: u32 = 0x20;
pub const INEXACT: u32 = 0x40;
pub const INVALID_CONTEXT: u32 = 0x80;
pub const INVALID_OPERATION: u32 = 0x100;
pub const MALLOC_ERROR: u32 = 0x200;
/// MPD_Not_implemented, which _decimal uses for FloatOperation
pub const FLOAT_OPERATION: u32 = 0x400;
pub const OVERFLOW: u32 = 0x800;
pub const ROUNDED: u32 = 0x1000;
pub const SUBNORMAL: u32 = 0x2000;
pub const UNDERFLOW: u32 = 0x4000;
pub const IEEE_INVALID: u32 = CONVERSION_SYNTAX | DIVISION_IMPOSSIBLE | DIVISION_UNDEFINED | FPU_ERROR | INVALID_CONTEXT | INVALID_OPERATION | MALLOC_ERROR;
pub const ERRORS: u32 = IEEE_INVALID | DIVISION_BY_ZERO;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Round {
    Up,
    Down,
    Ceiling,
    Floor,
    HalfUp,
    HalfDown,
    HalfEven,
    R05Up,
    Trunc,
}

impl Round {
    pub const ALL: [Round; 8] = [Round::Up, Round::Down, Round::Ceiling, Round::Floor, Round::HalfUp, Round::HalfDown, Round::HalfEven, Round::R05Up];
    pub fn name(self) -> &'static str {
        match self {
            Round::Up => "ROUND_UP",
            Round::Down => "ROUND_DOWN",
            Round::Ceiling => "ROUND_CEILING",
            Round::Floor => "ROUND_FLOOR",
            Round::HalfUp => "ROUND_HALF_UP",
            Round::HalfDown => "ROUND_HALF_DOWN",
            Round::HalfEven => "ROUND_HALF_EVEN",
            Round::R05Up => "ROUND_05UP",
            Round::Trunc => "ROUND_TRUNC",
        }
    }
}

/// what an operation reads from a context; traps and flags stay with the
/// caller, which collects each operation's status
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    pub prec: i64,
    pub emax: i64,
    pub emin: i64,
    pub round: Round,
    pub clamp: bool,
    pub allcr: bool,
}

impl Ctx {
    pub fn max() -> Ctx {
        Ctx { prec: MAX_PREC, emax: MAX_EMAX, emin: MIN_EMIN, round: Round::HalfEven, clamp: false, allcr: true }
    }
    pub fn etiny(&self) -> i64 {
        self.emin - self.prec + 1
    }
    pub fn etop(&self) -> i64 {
        self.emax - self.prec + 1
    }
    fn with_round(&self, round: Round) -> Ctx {
        Ctx { round, ..self.clone() }
    }
}

// -------------------------------------------------------------------------
// coefficients

crate::gil_static! {
    static POW10: RefCell<HashMap<i64, BigInt>> = RefCell::new(HashMap::new());
}

fn big(v: u128) -> BigInt {
    BigInt::from_u128(v, false)
}

fn one() -> BigInt {
    BigInt::from_i64(1)
}

/// the value when it fits in a u128
fn small(c: &BigInt) -> Option<u128> {
    if c.mag.len() > 4 {
        return None;
    }
    Some(c.mag.iter().rev().fold(0u128, |acc, &x| (acc << 32) | x as u128))
}

pub fn pow10(n: i64) -> BigInt {
    if n < 39 {
        return big(10u128.pow(n.max(0) as u32));
    }
    if let Some(p) = POW10.with(|c| c.borrow().get(&n).cloned()) {
        return p;
    }
    let h = pow10(n / 2);
    let mut p = h.mul(&h);
    if n % 2 == 1 {
        p = p.mul(&big(10));
    }
    POW10.with(|c| c.borrow_mut().insert(n, p.clone()));
    p
}

/// the number of decimal digits of c (1 for zero)
pub fn ndigits(c: &BigInt) -> i64 {
    if let Some(v) = small(c) {
        return if v == 0 { 1 } else { v.ilog10() as i64 + 1 };
    }
    let bits = c.bit_length() as i64;
    let mut d = ((bits - 1) as f64 * std::f64::consts::LOG10_2) as i64 + 1;
    while c.cmp(&pow10(d)) != Ordering::Less {
        d += 1;
    }
    while d > 1 && c.cmp(&pow10(d - 1)) == Ordering::Less {
        d -= 1;
    }
    d
}

/// c * 10**n
fn shl(c: &BigInt, n: i64) -> BigInt {
    if n <= 0 || c.is_zero() {
        return c.clone();
    }
    if let Some(v) = small(c) {
        if n < 39 {
            if let Some(r) = v.checked_mul(10u128.pow(n as u32)) {
                return big(r);
            }
        }
    }
    c.mul(&pow10(n))
}

/// the rounding indicator of dropped digits: the first of them, plus one
/// when that is 0 or 5 and any other is nonzero
fn indicator(d: u8, rest: bool) -> u8 {
    if (d == 0 || d == 5) && rest {
        d + 1
    } else {
        d
    }
}

/// c // 10**n and the rounding indicator of what was dropped; `digits` is
/// c's digit count
fn shr(c: &BigInt, digits: i64, n: i64) -> (BigInt, u8) {
    if n <= 0 || c.is_zero() {
        return (c.clone(), 0);
    }
    if let Some(v) = small(c) {
        if n <= 38 {
            let p = 10u128.pow(n as u32);
            let (q, r) = (v / p, v % p);
            let p1 = p / 10;
            return (big(q), indicator((r / p1) as u8, r % p1 != 0));
        }
    }
    if n >= digits {
        if n == digits {
            let (d, rest) = c.divmod(&pow10(digits - 1)).unwrap();
            return (BigInt::zero(), indicator(d.to_i64().unwrap_or(0) as u8, !rest.is_zero()));
        }
        return (BigInt::zero(), 1);
    }
    let (q, r) = c.divmod(&pow10(n)).unwrap();
    let (d, rest) = r.divmod(&pow10(n - 1)).unwrap();
    (q, indicator(d.to_i64().unwrap_or(0) as u8, !rest.is_zero()))
}

/// the number of trailing zero digits (0 for zero)
fn trail_zeros(c: &BigInt) -> i64 {
    if c.is_zero() {
        return 0;
    }
    if let Some(mut v) = small(c) {
        let mut n = 0;
        while v % 10 == 0 {
            v /= 10;
            n += 1;
        }
        return n;
    }
    let s = c.to_radix(10);
    (s.len() - s.trim_end_matches('0').len()) as i64
}

/// c with only its last n digits
fn low_digits(c: &BigInt, n: i64) -> BigInt {
    if n >= ndigits(c) {
        return c.clone();
    }
    c.divmod(&pow10(n)).unwrap().1
}

fn digits_str(c: &BigInt) -> String {
    match small(c) {
        Some(v) => v.to_string(),
        None => c.to_radix(10),
    }
}

// -------------------------------------------------------------------------
// decimals

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Fin,
    Inf,
    NaN,
    SNaN,
}

/// sign, coefficient, exponent; a NaN's coefficient is its payload, with
/// zero digits when it has none
#[derive(Clone, Debug)]
pub struct Dec {
    pub sign: bool,
    pub kind: Kind,
    pub exp: i64,
    pub coef: BigInt,
    pub digits: i64,
}

impl Dec {
    pub fn fin(sign: bool, coef: BigInt, exp: i64) -> Dec {
        let digits = ndigits(&coef);
        Dec { sign, kind: Kind::Fin, exp, coef, digits }
    }
    pub fn triple(sign: bool, c: u64, exp: i64) -> Dec {
        Dec::fin(sign, big(c as u128), exp)
    }
    pub fn special(sign: bool, kind: Kind) -> Dec {
        Dec { sign, kind, exp: 0, coef: BigInt::zero(), digits: 0 }
    }
    pub fn nan() -> Dec {
        Dec::special(false, Kind::NaN)
    }
    pub fn inf(sign: bool) -> Dec {
        Dec::special(sign, Kind::Inf)
    }
    pub fn from_i64(v: i64) -> Dec {
        Dec::fin(v < 0, big(v.unsigned_abs() as u128), 0)
    }
    /// an int, exactly
    pub fn from_bigint(v: &BigInt) -> Dec {
        Dec::fin(v.neg, v.abs(), 0)
    }
    pub fn is_special(&self) -> bool {
        self.kind != Kind::Fin
    }
    pub fn is_nan(&self) -> bool {
        matches!(self.kind, Kind::NaN | Kind::SNaN)
    }
    pub fn is_snan(&self) -> bool {
        self.kind == Kind::SNaN
    }
    pub fn is_qnan(&self) -> bool {
        self.kind == Kind::NaN
    }
    pub fn is_inf(&self) -> bool {
        self.kind == Kind::Inf
    }
    pub fn is_finite(&self) -> bool {
        self.kind == Kind::Fin
    }
    /// a finite zero
    pub fn is_zero(&self) -> bool {
        self.kind == Kind::Fin && self.coef.is_zero()
    }
    fn zero_coef(&self) -> bool {
        self.coef.is_zero()
    }
    pub fn adjexp(&self) -> i64 {
        self.exp + self.digits - 1
    }
    fn lsd(&self) -> u32 {
        // 2**(32*k) ends in 6 for every k > 0
        let m = &self.coef.mag;
        let rest: u64 = m.iter().skip(1).map(|&x| x as u64 % 10).sum();
        ((m.first().map_or(0, |&x| x as u64 % 10) + 6 * rest) % 10) as u32
    }
    fn set_coef(&mut self, c: BigInt) {
        self.digits = ndigits(&c);
        self.coef = c;
    }
    fn set_zero(&mut self) {
        self.coef = BigInt::zero();
        self.digits = 1;
    }
    fn with_sign(&self, sign: bool) -> Dec {
        Dec { sign, ..self.clone() }
    }
    fn quiet(&self) -> Dec {
        Dec { kind: Kind::NaN, ..self.clone() }
    }
    /// an integral value (libmpdec's _mpd_isint); finite only
    fn is_int(&self) -> bool {
        self.zero_coef() || self.exp >= 0 || self.exp + trail_zeros(&self.coef) >= 0
    }
    pub fn is_integer(&self) -> bool {
        self.is_finite() && self.is_int()
    }
    /// an odd integer; the value must be integral
    fn is_odd(&self) -> bool {
        if self.zero_coef() || self.exp > 0 {
            return false;
        }
        if self.exp == 0 {
            return self.lsd() % 2 == 1;
        }
        let (q, _) = shr(&self.coef, self.digits, -self.exp);
        q.is_odd()
    }
    pub fn is_normal(&self, ctx: &Ctx) -> bool {
        self.is_finite() && !self.zero_coef() && self.adjexp() >= ctx.emin
    }
    pub fn is_subnormal(&self, ctx: &Ctx) -> bool {
        self.is_finite() && !self.zero_coef() && self.adjexp() < ctx.emin
    }
    /// the integer value of an integral finite decimal
    pub fn int_value(&self) -> BigInt {
        let c = if self.exp >= 0 { shl(&self.coef, self.exp) } else { shr(&self.coef, self.digits, -self.exp).0 };
        if self.sign {
            c.neg()
        } else {
            c
        }
    }
    /// a power of ten coefficient
    fn coef_is_pow10(&self) -> bool {
        self.coef.cmp(&pow10(self.digits - 1)) == Ordering::Equal
    }
    fn coef_is_all_nine(&self) -> bool {
        self.coef.add(&one()).cmp(&pow10(self.digits)) == Ordering::Equal
    }
}

/// a quiet NaN, with the error flags set
fn seterror(st: &mut u32, flags: u32) -> Dec {
    *st |= flags;
    Dec::nan()
}

fn too_big(digits: i64) -> bool {
    digits > MAX_ALLOC
}

/// a shifted left by n digits, or a MallocError where libmpdec's
/// allocation would fail
fn shiftl(a: &Dec, n: i64, st: &mut u32) -> Option<Dec> {
    if !a.zero_coef() && too_big(a.digits + n) {
        *st |= MALLOC_ERROR;
        return None;
    }
    let c = shl(&a.coef, n);
    let digits = if a.zero_coef() { 1 } else { a.digits + n.max(0) };
    Some(Dec { coef: c, digits, ..a.clone() })
}

/// a shifted right by n digits, and the rounding indicator
fn shiftr(a: &Dec, n: i64) -> (Dec, u8) {
    let (q, rnd) = shr(&a.coef, a.digits, n);
    let mut r = a.clone();
    if n >= a.digits {
        r.set_zero();
    } else if n > 0 {
        r.coef = q;
        r.digits = a.digits - n;
    }
    (r, rnd)
}

// -------------------------------------------------------------------------
// NaNs, rounding and finalizing

/// shortens a NaN payload to what the context can hold
fn fix_nan(r: &mut Dec, ctx: &Ctx) {
    let prec = ctx.prec - ctx.clamp as i64;
    if r.digits > 0 && r.digits > prec {
        if prec == 0 {
            r.coef = BigInt::zero();
            r.digits = 0;
        } else {
            let c = low_digits(&r.coef, prec);
            if c.is_zero() {
                r.coef = c;
                r.digits = 0;
            } else {
                r.set_coef(c);
            }
        }
    }
}

fn check_nan(a: &Dec, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if a.is_nan() {
        if a.is_snan() {
            *st |= INVALID_OPERATION;
        }
        let mut r = a.quiet();
        fix_nan(&mut r, ctx);
        return Some(r);
    }
    None
}

fn check_nans(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if a.is_nan() || b.is_nan() {
        let mut choice = b;
        if a.is_snan() {
            choice = a;
            *st |= INVALID_OPERATION;
        } else if b.is_snan() {
            *st |= INVALID_OPERATION;
        } else if a.is_qnan() {
            choice = a;
        }
        let mut r = choice.quiet();
        fix_nan(&mut r, ctx);
        return Some(r);
    }
    None
}

fn check_3nans(a: &Dec, b: &Dec, c: &Dec, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if a.is_nan() || b.is_nan() || c.is_nan() {
        let mut choice = c;
        if a.is_snan() {
            choice = a;
            *st |= INVALID_OPERATION;
        } else if b.is_snan() {
            choice = b;
            *st |= INVALID_OPERATION;
        } else if c.is_snan() {
            *st |= INVALID_OPERATION;
        } else if a.is_qnan() {
            choice = a;
        } else if b.is_qnan() {
            choice = b;
        }
        let mut r = choice.quiet();
        fix_nan(&mut r, ctx);
        return Some(r);
    }
    None
}

/// whether rounding indicator rnd rounds d's (already shortened)
/// coefficient up
fn rnd_incr(d: &Dec, rnd: u8, ctx: &Ctx) -> bool {
    match ctx.round {
        Round::Down | Round::Trunc => false,
        Round::HalfUp => rnd >= 5,
        Round::HalfEven => rnd > 5 || (rnd == 5 && d.lsd() % 2 == 1),
        Round::Ceiling => !(rnd == 0 || d.sign),
        Round::Floor => !(rnd == 0 || !d.sign),
        Round::HalfDown => rnd > 5,
        Round::Up => rnd != 0,
        Round::R05Up => {
            let ld = d.lsd();
            rnd != 0 && (ld == 0 || ld == 5)
        }
    }
}

fn incr(d: &mut Dec) {
    let c = d.coef.add(&one());
    d.set_coef(c);
}

/// rounds a coefficient that has exactly prec digits
fn apply_round(d: &mut Dec, rnd: u8, ctx: &Ctx, st: &mut u32) {
    if rnd_incr(d, rnd, ctx) {
        incr(d);
        if d.digits > ctx.prec {
            let (q, _) = shr(&d.coef, d.digits, 1);
            d.coef = q;
            d.digits = ctx.prec;
            d.exp += 1;
            check_exp(d, ctx, st);
        }
    }
}

/// rounds, letting the coefficient grow past prec
fn apply_round_excess(d: &mut Dec, rnd: u8, ctx: &Ctx) {
    if rnd_incr(d, rnd, ctx) {
        incr(d);
    }
}

/// rounds a coefficient of at most prec digits; false (and a NaN) when the
/// increment would not fit
fn apply_round_fit(d: &mut Dec, rnd: u8, ctx: &Ctx, st: &mut u32) -> bool {
    if rnd_incr(d, rnd, ctx) {
        incr(d);
        if d.digits > ctx.prec {
            *d = seterror(st, INVALID_OPERATION);
            return false;
        }
    }
    true
}

/// 10**prec - 1: the largest coefficient
fn maxcoeff(d: &mut Dec, ctx: &Ctx, st: &mut u32) {
    if too_big(ctx.prec) {
        *d = seterror(st, MALLOC_ERROR);
        return;
    }
    d.coef = pow10(ctx.prec).sub(&one());
    d.digits = ctx.prec;
}

/// overflow, clamping and subnormals (libmpdec's _mpd_check_exp)
fn check_exp(d: &mut Dec, ctx: &Ctx, st: &mut u32) {
    let adjexp = d.adjexp();
    if adjexp > ctx.emax {
        if d.zero_coef() {
            d.exp = ctx.emax;
            if ctx.clamp {
                d.exp -= ctx.prec - 1;
            }
            d.set_zero();
            *st |= CLAMPED;
            return;
        }
        let largest = match ctx.round {
            Round::HalfUp | Round::HalfEven | Round::HalfDown | Round::Up | Round::Trunc => false,
            Round::Down | Round::R05Up => true,
            Round::Ceiling => d.sign,
            Round::Floor => !d.sign,
        };
        if largest {
            maxcoeff(d, ctx, st);
            d.exp = ctx.emax - ctx.prec + 1;
        } else {
            *d = Dec::inf(d.sign);
        }
        *st |= OVERFLOW | INEXACT | ROUNDED;
    } else if ctx.clamp && d.exp > ctx.etop() {
        // fold down
        let shift = d.exp - ctx.etop();
        match shiftl(d, shift, st) {
            Some(r) => *d = r,
            None => {
                *d = Dec::nan();
                return;
            }
        }
        d.exp -= shift;
        *st |= CLAMPED;
        if !d.zero_coef() && adjexp < ctx.emin {
            *st |= SUBNORMAL;
        }
    } else if adjexp < ctx.emin {
        let etiny = ctx.etiny();
        if d.zero_coef() {
            if d.exp < etiny {
                d.exp = etiny;
                d.set_zero();
                *st |= CLAMPED;
            }
            return;
        }
        *st |= SUBNORMAL;
        if d.exp < etiny {
            let shift = etiny - d.exp;
            let (r, rnd) = shiftr(d, shift);
            *d = r;
            d.exp = etiny;
            apply_round_excess(d, rnd, ctx);
            *st |= ROUNDED;
            if rnd != 0 {
                *st |= INEXACT | UNDERFLOW;
                if d.zero_coef() {
                    d.set_zero();
                    *st |= CLAMPED;
                }
            }
        }
    }
}

/// Underflow for results of the transcendental functions, which may be
/// exact enough for no rounding digit to show it
fn check_underflow(d: &Dec, ctx: &Ctx, st: &mut u32) {
    if d.is_finite() && d.adjexp() < ctx.emin && !d.zero_coef() && d.exp < ctx.etiny() {
        *st |= UNDERFLOW;
    }
}

fn check_round(d: &mut Dec, ctx: &Ctx, st: &mut u32) {
    if d.is_special() {
        return;
    }
    if d.digits > ctx.prec {
        let shift = d.digits - ctx.prec;
        let (r, rnd) = shiftr(d, shift);
        *d = r;
        d.exp += shift;
        apply_round(d, rnd, ctx, st);
        *st |= ROUNDED;
        if rnd != 0 {
            *st |= INEXACT;
        }
    }
}

/// fits a result to the context: what every operation does last
pub fn finalize(d: &mut Dec, ctx: &Ctx, st: &mut u32) {
    match d.kind {
        Kind::NaN | Kind::SNaN => fix_nan(d, ctx),
        Kind::Inf => {}
        Kind::Fin => {
            check_exp(d, ctx, st);
            check_round(d, ctx, st);
        }
    }
}

fn finalized(mut d: Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    finalize(&mut d, ctx, st);
    d
}

/// the most significant digits beyond prec cut off (libmpdec's _mpd_cap)
fn cap(d: &mut Dec, ctx: &Ctx) {
    if d.digits > ctx.prec {
        let c = low_digits(&d.coef, ctx.prec);
        d.set_coef(c);
    }
}

// -------------------------------------------------------------------------
// comparisons

fn sgn(o: Ordering) -> i32 {
    match o {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn arith_sign(d: &Dec) -> i32 {
    if d.sign {
        -1
    } else {
        1
    }
}

/// compares the coefficients of a and b, which have the same adjusted exponent
fn cmp_same_adjexp(a: &Dec, b: &Dec) -> i32 {
    let o = match a.exp.cmp(&b.exp) {
        Ordering::Equal => a.coef.cmp(&b.coef),
        Ordering::Greater => shl(&a.coef, a.exp - b.exp).cmp(&b.coef),
        Ordering::Less => a.coef.cmp(&shl(&b.coef, b.exp - a.exp)),
    };
    sgn(o)
}

/// numeric comparison of non-NaNs
pub fn cmp(a: &Dec, b: &Dec) -> i32 {
    if a.is_inf() {
        if b.is_inf() {
            return b.sign as i32 - a.sign as i32;
        }
        return arith_sign(a);
    }
    if b.is_inf() {
        return -arith_sign(b);
    }
    if a.zero_coef() {
        if b.zero_coef() {
            return 0;
        }
        return -arith_sign(b);
    }
    if b.zero_coef() {
        return arith_sign(a);
    }
    if a.sign != b.sign {
        return b.sign as i32 - a.sign as i32;
    }
    let (ea, eb) = (a.adjexp(), b.adjexp());
    if ea != eb {
        return if ea < eb { -arith_sign(a) } else { arith_sign(a) };
    }
    cmp_same_adjexp(a, b) * arith_sign(a)
}

pub fn cmp_abs(a: &Dec, b: &Dec) -> i32 {
    if a.is_inf() {
        return if b.is_inf() { 0 } else { 1 };
    }
    if b.is_inf() {
        return -1;
    }
    if a.zero_coef() {
        return if b.zero_coef() { 0 } else { -1 };
    }
    if b.zero_coef() {
        return 1;
    }
    let (ea, eb) = (a.adjexp(), b.adjexp());
    if ea != eb {
        return if ea < eb { -1 } else { 1 };
    }
    cmp_same_adjexp(a, b)
}

/// numeric comparison; None (and InvalidOperation) for a NaN
pub fn qcmp(a: &Dec, b: &Dec, st: &mut u32) -> Option<i32> {
    if a.is_nan() || b.is_nan() {
        *st |= INVALID_OPERATION;
        return None;
    }
    Some(cmp(a, b))
}

fn cmp_result(c: i32) -> Dec {
    Dec::triple(c < 0, (c != 0) as u64, 0)
}

pub fn compare(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nans(a, b, ctx, st) {
        return r;
    }
    cmp_result(cmp(a, b))
}

pub fn compare_signal(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nans(a, b, ctx, st) {
        *st |= INVALID_OPERATION;
        return r;
    }
    cmp_result(cmp(a, b))
}

pub fn cmp_total(a: &Dec, b: &Dec) -> i32 {
    if a.sign != b.sign {
        return b.sign as i32 - a.sign as i32;
    }
    let c = if a.is_nan() {
        if b.is_nan() {
            let (na, nb) = (a.is_qnan() as i32, b.is_qnan() as i32);
            if na == nb {
                if a.digits > 0 && b.digits > 0 {
                    // compare the payloads
                    let aa = Dec { kind: Kind::Fin, exp: 0, ..a.clone() };
                    let bb = Dec { kind: Kind::Fin, exp: 0, ..b.clone() };
                    cmp_abs(&aa, &bb)
                } else {
                    (a.digits > 0) as i32 - (b.digits > 0) as i32
                }
            } else {
                na - nb
            }
        } else {
            1
        }
    } else if b.is_nan() {
        -1
    } else {
        let c = cmp_abs(a, b);
        if c == 0 && a.exp != b.exp {
            if a.exp < b.exp {
                -1
            } else {
                1
            }
        } else {
            c
        }
    };
    c * arith_sign(a)
}

pub fn cmp_total_mag(a: &Dec, b: &Dec) -> i32 {
    cmp_total(&a.with_sign(false), &b.with_sign(false))
}

pub fn compare_total(a: &Dec, b: &Dec) -> Dec {
    cmp_result(cmp_total(a, b))
}

pub fn compare_total_mag(a: &Dec, b: &Dec) -> Dec {
    cmp_result(cmp_total_mag(a, b))
}

/// an order for numerically equal operands
fn cmp_numequal(a: &Dec, b: &Dec) -> i32 {
    if a.sign != b.sign {
        b.sign as i32 - a.sign as i32
    } else {
        (if a.exp < b.exp { -1 } else { 1 }) * arith_sign(a)
    }
}

fn minmax(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32, mag: bool, max: bool) -> Dec {
    let r = if a.is_qnan() && !b.is_nan() {
        b.clone()
    } else if b.is_qnan() && !a.is_nan() {
        a.clone()
    } else if let Some(r) = check_nans(a, b, ctx, st) {
        return r;
    } else {
        let mut c = if mag { cmp_abs(a, b) } else { cmp(a, b) };
        if c == 0 {
            c = cmp_numequal(a, b);
        }
        if (c < 0) == max {
            b.clone()
        } else {
            a.clone()
        }
    };
    finalized(r, ctx, st)
}

pub fn max(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    minmax(a, b, ctx, st, false, true)
}

pub fn max_mag(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    minmax(a, b, ctx, st, true, true)
}

pub fn min(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    minmax(a, b, ctx, st, false, false)
}

pub fn min_mag(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    minmax(a, b, ctx, st, true, false)
}

// -------------------------------------------------------------------------
// arithmetic

pub fn copy_abs(a: &Dec) -> Dec {
    a.with_sign(false)
}

pub fn copy_negate(a: &Dec) -> Dec {
    a.with_sign(!a.sign)
}

pub fn copy_sign(a: &Dec, b: &Dec) -> Dec {
    a.with_sign(b.sign)
}

pub fn minus(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nan(a, ctx, st) {
        return r;
    }
    let r = if a.is_zero() && ctx.round != Round::Floor { copy_abs(a) } else { copy_negate(a) };
    finalized(r, ctx, st)
}

pub fn plus(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nan(a, ctx, st) {
        return r;
    }
    let r = if a.is_zero() && ctx.round != Round::Floor { copy_abs(a) } else { a.clone() };
    finalized(r, ctx, st)
}

pub fn abs(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nan(a, ctx, st) {
        return r;
    }
    if a.sign {
        minus(a, ctx, st)
    } else {
        plus(a, ctx, st)
    }
}

fn addsub_inf(a: &Dec, b: &Dec, sign_b: bool, st: &mut u32) -> Dec {
    if a.is_inf() {
        if a.sign != sign_b && b.is_inf() {
            return seterror(st, INVALID_OPERATION);
        }
        return Dec::inf(a.sign);
    }
    Dec::inf(sign_b)
}

/// a + b with b's sign taken as sign_b, finite operands, not finalized
fn addsub(a: &Dec, b: &Dec, sign_b: bool, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut ac = a.coef.clone();
    let mut bc = b.coef.clone();
    let mut exp = a.exp.min(b.exp);
    if a.exp != b.exp {
        let a_big = a.exp > b.exp;
        let (bg, sm) = if a_big { (a, b) } else { (b, a) };
        if !bg.zero_coef() {
            // a small operand far below big's precision only matters as a
            // sticky digit: replace it with 0 or 1 just below big's digits
            let mut e = bg.exp - 1;
            e += if bg.digits > ctx.prec { 0 } else { bg.digits - ctx.prec - 1 };
            let (sc, se) = if sm.adjexp() < e { (if sm.zero_coef() { BigInt::zero() } else { one() }, e) } else { (sm.coef.clone(), sm.exp) };
            let shift = bg.exp - se;
            if too_big(bg.digits + shift) {
                return seterror(st, MALLOC_ERROR);
            }
            let bigc = shl(&bg.coef, shift);
            exp = se;
            if a_big {
                ac = bigc;
                bc = sc;
            } else {
                bc = bigc;
                ac = sc;
            }
        }
    }
    if a.sign == sign_b {
        return Dec::fin(sign_b, ac.add(&bc), exp);
    }
    match ac.cmp(&bc) {
        Ordering::Greater => Dec::fin(a.sign, ac.sub(&bc), exp),
        Ordering::Less => Dec::fin(sign_b, bc.sub(&ac), exp),
        Ordering::Equal => Dec::fin(ctx.round == Round::Floor, BigInt::zero(), exp),
    }
}

pub fn add(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
        return addsub_inf(a, b, b.sign, st);
    }
    finalized(addsub(a, b, b.sign, ctx, st), ctx, st)
}

pub fn sub(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
        return addsub_inf(a, b, !b.sign, st);
    }
    finalized(addsub(a, b, !b.sign, ctx, st), ctx, st)
}

/// a * b, not finalized
fn mul_raw(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
        let (inf, other) = if a.is_inf() { (a, b) } else { (b, a) };
        if other.is_zero() {
            return seterror(st, INVALID_OPERATION);
        }
        return Dec::inf(inf.sign != other.sign);
    }
    if !a.zero_coef() && !b.zero_coef() && too_big(a.digits + b.digits) {
        return seterror(st, MALLOC_ERROR);
    }
    Dec::fin(a.sign != b.sign, a.coef.mul(&b.coef), a.exp + b.exp)
}

pub fn mul(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    finalized(mul_raw(a, b, ctx, st), ctx, st)
}

/// a * b + c with one rounding
pub fn fma(a: &Dec, b: &Dec, c: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut ws = 0;
    let mut r = mul_raw(a, b, ctx, &mut ws);
    if ws & INVALID_OPERATION == 0 {
        r = add(&r, c, ctx, &mut ws);
    }
    *st |= ws;
    r
}

fn div_inf(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_inf() {
        if b.is_inf() {
            return seterror(st, INVALID_OPERATION);
        }
        return Dec::inf(a.sign != b.sign);
    }
    *st |= CLAMPED;
    Dec::triple(a.sign != b.sign, 0, ctx.etiny())
}

/// a / b; with `ideal` an exact quotient gets the ideal exponent
fn div_raw(ideal: bool, a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let sign = a.sign != b.sign;
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
        return div_inf(a, b, ctx, st);
    }
    if b.zero_coef() {
        if a.zero_coef() {
            return seterror(st, DIVISION_UNDEFINED);
        }
        *st |= DIVISION_BY_ZERO;
        return Dec::inf(sign);
    }
    if a.zero_coef() {
        return finalized(Dec::triple(sign, 0, a.exp - b.exp), ctx, st);
    }
    let shift = (b.digits - a.digits) + ctx.prec + 1;
    let ideal_exp = a.exp - b.exp;
    let mut exp = ideal_exp - shift;
    let (ac, bc) = if shift > 0 {
        if too_big(a.digits + shift) {
            return seterror(st, MALLOC_ERROR);
        }
        (shl(&a.coef, shift), b.coef.clone())
    } else if shift < 0 {
        if too_big(b.digits - shift) {
            return seterror(st, MALLOC_ERROR);
        }
        (a.coef.clone(), shl(&b.coef, -shift))
    } else {
        (a.coef.clone(), b.coef.clone())
    };
    let (qc, rc) = ac.divmod(&bc).unwrap();
    let mut q = Dec::fin(sign, qc, exp);
    if !rc.is_zero() {
        // a sticky digit for the rounding
        let ld = q.lsd();
        if ld == 0 || ld == 5 {
            incr(&mut q);
        }
    } else if ideal && ideal_exp - exp > 0 {
        let tz = trail_zeros(&q.coef);
        let s = tz.min(ideal_exp - exp);
        q = shiftr(&q, s).0;
        exp += s;
    }
    q.exp = exp;
    finalized(q, ctx, st)
}

pub fn div(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut xs = 0;
    let q = div_raw(true, a, b, ctx, &mut xs);
    if xs & (MALLOC_ERROR | DIVISION_IMPOSSIBLE) != 0 {
        // a full precision quotient is too large: retry with the
        // precision an exact quotient needs at most
        let wprec = a.digits + b.digits * 4;
        if wprec >= ctx.prec {
            *st |= xs & ERRORS;
            return q;
        }
        let w = Ctx { prec: wprec, ..ctx.clone() };
        let mut ys = 0;
        let q = div_raw(true, a, b, &w, &mut ys);
        if ys != 0 {
            let ys = *st | ((ys | xs) & ERRORS);
            return seterror(st, ys);
        }
        return q;
    }
    *st |= xs;
    q
}

fn nan_pair() -> (Dec, Dec) {
    (Dec::nan(), Dec::nan())
}

/// integer quotient and remainder of finite operands, b nonzero
fn divmod_raw(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> (Dec, Dec) {
    let sign_ab = a.sign != b.sign;
    let ideal_exp = a.exp.min(b.exp);
    if a.zero_coef() {
        let r = Dec { exp: ideal_exp, ..a.clone() };
        return (Dec::triple(sign_ab, 0, 0), r);
    }
    let expdiff = a.adjexp() - b.adjexp();
    if expdiff < 0 {
        let r = if a.exp > b.exp {
            match shiftl(a, a.exp - b.exp, st) {
                Some(r) => Dec { exp: ideal_exp, ..r },
                None => return nan_pair(),
            }
        } else {
            a.clone()
        };
        return (Dec::triple(sign_ab, 0, 0), r);
    }
    if expdiff > ctx.prec {
        *st |= DIVISION_IMPOSSIBLE;
        return nan_pair();
    }
    let (mut ac, mut bc) = (a.coef.clone(), b.coef.clone());
    if a.exp > b.exp {
        if too_big(a.digits + a.exp - b.exp) {
            *st |= MALLOC_ERROR;
            return nan_pair();
        }
        ac = shl(&ac, a.exp - b.exp);
    } else if a.exp < b.exp {
        if too_big(b.digits + b.exp - a.exp) {
            *st |= MALLOC_ERROR;
            return nan_pair();
        }
        bc = shl(&bc, b.exp - a.exp);
    }
    let (qc, rc) = ac.divmod(&bc).unwrap();
    let q = Dec::fin(sign_ab, qc, 0);
    if q.digits > ctx.prec {
        *st |= DIVISION_IMPOSSIBLE;
        return nan_pair();
    }
    (q, Dec::fin(a.sign, rc, ideal_exp))
}

pub fn divmod(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> (Dec, Dec) {
    let sign = a.sign != b.sign;
    if a.is_special() || b.is_special() {
        if let Some(q) = check_nans(a, b, ctx, st) {
            return (q.clone(), q);
        }
        if a.is_inf() {
            let q = if b.is_inf() { Dec::nan() } else { Dec::inf(sign) };
            *st |= INVALID_OPERATION;
            return (q, Dec::nan());
        }
        let r = finalized(a.clone(), ctx, st);
        return (Dec::triple(sign, 0, 0), r);
    }
    if b.zero_coef() {
        if a.zero_coef() {
            *st |= DIVISION_UNDEFINED;
            return nan_pair();
        }
        *st |= DIVISION_BY_ZERO | INVALID_OPERATION;
        return (Dec::inf(sign), Dec::nan());
    }
    let (q, r) = divmod_raw(a, b, ctx, st);
    let q = finalized(q, ctx, st);
    let r = finalized(r, ctx, st);
    (q, r)
}

pub fn divint(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let sign = a.sign != b.sign;
    if a.is_special() || b.is_special() {
        if let Some(q) = check_nans(a, b, ctx, st) {
            return q;
        }
        if a.is_inf() && b.is_inf() {
            return seterror(st, INVALID_OPERATION);
        }
        if a.is_inf() {
            return Dec::inf(sign);
        }
        return Dec::triple(sign, 0, 0);
    }
    if b.zero_coef() {
        if a.zero_coef() {
            return seterror(st, DIVISION_UNDEFINED);
        }
        *st |= DIVISION_BY_ZERO;
        return Dec::inf(sign);
    }
    let (q, _) = divmod_raw(a, b, ctx, st);
    finalized(q, ctx, st)
}

/// the special cases of rem and rem_near
fn rem_special(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return Some(r);
        }
        if a.is_inf() {
            return Some(seterror(st, INVALID_OPERATION));
        }
        return Some(finalized(a.clone(), ctx, st));
    }
    if b.zero_coef() {
        return Some(seterror(st, if a.zero_coef() { DIVISION_UNDEFINED } else { INVALID_OPERATION }));
    }
    None
}

pub fn rem(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = rem_special(a, b, ctx, st) {
        return r;
    }
    let (_, r) = divmod_raw(a, b, ctx, st);
    finalized(r, ctx, st)
}

pub fn rem_near(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = rem_special(a, b, ctx, st) {
        return r;
    }
    let (q, mut r) = divmod_raw(a, b, ctx, st);
    if !q.is_nan() && !r.is_nan() && !r.zero_coef() {
        let expdiff = b.adjexp() - r.adjexp();
        if (-1..=1).contains(&expdiff) {
            let allnine = q.coef_is_all_nine();
            let qdigits = q.digits;
            let isodd = q.is_odd();
            let mx = Ctx::max();
            let mut ws = 0;
            let t = if a.sign == b.sign { addsub(&r, b, !b.sign, &mx, &mut ws) } else { addsub(&r, b, b.sign, &mx, &mut ws) };
            if ws & ERRORS != 0 {
                r = seterror(st, ws & ERRORS);
            } else {
                let c = cmp_abs(&t, &r);
                if c < 0 || (c == 0 && isodd) {
                    if allnine && qdigits == ctx.prec {
                        r = seterror(st, DIVISION_IMPOSSIBLE);
                    } else {
                        r = t;
                    }
                }
            }
        }
    }
    finalized(r, ctx, st)
}

// -------------------------------------------------------------------------
// exponents, integers, quantum

pub fn same_quantum(a: &Dec, b: &Dec) -> bool {
    if a.is_special() || b.is_special() {
        return (a.is_nan() && b.is_nan()) || (a.is_inf() && b.is_inf());
    }
    a.exp == b.exp
}

pub fn quantize(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
        if a.is_inf() && b.is_inf() {
            return a.clone();
        }
        return seterror(st, INVALID_OPERATION);
    }
    if b.exp > ctx.emax || b.exp < ctx.etiny() {
        return seterror(st, INVALID_OPERATION);
    }
    if a.zero_coef() {
        return finalized(Dec::triple(a.sign, 0, b.exp), ctx, st);
    }
    let expdiff = a.exp - b.exp;
    if a.digits as i128 + expdiff as i128 > ctx.prec as i128 {
        return seterror(st, INVALID_OPERATION);
    }
    let mut ws = 0;
    let mut r;
    if expdiff >= 0 {
        r = match shiftl(a, expdiff, st) {
            Some(r) => r,
            None => return Dec::nan(),
        };
        r.exp = b.exp;
    } else {
        let (s, rnd) = shiftr(a, -expdiff);
        r = s;
        r.exp = b.exp;
        if !apply_round_fit(&mut r, rnd, ctx, st) {
            return r;
        }
        ws |= ROUNDED;
        if rnd != 0 {
            ws |= INEXACT;
        }
    }
    if r.adjexp() > ctx.emax || r.adjexp() < ctx.etiny() {
        return seterror(st, INVALID_OPERATION);
    }
    *st |= ws;
    finalized(r, ctx, st)
}

pub fn reduce(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        return a.clone();
    }
    let r = finalized(a.clone(), ctx, st);
    if r.is_special() {
        return r;
    }
    if r.zero_coef() {
        return Dec::triple(a.sign, 0, 0);
    }
    let tz = trail_zeros(&r.coef);
    let maxexp = if ctx.clamp { ctx.etop() } else { ctx.emax };
    let shift = tz.min(maxexp - r.exp);
    let (mut s, _) = shiftr(&r, shift);
    s.exp += shift;
    s
}

/// a with exponent exp, rounded by the context's mode but otherwise
/// without its limits
fn rescale_raw(a: &Dec, exp: i64, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        return a.clone();
    }
    if a.zero_coef() {
        return Dec::triple(a.sign, 0, exp);
    }
    let expdiff = a.exp - exp;
    let mut r;
    if expdiff >= 0 {
        if a.digits as i128 + expdiff as i128 > MAX_PREC as i128 + 1 {
            return seterror(st, INVALID_OPERATION);
        }
        r = match shiftl(a, expdiff, st) {
            Some(r) => r,
            None => return Dec::nan(),
        };
        r.exp = exp;
    } else {
        let (s, rnd) = shiftr(a, -expdiff);
        r = s;
        r.exp = exp;
        apply_round_excess(&mut r, rnd, ctx);
        *st |= ROUNDED;
        if rnd != 0 {
            *st |= INEXACT;
        }
    }
    if r.is_subnormal(ctx) {
        *st |= SUBNORMAL;
    }
    r
}

pub fn rescale(a: &Dec, exp: i64, ctx: &Ctx, st: &mut u32) -> Dec {
    if exp > MAX_EMAX + 1 || exp < MIN_ETINY {
        return seterror(st, INVALID_OPERATION);
    }
    rescale_raw(a, exp, ctx, st)
}

fn rescale_fmt(a: &Dec, exp: i64, ctx: &Ctx, st: &mut u32) -> Dec {
    if exp > MAX_EMAX + 1 || exp < MIN_ETINY - MAX_PREC {
        return seterror(st, INVALID_OPERATION);
    }
    rescale_raw(a, exp, ctx, st)
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToInt {
    Exact,
    Silent,
    Trunc,
}

fn round_to_integral(action: ToInt, a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        return a.clone();
    }
    if a.exp >= 0 {
        return a.clone();
    }
    if a.zero_coef() {
        return Dec::triple(a.sign, 0, 0);
    }
    let (mut r, rnd) = shiftr(a, -a.exp);
    r.exp = 0;
    if action != ToInt::Trunc {
        apply_round_excess(&mut r, rnd, ctx);
        if action == ToInt::Exact {
            *st |= ROUNDED;
            if rnd != 0 {
                *st |= INEXACT;
            }
        }
    }
    r
}

pub fn round_to_intx(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    round_to_integral(ToInt::Exact, a, ctx, st)
}

pub fn round_to_int(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    round_to_integral(ToInt::Silent, a, ctx, st)
}

/// a rounded to an integer by `round`; specials are InvalidOperation
pub fn to_integral_by(a: &Dec, round: Round, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        return seterror(st, INVALID_OPERATION);
    }
    if round == Round::Down {
        return round_to_integral(ToInt::Trunc, a, ctx, st);
    }
    round_to_integral(ToInt::Silent, a, &ctx.with_round(round), st)
}

/// the absolute integer value when it fits in a u64 (libmpdec's
/// mpd_qabs_uint)
fn abs_uint(a: &Dec) -> Option<u64> {
    if a.is_special() {
        return None;
    }
    if a.zero_coef() {
        return Some(0);
    }
    if a.digits + a.exp > 20 || !a.is_int() {
        return None;
    }
    small(&a.int_value().abs()).and_then(|v| u64::try_from(v).ok())
}

/// the integer value as an i64 (mpd_qget_ssize)
pub fn get_ssize(a: &Dec) -> Option<i64> {
    let u = abs_uint(a)?;
    if u <= i64::MAX as u64 {
        Some(if a.sign { -(u as i64) } else { u as i64 })
    } else if a.sign && u == i64::MAX as u64 + 1 {
        Some(i64::MIN)
    } else {
        None
    }
}

pub fn logb(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        return Dec::inf(false);
    }
    if a.zero_coef() {
        *st |= DIVISION_BY_ZERO;
        return Dec::inf(true);
    }
    finalized(Dec::from_i64(a.adjexp()), ctx, st)
}

pub fn scaleb(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
    }
    if b.exp != 0 || b.is_inf() {
        return seterror(st, INVALID_OPERATION);
    }
    let maxjump = 2 * (ctx.emax as i128 + ctx.prec as i128);
    let n = match abs_uint(b) {
        Some(n) if (n as i128) <= maxjump => n as i128,
        _ => return seterror(st, INVALID_OPERATION),
    };
    if a.is_inf() {
        return a.clone();
    }
    let exp = (a.exp as i128 + if b.sign { -n } else { n }).clamp(EXP_CLAMP as i128, EXP_INF as i128);
    finalized(Dec { exp: exp as i64, ..a.clone() }, ctx, st)
}

/// the shift count b of rotate and shift: an integer within prec
fn shift_count(b: &Dec, ctx: &Ctx) -> Option<i64> {
    if b.exp != 0 || b.is_inf() {
        return None;
    }
    let n = get_ssize(b)?;
    if n > ctx.prec || n < -ctx.prec {
        return None;
    }
    Some(n)
}

pub fn rotate(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
    }
    let Some(n) = shift_count(b, ctx) else {
        return seterror(st, INVALID_OPERATION);
    };
    if a.is_inf() {
        return a.clone();
    }
    let (lshift, rshift) = if n >= 0 { (n, ctx.prec - n) } else { (ctx.prec + n, -n) };
    let mut a = a.clone();
    cap(&mut a, ctx);
    let Some(mut bg) = shiftl(&a, lshift, st) else {
        return seterror(st, MALLOC_ERROR);
    };
    cap(&mut bg, ctx);
    let (sm, _) = shiftr(&a, rshift);
    addsub(&bg, &sm, sm.sign, ctx, st)
}

pub fn shift(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() || b.is_special() {
        if let Some(r) = check_nans(a, b, ctx, st) {
            return r;
        }
    }
    let Some(n) = shift_count(b, ctx) else {
        return seterror(st, INVALID_OPERATION);
    };
    if a.is_inf() {
        return a.clone();
    }
    if n >= 0 {
        let Some(mut r) = shiftl(a, n, st) else {
            return Dec::nan();
        };
        cap(&mut r, ctx);
        r
    } else {
        let mut r = a.clone();
        cap(&mut r, ctx);
        shiftr(&r, -n).0
    }
}

pub fn next_minus(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    next(a, ctx, st, true)
}

pub fn next_plus(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    next(a, ctx, st, false)
}

fn next(a: &Dec, ctx: &Ctx, st: &mut u32, minus: bool) -> Dec {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        if a.sign == minus {
            return a.clone();
        }
        let mut r = Dec::triple(!minus, 0, 0);
        maxcoeff(&mut r, ctx, st);
        if r.is_nan() {
            return r;
        }
        r.exp = ctx.etop();
        return r;
    }
    let w = ctx.with_round(if minus { Round::Floor } else { Round::Ceiling });
    let mut ws = 0;
    let r = finalized(a.clone(), &w, &mut ws);
    if ws & (INEXACT | ERRORS) != 0 {
        *st |= ws & ERRORS;
        return r;
    }
    let tiny = Dec::triple(false, 1, ctx.etiny() - 1);
    let mut ws = 0;
    let r = if minus { sub(a, &tiny, &w, &mut ws) } else { add(a, &tiny, &w, &mut ws) };
    *st |= ws & ERRORS;
    r
}

pub fn next_toward(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = check_nans(a, b, ctx, st) {
        return r;
    }
    let c = cmp(a, b);
    if c == 0 {
        return copy_sign(a, b);
    }
    let r = if c < 0 { next_plus(a, ctx, st) } else { next_minus(a, ctx, st) };
    if r.is_inf() {
        *st |= OVERFLOW | ROUNDED | INEXACT;
    } else if r.is_finite() && r.adjexp() < ctx.emin {
        *st |= UNDERFLOW | SUBNORMAL | ROUNDED | INEXACT;
        if r.is_zero() {
            *st |= CLAMPED;
        }
    }
    r
}

// -------------------------------------------------------------------------
// logical operations: operands of digits 0 and 1 only

/// the digits of a logical operand, least significant first; None when it
/// is not one
fn logical_digits(a: &Dec) -> Option<Vec<u8>> {
    if a.is_special() || a.sign || a.exp != 0 {
        return None;
    }
    let s = digits_str(&a.coef);
    let v: Vec<u8> = s.bytes().rev().map(|b| b - b'0').collect();
    if v.iter().any(|&d| d > 1) {
        return None;
    }
    Some(v)
}

fn from_logical(v: &[u8], ctx: &Ctx) -> Dec {
    let n = v.len().min(ctx.prec.max(0) as usize);
    let s: String = v[..n].iter().rev().map(|&d| (b'0' + d) as char).collect();
    let s = s.trim_start_matches('0');
    let c = if s.is_empty() { BigInt::zero() } else { BigInt::from_radix(s, 10).unwrap() };
    Dec::fin(false, c, 0)
}

pub fn logical(a: &Dec, b: &Dec, op: char, ctx: &Ctx, st: &mut u32) -> Dec {
    let (Some(x), Some(y)) = (logical_digits(a), logical_digits(b)) else {
        return seterror(st, INVALID_OPERATION);
    };
    let n = x.len().max(y.len());
    let v: Vec<u8> = (0..n)
        .map(|i| {
            let (p, q) = (*x.get(i).unwrap_or(&0), *y.get(i).unwrap_or(&0));
            match op {
                '&' => p & q,
                '|' => p | q,
                _ => p ^ q,
            }
        })
        .collect();
    from_logical(&v, ctx)
}

pub fn invert(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let Some(x) = logical_digits(a) else {
        return seterror(st, INVALID_OPERATION);
    };
    let n = x.len().max(ctx.prec.max(0) as usize);
    let v: Vec<u8> = (0..n).map(|i| 1 - *x.get(i).unwrap_or(&0)).collect();
    from_logical(&v, ctx)
}

pub fn class(a: &Dec, ctx: &Ctx) -> &'static str {
    match a.kind {
        Kind::NaN => "NaN",
        Kind::SNaN => "sNaN",
        Kind::Inf => {
            if a.sign {
                "-Infinity"
            } else {
                "+Infinity"
            }
        }
        Kind::Fin => match (a.sign, a.zero_coef(), a.is_normal(ctx)) {
            (false, true, _) => "+Zero",
            (true, true, _) => "-Zero",
            (false, _, true) => "+Normal",
            (true, _, true) => "-Normal",
            (false, _, false) => "+Subnormal",
            (true, _, false) => "-Subnormal",
        },
    }
}

// -------------------------------------------------------------------------
// square root

fn sqrt_raw(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut ideal_exp = (a.exp - (a.exp & 1)) / 2;
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        if a.sign {
            return seterror(st, INVALID_OPERATION);
        }
        return Dec::inf(false);
    }
    if a.zero_coef() {
        return finalized(Dec::triple(a.sign, 0, ideal_exp), ctx, st);
    }
    if a.sign {
        return seterror(st, INVALID_OPERATION);
    }
    let prec = ctx.prec + 1;
    let mut c = a.coef.clone();
    let mut cdigits = a.digits;
    let l = if a.exp & 1 != 0 {
        c = shl(&c, 1);
        cdigits += 1;
        (a.digits >> 1) + 1
    } else {
        (a.digits + 1) >> 1
    };
    let shift = prec - l;
    let mut exact;
    if shift >= 0 {
        if too_big(cdigits + 2 * shift) {
            return finalized(seterror(st, MALLOC_ERROR), &ctx.with_round(Round::HalfEven), st);
        }
        c = shl(&c, 2 * shift);
        exact = true;
    } else {
        let (q, rnd) = shr(&c, cdigits, -2 * shift);
        c = q;
        exact = rnd == 0;
    }
    ideal_exp -= shift;
    // floor(sqrt(c)) by Newton's method, starting above it
    let mut x = pow10(prec);
    loop {
        let q = c.divmod(&x).unwrap().0;
        if x.cmp(&q) != Ordering::Greater {
            break;
        }
        x = x.add(&q).shr(1);
    }
    if exact {
        exact = x.mul(&x).cmp(&c) == Ordering::Equal;
    }
    let mut r = Dec::fin(false, x, 0);
    if exact {
        if shift >= 0 {
            r = shiftr(&r, shift).0;
        } else {
            r = match shiftl(&r, -shift, st) {
                Some(r) => r,
                None => Dec::nan(),
            };
        }
        ideal_exp += shift;
    } else {
        let ld = r.lsd();
        if ld == 0 || ld == 5 {
            incr(&mut r);
        }
    }
    if r.is_finite() {
        r.exp = ideal_exp;
    }
    finalized(r, &ctx.with_round(Round::HalfEven), st)
}

pub fn sqrt(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut xs = 0;
    let r = sqrt_raw(a, ctx, &mut xs);
    if xs & (MALLOC_ERROR | DIVISION_IMPOSSIBLE) != 0 {
        let wprec = a.digits;
        if wprec >= ctx.prec {
            *st |= xs | ERRORS;
            return r;
        }
        let w = Ctx { prec: wprec, ..ctx.clone() };
        let mut ys = 0;
        let r = sqrt_raw(a, &w, &mut ys);
        if ys != 0 {
            let ys = *st | ((xs | ys) & ERRORS);
            return seterror(st, ys);
        }
        return r;
    }
    *st |= xs;
    r
}

// -------------------------------------------------------------------------
// integer kernels for exp, ln, log10 and power, after the pure-Python
// decimal's _dexp, _dlog, _dlog10 and _dpower: each returns an integer
// approximation with an error below one unit

fn bi(v: i64) -> BigInt {
    BigInt::from_i64(v)
}

/// len(str(c)) of a Python int: its digits and any minus sign
fn strlen(c: &BigInt) -> i64 {
    ndigits(&c.abs()) + c.neg as i64
}

fn strlen_i128(v: i128) -> i64 {
    v.to_string().len() as i64
}

/// the integer nearest a / b (b > 0), ties to even
fn div_nearest(a: &BigInt, b: &BigInt) -> BigInt {
    let (q, r) = a.divmod(b).unwrap();
    let t = r.shl(1).add(&bi(q.is_odd() as i64));
    if t.cmp(b) == Ordering::Greater {
        q.add(&one())
    } else {
        q
    }
}

/// the integer nearest x / 2**shift, ties to even
fn rshift_nearest(x: &BigInt, shift: usize) -> BigInt {
    let q = x.shr(shift);
    let r = x.sub(&q.shl(shift));
    let t = r.shl(1).add(&bi(q.is_odd() as i64));
    if t.cmp(&one().shl(shift)) == Ordering::Greater {
        q.add(&one())
    } else {
        q
    }
}

/// the integer nearest sqrt(n), from the approximation a
fn sqrt_nearest(n: &BigInt, a: &BigInt) -> BigInt {
    let mut a = a.clone();
    let mut b = BigInt::zero();
    while a.cmp(&b) != Ordering::Equal {
        b = a.clone();
        let q = n.neg().divmod(&a).unwrap().0;
        a = a.sub(&q).shr(1);
    }
    a
}

/// M * log(x / M), for 0.1 <= x/M <= 10
fn ilog(x: &BigInt, m: &BigInt) -> BigInt {
    const L: i64 = 8;
    let mut y = x.sub(m);
    let mut r: i64 = 0;
    loop {
        let ay = y.abs();
        let more = if r <= L { ay.shl((L - r) as usize).cmp(m) != Ordering::Less } else { ay.shr((r - L) as usize).cmp(m) != Ordering::Less };
        if !more {
            break;
        }
        let t = m.mul(&m.add(&rshift_nearest(&y, r as usize)));
        y = div_nearest(&m.mul(&y).shl(1), &m.add(&sqrt_nearest(&t, m)));
        r += 1;
    }
    let terms = (10 * strlen(m) + 3 * L - 1) / (3 * L);
    let yshift = rshift_nearest(&y, r as usize);
    let mut w = div_nearest(m, &bi(terms));
    for k in (1..terms).rev() {
        w = div_nearest(m, &bi(k)).sub(&div_nearest(&yshift.mul(&w), m));
    }
    div_nearest(&w.mul(&y), m)
}

crate::gil_static! {
    /// digits of log(10), truncated, as far as they have been needed
    static LOG10: RefCell<String> = RefCell::new("23025850929940456840179914546843642076011014886".into());
}

/// floor(10**p * log(10))
fn log10_digits(p: i64) -> BigInt {
    let have = LOG10.with(|d| d.borrow().len() as i64);
    if p >= have {
        let mut extra = 3;
        let digits = loop {
            let m = pow10(p + extra + 2);
            let d = digits_str(&div_nearest(&ilog(&m.mul(&bi(10)), &m), &bi(100)));
            if !d.ends_with(&"0".repeat(extra as usize)) {
                break d;
            }
            extra += 3;
        };
        let t = digits.trim_end_matches('0');
        let t = t[..t.len() - 1].to_string();
        LOG10.with(|d| *d.borrow_mut() = t);
    }
    LOG10.with(|d| BigInt::from_radix(&d.borrow()[..(p + 1) as usize], 10).unwrap())
}

/// M * exp(x / M), for small x / M
fn iexp(x: &BigInt, m: &BigInt) -> BigInt {
    const L: usize = 8;
    let r = x.shl(L).divmod(m).unwrap().0.bit_length();
    let terms = (10 * strlen(m) + 3 * L as i64 - 1) / (3 * L as i64);
    let mut y = div_nearest(x, &bi(terms));
    let mshift = m.shl(r);
    for i in (1..terms).rev() {
        y = div_nearest(&x.mul(&mshift.add(&y)), &mshift.mul(&bi(i)));
    }
    for k in (0..r).rev() {
        let ms = m.shl(k + 2);
        y = div_nearest(&y.mul(&y.add(&ms)), &ms);
    }
    m.add(&y)
}

fn sat_i64(v: &BigInt) -> i64 {
    v.to_i64().unwrap_or(if v.neg { i64::MIN / 2 } else { i64::MAX / 2 })
}

/// (d, f) with (d-1)*10**f < exp(c*10**e) < (d+1)*10**f and p or p+1
/// digits in d
fn dexp(c: &BigInt, e: i64, p: i64) -> (BigInt, i64) {
    let p = p + 2;
    let extra = (e + strlen(c) - 1).max(0);
    let q = p + extra;
    let shift = e + q;
    let cshift = if shift >= 0 { c.mul(&pow10(shift)) } else { c.divmod(&pow10(-shift)).unwrap().0 };
    let (quot, rem) = cshift.divmod(&log10_digits(q)).unwrap();
    let rem = div_nearest(&rem, &pow10(extra));
    (div_nearest(&iexp(&rem, &pow10(p)), &bi(1000)), sat_i64(&quot) - p + 3)
}

/// 10**p * log(c*10**e), c*10**e != 1
fn dlog(c: &BigInt, e: i64, p: i64) -> BigInt {
    let p = p + 2;
    let l = strlen(c);
    let f = e + l - (e + l >= 1) as i64;
    let log_d = if p > 0 {
        let k = e + p - f;
        let c = if k >= 0 { c.mul(&pow10(k)) } else { div_nearest(c, &pow10(-k)) };
        ilog(&c, &pow10(p))
    } else {
        BigInt::zero()
    };
    let mut f_log_ten = BigInt::zero();
    if f != 0 {
        let extra = strlen_i128((f as i128).abs()) - 1;
        if p + extra >= 0 {
            f_log_ten = div_nearest(&bi(f).mul(&log10_digits(p + extra)), &pow10(extra));
        }
    }
    div_nearest(&f_log_ten.add(&log_d), &bi(100))
}

/// 10**p * log10(c*10**e), c*10**e != 1
fn dlog10(c: &BigInt, e: i64, p: i64) -> BigInt {
    let p = p + 2;
    let l = strlen(c);
    let f = e + l - (e + l >= 1) as i64;
    let (log_d, log_tenpower) = if p > 0 {
        let m = pow10(p);
        let k = e + p - f;
        let c = if k >= 0 { c.mul(&pow10(k)) } else { div_nearest(c, &pow10(-k)) };
        let log_d = ilog(&c, &m);
        let log_d = div_nearest(&log_d.mul(&m), &log10_digits(p));
        (log_d, bi(f).mul(&m))
    } else {
        (BigInt::zero(), div_nearest(&bi(f), &pow10(-p)))
    };
    div_nearest(&log_tenpower.add(&log_d), &bi(100))
}

/// (c, e) with (c-1)*10**e < x**y < (c+1)*10**e, x = xc*10**xe positive and
/// not 1, y = yc*10**ye nonzero
fn dpower(xc: &BigInt, xe: i64, yc: &BigInt, ye: i64, p: i64) -> (BigInt, i64) {
    let b = strlen(&yc.abs()) + ye;
    let lxc = dlog(xc, xe, p + b + 1);
    let shift = ye - b;
    let pc = if shift >= 0 { lxc.mul(yc).mul(&pow10(shift)) } else { div_nearest(&lxc.mul(yc), &pow10(-shift)) };
    if pc.is_zero() {
        let ypos = !yc.neg && !yc.is_zero();
        if (strlen(xc) + xe >= 1) == ypos {
            (pow10(p - 1).add(&one()), 1 - p)
        } else {
            (pow10(p).sub(&one()), -p)
        }
    } else {
        let (coeff, exp) = dexp(&pc, -(p + 1), p + 1);
        (div_nearest(&coeff, &bi(10)), exp + 1)
    }
}

/// r with ln(a) >= 10**r, a finite, positive and not 1
fn ln_exp_bound(a: &Dec) -> i64 {
    let adj = a.adjexp();
    if adj >= 1 {
        return strlen_i128(adj as i128 * 23 / 10) - 1;
    }
    if adj <= -2 {
        return strlen_i128((-1 - adj as i128) * 23 / 10) - 1;
    }
    let (c, e) = (&a.coef, a.exp);
    if adj == 0 {
        let num = digits_str(&c.sub(&pow10(-e)));
        let den = digits_str(c);
        return num.len() as i64 - den.len() as i64 - (num < den) as i64;
    }
    e + digits_str(&pow10(-e).sub(c)).len() as i64 - 1
}

/// r with log10(a) >= 10**r
fn log10_exp_bound(a: &Dec) -> i64 {
    let adj = a.adjexp();
    if adj >= 1 {
        return strlen_i128(adj as i128) - 1;
    }
    if adj <= -2 {
        return strlen_i128(-1 - adj as i128) - 1;
    }
    let (c, e) = (&a.coef, a.exp);
    if adj == 0 {
        let num = digits_str(&c.sub(&pow10(-e)));
        let den = digits_str(&c.mul(&bi(231)));
        return num.len() as i64 - den.len() as i64 - (num < den) as i64 + 2;
    }
    let num = digits_str(&pow10(-e).sub(c));
    num.len() as i64 + e - (num.as_str() < "231") as i64 - 1
}

fn signed_coef(a: &Dec) -> BigInt {
    if a.sign {
        a.coef.neg()
    } else {
        a.coef.clone()
    }
}

fn zeropad(r: &mut Dec, ctx: &Ctx, st: &mut u32) {
    if r.is_finite() && !r.zero_coef() && r.digits < ctx.prec {
        let shift = ctx.prec - r.digits;
        if let Some(s) = shiftl(r, shift, st) {
            *r = s;
            r.exp -= shift;
        }
    }
}

/// libmpdec's loop for a correctly rounded result: approximations r with
/// error below 10**ulp_exp at growing precisions, until r - ulp and r + ulp
/// round alike
fn correctly_rounded(ctx: &Ctx, st: &mut u32, pad: bool, mut approx: impl FnMut(i64) -> (Dec, i64)) -> Dec {
    let w = Ctx { round: Round::HalfEven, clamp: false, ..ctx.clone() };
    let mut p = ctx.prec + 3;
    loop {
        let (mut r, ulp_exp) = approx(p);
        let ulp = Dec::triple(false, 1, ulp_exp);
        let mut ws = 0;
        let t1 = add(&r, &ulp, &w, &mut ws);
        let t2 = sub(&r, &ulp, &w, &mut ws);
        if r.is_special() || r.zero_coef() || qcmp(&t1, &t2, st) == Some(0) {
            let fw = Ctx { clamp: ctx.clamp, ..w };
            if pad {
                zeropad(&mut r, &fw, st);
            }
            check_underflow(&r, &fw, st);
            finalize(&mut r, &fw, st);
            *st |= INEXACT | ROUNDED;
            return r;
        }
        p += 19;
    }
}

pub fn exp(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return r;
        }
        return if a.sign { Dec::triple(false, 0, 0) } else { Dec::inf(false) };
    }
    if a.zero_coef() {
        return Dec::triple(false, 1, 0);
    }
    // e**x for |x| >= 10**19 is past any exponent
    let t = (a.digits + a.exp).max(0);
    if t > 19 {
        if !a.sign {
            *st |= OVERFLOW | INEXACT | ROUNDED;
            return Dec::inf(false);
        }
        *st |= INEXACT | ROUNDED | SUBNORMAL | UNDERFLOW | CLAMPED;
        return Dec::triple(false, 0, ctx.etiny());
    }
    let c = signed_coef(a);
    correctly_rounded(ctx, st, true, |p| {
        // at most 9 * 10**(-p-1) from zero: one
        if cmp_abs(a, &Dec::triple(false, 9, -(p + 1))) <= 0 {
            return (Dec::triple(false, 1, 0), 1 - p);
        }
        let (d, f) = dexp(&c, a.exp, p);
        (Dec::fin(false, d, f), f)
    })
}

/// the checks ln and log10 share: NaNs, infinities, zero, negatives
fn log_special(a: &Dec, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if a.is_special() {
        if let Some(r) = check_nan(a, ctx, st) {
            return Some(r);
        }
        if a.sign {
            return Some(seterror(st, INVALID_OPERATION));
        }
        return Some(Dec::inf(false));
    }
    if a.zero_coef() {
        return Some(Dec::inf(true));
    }
    if a.sign {
        return Some(seterror(st, INVALID_OPERATION));
    }
    None
}

fn exp_digits(v: i64) -> i64 {
    strlen_i128((v as i128).abs())
}

/// ln of a number so close to one that the result underflows at precision
/// p: libmpdec's shortcut, ±10**(etiny-1)
fn ln_underflow(a: &Dec, p: i64, ctx: &Ctx) -> Option<Dec> {
    let s = digits_str(&a.coef);
    let mut x: i64 = s[..s.len().min(3)].parse().unwrap();
    if x < 10 {
        x *= 10;
    }
    if x < 100 {
        x *= 10;
    }
    x -= 100;
    let (vexp, t) = if x <= 400 { (-(a.digits - 1), a.exp + a.digits - 1) } else { (-a.digits, a.exp + a.digits) };
    if t == 0 && (x <= 15 || x >= 800) {
        let v = Dec { exp: vexp, ..a.clone() };
        let one = Dec::triple(false, 1, 0);
        let c = cmp(&v, &one);
        let mut ws = 0;
        let mut d = addsub(&v, &one, true, &Ctx::max(), &mut ws);
        if c < 0 {
            d.exp += 1;
        }
        let etiny = ctx.emin - p + 1;
        if d.adjexp() < etiny {
            return Some(Dec::triple(c < 0, 1, etiny - 1));
        }
    }
    None
}

pub fn ln(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = log_special(a, ctx, st) {
        return r;
    }
    if cmp(a, &Dec::triple(false, 1, 0)) == 0 {
        return Dec::triple(false, 0, 0);
    }
    let adjexp = a.adjexp();
    let t = 2 * if adjexp < 0 { -adjexp - 1 } else { adjexp };
    if exp_digits(t) - 1 > ctx.emax {
        *st |= OVERFLOW | INEXACT | ROUNDED;
        return Dec::inf(adjexp < 0);
    }
    correctly_rounded(ctx, st, false, |p| {
        if let Some(r) = ln_underflow(a, p, ctx) {
            let u = r.exp + r.digits - p;
            return (r, u);
        }
        let places = p - ln_exp_bound(a) + 2;
        let c = dlog(&a.coef, a.exp, places);
        (Dec::fin(c.neg, c.abs(), -places), -places)
    })
}

pub fn log10(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if let Some(r) = log_special(a, ctx, st) {
        return r;
    }
    let w = ctx.with_round(Round::HalfEven);
    if a.coef_is_pow10() {
        let adj = a.adjexp();
        return finalized(Dec::fin(adj < 0, big(adj.unsigned_abs() as u128), 0), &w, st);
    }
    let adjexp = a.adjexp();
    let t = if adjexp < 0 { -adjexp - 1 } else { adjexp };
    if exp_digits(t) - 1 > ctx.emax {
        *st |= OVERFLOW | INEXACT | ROUNDED;
        return Dec::inf(adjexp < 0);
    }
    correctly_rounded(ctx, st, false, |p| {
        let places = p - log10_exp_bound(a) + 2;
        let c = dlog10(&a.coef, a.exp, places);
        (Dec::fin(c.neg, c.abs(), -places), -places)
    })
}

// -------------------------------------------------------------------------
// powers

/// one with prec digits: 1.000...
fn one_prec(sign: bool, shift: i64, st: &mut u32) -> Dec {
    if too_big(shift + 1) {
        return seterror(st, MALLOC_ERROR);
    }
    Dec::fin(sign, pow10(shift), -shift)
}

/// x**y for |x| == 1; None otherwise
fn pow_one(base: &Dec, exp: &Dec, resultsign: bool, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    if cmp_abs(base, &Dec::triple(false, 1, 0)) != 0 {
        return None;
    }
    let shift;
    if exp.is_int() {
        if exp.sign {
            return Some(Dec::triple(resultsign, 1, 0));
        }
        // 1.000**3 = 1.000000000
        let n = exp.int_value().mul(&bi(-base.exp));
        let lim = bi(ctx.prec - 1);
        if n.cmp(&lim) == Ordering::Greater {
            shift = ctx.prec - 1;
            *st |= ROUNDED;
        } else {
            shift = n.to_i64().unwrap();
        }
    } else {
        shift = ctx.prec - 1;
        *st |= INEXACT | ROUNDED;
    }
    Some(one_prec(resultsign, shift, st))
}

/// a lower bound for log10(|log10(x)|), as libmpdec's _lower_bound_zeta
fn lower_bound_zeta(x: &Dec) -> i64 {
    let t = x.adjexp();
    if t > 0 {
        return exp_digits(t) - 1;
    }
    if t < -1 {
        return exp_digits(t + 1) - 1;
    }
    let mut ws = 0;
    let d = addsub(x, &Dec::triple(false, 1, 0), true, &Ctx::max(), &mut ws);
    let u = d.adjexp();
    if t == 0 {
        u - 2
    } else {
        u - 1
    }
}

/// certain overflow or underflow of x**y
fn pow_bounds(x: &Dec, y: &Dec, resultsign: bool, ctx: &Ctx, st: &mut u32) -> Option<Dec> {
    let ax = x.with_sign(false);
    let lb_theta = y.adjexp() as i128;
    let lb_zeta = lower_bound_zeta(&ax) as i128;
    if (ax.adjexp() < 0) == y.sign {
        if (exp_digits(ctx.emax) as i128) < lb_zeta + lb_theta {
            return Some(finalized(Dec::triple(resultsign, 1, EXP_INF), ctx, st));
        }
    } else if (exp_digits(ctx.etiny()) as i128) < lb_zeta + lb_theta {
        return Some(finalized(Dec::triple(resultsign, 1, ctx.etiny() - 1), ctx, st));
    }
    None
}

fn pow_uint(base: &Dec, n: u64, resultsign: bool, ctx: &Ctx, st: &mut u32) -> Dec {
    if n == 0 {
        return Dec::triple(resultsign, 1, 0);
    }
    let mut ws = 0;
    let mut r = base.clone();
    let mut bit = 1u64 << (63 - n.leading_zeros());
    loop {
        bit >>= 1;
        if bit == 0 {
            break;
        }
        r = mul(&r, &r, ctx, &mut ws);
        if n & bit != 0 {
            r = mul(&r, base, ctx, &mut ws);
        }
        if r.is_special() || (r.zero_coef() && ws & CLAMPED != 0) {
            break;
        }
    }
    *st |= ws;
    r.sign = resultsign;
    r
}

/// an integer power with an exponent past u64
fn pow_bigexp(mut tbase: Dec, texp: &Dec, resultsign: bool, ctx: &Ctx, st: &mut u32) -> Dec {
    if too_big(texp.digits + texp.exp) {
        return seterror(st, MALLOC_ERROR);
    }
    let mut e = texp.int_value().abs();
    let mut ws = 0;
    let mut r = Dec::triple(false, 1, 0);
    while !e.is_zero() {
        if e.is_odd() {
            r = mul(&r, &tbase, ctx, &mut ws);
            *st |= ws;
            if r.is_special() || (r.zero_coef() && ws & CLAMPED != 0) {
                break;
            }
        }
        tbase = mul(&tbase, &tbase, ctx, &mut ws);
        e = e.shr(1);
        if tbase.is_nan() {
            return seterror(st, ws & ERRORS);
        }
    }
    r.sign = resultsign;
    r
}

fn pow_int(base: &Dec, exp: &Dec, resultsign: bool, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut w = Ctx { prec: ctx.prec + exp.digits + exp.exp + 2, round: Round::HalfEven, clamp: false, ..ctx.clone() };
    let tbase = if exp.sign {
        w.prec += 1;
        let mut ws = 0;
        let t = div(&Dec::triple(false, 1, 0), base, &w, &mut ws);
        *st |= ws;
        if ws & ERRORS != 0 {
            return finalized(Dec::nan(), ctx, st);
        }
        t
    } else {
        base.clone()
    };
    let mut r = match abs_uint(exp) {
        Some(n) => pow_uint(&tbase, n, resultsign, &w, st),
        None => pow_bigexp(tbase, exp, resultsign, &w, st),
    };
    if r.is_inf() {
        // for the directed roundings
        r = Dec::triple(resultsign, 1, EXP_INF);
    }
    finalized(r, ctx, st)
}

/// x**y for a non-integral y: libmpdec computes exp(y * ln(x)) at a
/// precision of max(digits, prec) + 23; here the kernels' approximation
/// goes as far as the rounding needs, at most to that precision
fn pow_real(base: &Dec, exp: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let wp = base.digits.max(ctx.prec) + 4 + 19;
    // libmpdec's exp at the working precision is exactly 1 for
    // |y * ln(x)| <= 9E-(wp+1), whatever the rounding
    let w = Ctx { prec: 20, round: Round::HalfEven, ..Ctx::max() };
    let mut ws = 0;
    let mut t = mul(&ln(base, &w, &mut ws), exp, &w, &mut ws);
    t.sign = false;
    if t.is_finite() && cmp(&t, &Dec::triple(false, 9, -(wp + 1))) <= 0 {
        *st |= INEXACT | ROUNDED;
        return finalized(one_prec(false, ctx.prec - 1, st), ctx, st);
    }
    let yc = signed_coef(exp);
    let mut p = ctx.prec + 3;
    let (c, e) = loop {
        if p >= wp {
            break dpower(&base.coef, base.exp, &yc, exp.exp, wp);
        }
        let (c, e) = dpower(&base.coef, base.exp, &yc, exp.exp, p);
        let mut ws = 0;
        let t1 = finalized(Dec::fin(false, c.add(&one()), e), ctx, &mut ws);
        let t2 = finalized(Dec::fin(false, c.sub(&one()), e), ctx, &mut ws);
        if !t1.is_nan() && !t2.is_nan() && cmp(&t1, &t2) == 0 {
            break (c, e);
        }
        p += 19;
    };
    *st |= INEXACT | ROUNDED;
    // a sticky digit keeps the result inexact when it rounds to a subnormal
    let c = c.mul(&BigInt::from_i64(10)).add(&one());
    let mut r = Dec::fin(false, c, e - 1);
    let mut ws = 0;
    let rw = finalized(r.clone(), &Ctx { prec: wp, ..Ctx::max() }, &mut ws);
    if rw.is_finite() && cmp(&rw, &Dec::triple(false, 1, 0)) == 0 {
        r = one_prec(false, ctx.prec - 1, st);
    }
    finalized(r, ctx, st)
}

pub fn pow(base: &Dec, exp: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if base.is_special() || exp.is_special() {
        if let Some(r) = check_nans(base, exp, ctx, st) {
            return r;
        }
    }
    let intexp = exp.is_integer();
    let resultsign = intexp && base.sign && exp.is_odd();
    if base.is_zero() {
        if exp.is_zero() {
            return seterror(st, INVALID_OPERATION);
        }
        if exp.sign {
            return Dec::inf(resultsign);
        }
        return Dec::triple(resultsign, 0, 0);
    }
    if base.sign && (!intexp || exp.is_inf()) {
        return seterror(st, INVALID_OPERATION);
    }
    if exp.is_inf() {
        let c = cmp(base, &Dec::triple(false, 1, 0));
        if c == 0 {
            *st |= INEXACT | ROUNDED;
            return one_prec(resultsign, ctx.prec - 1, st);
        }
        if c * arith_sign(exp) < 0 {
            return Dec::triple(resultsign, 0, 0);
        }
        return Dec::inf(resultsign);
    }
    if base.is_inf() {
        if exp.is_zero() {
            return Dec::triple(resultsign, 1, 0);
        }
        if exp.sign {
            return Dec::triple(resultsign, 0, 0);
        }
        return Dec::inf(resultsign);
    }
    if exp.is_zero() {
        return Dec::triple(resultsign, 1, 0);
    }
    if let Some(r) = pow_one(base, exp, resultsign, ctx, st) {
        return r;
    }
    if let Some(r) = pow_bounds(base, exp, resultsign, ctx, st) {
        return r;
    }
    if intexp {
        pow_int(base, exp, resultsign, ctx, st)
    } else {
        pow_real(base, exp, ctx, st)
    }
}

fn modpow(b: &BigInt, e: &BigInt, m: &BigInt) -> BigInt {
    let mut result = one().divmod(m).unwrap().1;
    let mut base = b.divmod(m).unwrap().1;
    let mut e = e.clone();
    while !e.is_zero() {
        if e.is_odd() {
            result = result.mul(&base).divmod(m).unwrap().1;
        }
        e = e.shr(1);
        if !e.is_zero() {
            base = base.mul(&base).divmod(m).unwrap().1;
        }
    }
    result
}

/// an integral decimal as coefficient and non-negative exponent
fn int_parts(a: &Dec) -> (BigInt, i64) {
    if a.exp < 0 {
        (shr(&a.coef, a.digits, -a.exp).0, 0)
    } else {
        (a.coef.clone(), a.exp)
    }
}

/// (base ** exp) % m, all integers
pub fn powmod(base: &Dec, exp: &Dec, m: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    if base.is_special() || exp.is_special() || m.is_special() {
        if let Some(r) = check_3nans(base, exp, m, ctx, st) {
            return r;
        }
        return seterror(st, INVALID_OPERATION);
    }
    if !base.is_int() || !exp.is_int() || !m.is_int() || m.zero_coef() || m.digits + m.exp > ctx.prec {
        return seterror(st, INVALID_OPERATION);
    }
    let sign = base.sign && exp.is_odd();
    if exp.zero_coef() {
        if base.zero_coef() {
            return seterror(st, INVALID_OPERATION);
        }
        let r = if cmp_abs(m, &Dec::triple(false, 1, 0)) == 0 { 0 } else { 1 };
        return Dec::triple(sign, r, 0);
    }
    if exp.sign {
        return seterror(st, INVALID_OPERATION);
    }
    if base.zero_coef() {
        return Dec::triple(sign, 0, 0);
    }
    if too_big(m.digits + m.exp) {
        return seterror(st, MALLOC_ERROR);
    }
    let md = m.int_value().abs();
    let ten = bi(10);
    let (bc, bexp) = int_parts(base);
    let mut tb = bc.mul(&modpow(&ten, &bi(bexp), &md)).divmod(&md).unwrap().1;
    let (ec, eexp) = int_parts(exp);
    for _ in 0..eexp {
        tb = modpow(&tb, &ten, &md);
    }
    Dec::fin(sign, modpow(&tb, &ec, &md), 0)
}

// -------------------------------------------------------------------------
// strings

/// a numeric string in ASCII, as libmpdec's mpd_qset_string reads it
pub fn parse(s: &str, ctx: &Ctx, st: &mut u32) -> Dec {
    let b = s.as_bytes();
    let mut i = 0;
    let mut sign = false;
    if b.first() == Some(&b'+') {
        i = 1;
    } else if b.first() == Some(&b'-') {
        sign = true;
        i = 1;
    }
    let rest = &b[i..];
    let starts = |w: &str| rest.len() >= w.len() && rest[..w.len()].eq_ignore_ascii_case(w.as_bytes());
    let syntax = |st: &mut u32| seterror(st, CONVERSION_SYNTAX);
    if starts("nan") || starts("snan") {
        let kind = if starts("nan") { Kind::NaN } else { Kind::SNaN };
        let tail = &rest[if kind == Kind::NaN { 3 } else { 4 }..];
        let mut d = Dec::special(sign, kind);
        if tail.is_empty() {
            return d;
        }
        if !tail.iter().all(|c| c.is_ascii_digit()) {
            return syntax(st);
        }
        let payload = std::str::from_utf8(tail).unwrap().trim_start_matches('0');
        if payload.is_empty() {
            return d;
        }
        if payload.len() as i64 > ctx.prec - ctx.clamp as i64 {
            return syntax(st);
        }
        d.coef = BigInt::from_radix(payload, 10).unwrap();
        d.digits = payload.len() as i64;
        return finalized(d, ctx, st);
    }
    if starts("inf") {
        let tail = &rest[3..];
        if tail.is_empty() || tail.eq_ignore_ascii_case(b"inity") {
            return Dec::inf(sign);
        }
        return syntax(st);
    }
    // the coefficient's first significant digit, the point, the exponent
    let (mut dpoint, mut epos, mut coeff) = (None, None, None);
    let mut j = 0;
    while j < rest.len() {
        let c = rest[j];
        match c {
            b'.' => {
                if dpoint.is_some() || epos.is_some() {
                    return syntax(st);
                }
                dpoint = Some(j);
            }
            b'e' | b'E' => {
                if epos.is_some() {
                    return syntax(st);
                }
                epos = Some(j);
                if matches!(rest.get(j + 1), Some(b'+' | b'-')) {
                    j += 1;
                }
            }
            _ => {
                if !c.is_ascii_digit() {
                    return syntax(st);
                }
                if coeff.is_none() && epos.is_none() {
                    let next_digit = rest.get(j + 1).is_some_and(|x| x.is_ascii_digit());
                    let point_digit = rest.get(j + 1) == Some(&b'.') && rest.get(j + 2).is_some_and(|x| x.is_ascii_digit());
                    if c != b'0' || (!next_digit && !point_digit) {
                        coeff = Some(j);
                    }
                }
            }
        }
        j += 1;
    }
    let Some(coeff) = coeff else {
        return syntax(st);
    };
    let mut end = rest.len();
    let mut exp: i64 = 0;
    if let Some(e) = epos {
        end = e;
        let es = std::str::from_utf8(&rest[e + 1..]).unwrap();
        let digits = es.trim_start_matches(['+', '-']);
        if digits.is_empty() {
            return syntax(st);
        }
        exp = match es.parse::<i64>() {
            Ok(v) => v,
            // out of range: saturated, as strtoll
            Err(_) => {
                if es.starts_with('-') {
                    i64::MIN
                } else {
                    i64::MAX
                }
            }
        };
    }
    let mut digits = (end - coeff) as i64;
    if let Some(dp) = dpoint {
        let frac = (end - dp - 1) as i64;
        if dp > coeff {
            digits -= 1;
        }
        if frac > MAX_PREC {
            return syntax(st);
        }
        exp = if exp < i64::MIN + frac { i64::MIN } else { exp - frac };
    }
    if digits > MAX_PREC {
        return syntax(st);
    }
    if exp > EXP_INF {
        exp = EXP_INF;
    }
    if exp == i64::MIN {
        exp = i64::MIN + 1;
    }
    let ds: String = rest[coeff..end].iter().filter(|&&c| c != b'.').map(|&c| c as char).collect();
    if too_big(ds.len() as i64) {
        return seterror(st, MALLOC_ERROR);
    }
    let c = BigInt::from_radix(&ds, 10).unwrap();
    let d = Dec { sign, kind: Kind::Fin, exp, digits: ds.len() as i64, coef: c };
    finalized(d, ctx, st)
}

/// an exact conversion: InvalidOperation where the string does not fit
pub fn parse_exact(s: &str, st: &mut u32) -> Dec {
    let mut ws = 0;
    let mut d = parse(s, &Ctx::max(), &mut ws);
    if ws & (INEXACT | ROUNDED | CLAMPED) != 0 {
        d = seterror(&mut ws, INVALID_OPERATION);
    }
    *st |= ws & ERRORS;
    d
}

pub const FMT_UPPER: u32 = 0x1;
pub const FMT_TOSCI: u32 = 0x2;
pub const FMT_TOENG: u32 = 0x4;
pub const FMT_EXP: u32 = 0x8;
pub const FMT_FIXED: u32 = 0x10;
pub const FMT_PERCENT: u32 = 0x20;
pub const FMT_SIGN_SPACE: u32 = 0x40;
pub const FMT_SIGN_PLUS: u32 = 0x80;

/// libmpdec's _mpd_to_string; dplace is where the point goes (counted from
/// the coefficient's start) when the flags leave it to the caller. None
/// when the string would be too long to make.
fn to_string(d: &Dec, flags: u32, mut dplace: i64) -> Option<String> {
    let mut out = String::new();
    let sign = if d.sign {
        "-"
    } else if flags & FMT_SIGN_SPACE != 0 {
        " "
    } else if flags & FMT_SIGN_PLUS != 0 {
        "+"
    } else {
        ""
    };
    out.push_str(sign);
    match d.kind {
        Kind::NaN | Kind::SNaN => {
            out.push_str(if d.kind == Kind::NaN { "NaN" } else { "sNaN" });
            if d.digits > 0 {
                out.push_str(&digits_str(&d.coef));
            }
        }
        Kind::Inf => out.push_str("Infinity"),
        Kind::Fin => {
            let ldigits = d.digits + d.exp;
            if flags & FMT_EXP != 0 {
            } else if flags & FMT_FIXED != 0 || (d.exp <= 0 && ldigits > -6) {
                dplace = ldigits;
            } else if flags & FMT_TOENG != 0 {
                if d.zero_coef() {
                    dplace = -1 + (d.exp + 2).rem_euclid(3);
                } else {
                    dplace += (ldigits - 1).rem_euclid(3);
                }
            }
            let size = if dplace <= 0 {
                -dplace + d.digits + 2
            } else if dplace >= d.digits {
                dplace
            } else {
                d.digits
            };
            if too_big(size) {
                return None;
            }
            let cs = digits_str(&d.coef);
            if dplace <= 0 {
                out.push_str("0.");
                out.extend(std::iter::repeat_n('0', -dplace as usize));
                out.push_str(&cs);
            } else if dplace >= d.digits {
                out.push_str(&cs);
                out.extend(std::iter::repeat_n('0', (dplace - d.digits) as usize));
            } else {
                out.push_str(&cs[..dplace as usize]);
                out.push('.');
                out.push_str(&cs[dplace as usize..]);
            }
            if ldigits != dplace || flags & FMT_EXP != 0 {
                out.push(if flags & FMT_UPPER != 0 { 'E' } else { 'e' });
                let e = ldigits as i128 - dplace as i128;
                out.push(if e < 0 { '-' } else { '+' });
                out.push_str(&e.abs().to_string());
            }
        }
    }
    if flags & FMT_PERCENT != 0 {
        out.push('%');
    }
    Some(out)
}

pub fn to_sci(d: &Dec, upper: bool) -> String {
    to_string(d, FMT_TOSCI | if upper { FMT_UPPER } else { 0 }, 1).unwrap_or_default()
}

pub fn to_eng(d: &Dec, upper: bool) -> String {
    to_string(d, FMT_TOENG | if upper { FMT_UPPER } else { 0 }, 1).unwrap_or_default()
}

/// a parsed format specification, as libmpdec's mpd_spec_t
#[derive(Clone, Debug)]
pub struct Spec {
    pub min_width: i64,
    pub prec: i64,
    pub typ: char,
    pub align: char,
    pub sign: char,
    pub fill: char,
    pub dot: String,
    pub sep: String,
    /// group lengths, as the bytes of a C grouping string up to its NUL
    pub grouping: Vec<u8>,
}

/// libmpdec's mpd_parse_fmt_str; None for what it does not take
pub fn parse_fmt(fmt: &str, caps: bool) -> Option<Spec> {
    let mut spec = Spec { min_width: 0, prec: -1, typ: if caps { 'G' } else { 'g' }, align: '>', sign: '-', fill: ' ', dot: String::new(), sep: String::new(), grouping: vec![] };
    let c: Vec<char> = fmt.chars().collect();
    let at = |i: usize| c.get(i).copied().unwrap_or('\0');
    let is_align = |ch: char| matches!(ch, '<' | '>' | '=' | '^');
    let mut i = 0;
    let mut have_align = false;
    if !c.is_empty() && is_align(at(1)) {
        spec.fill = c[0];
        spec.align = c[1];
        i = 2;
        have_align = true;
    } else if is_align(at(0)) {
        spec.align = c[0];
        i = 1;
        have_align = true;
    }
    if matches!(at(i), '+' | '-' | ' ') {
        spec.sign = at(i);
        i += 1;
    }
    if at(i) == '0' {
        if have_align {
            return None;
        }
        spec.align = 'z';
        spec.fill = '0';
        i += 1;
    }
    let number = |i: &mut usize| -> Option<i64> {
        let start = *i;
        while at(*i).is_ascii_digit() {
            *i += 1;
        }
        c[start..*i].iter().collect::<String>().parse::<i64>().ok()
    };
    if at(i).is_ascii_digit() {
        if at(i) == '0' {
            return None;
        }
        spec.min_width = number(&mut i)?;
    }
    if at(i) == ',' {
        spec.dot = ".".into();
        spec.sep = ",".into();
        spec.grouping = vec![3, 3];
        i += 1;
    }
    if at(i) == '.' {
        i += 1;
        if !at(i).is_ascii_digit() {
            return None;
        }
        spec.prec = number(&mut i)?;
    }
    match at(i) {
        'E' | 'e' | 'F' | 'f' | 'G' | 'g' | '%' => {
            spec.typ = at(i);
            i += 1;
        }
        'N' | 'n' => {
            // the C locale's conventions
            if !spec.sep.is_empty() {
                return None;
            }
            spec.typ = if at(i) == 'N' { 'G' } else { 'g' };
            spec.dot = ".".into();
            spec.sep = String::new();
            spec.grouping = vec![];
            i += 1;
        }
        _ => {}
    }
    if i != c.len() {
        return None;
    }
    Some(spec)
}

/// libmpdec's _mpd_round for formatting: prec significant digits
fn fmt_round(a: &Dec, prec: i64, ctx: &Ctx, st: &mut u32) -> Dec {
    let exp = a.exp + a.digits - prec;
    if a.is_special() || a.zero_coef() {
        return a.clone();
    }
    let r = rescale_fmt(a, exp, ctx, st);
    if r.digits > prec {
        return rescale_fmt(&r, exp + 1, ctx, st);
    }
    r
}

/// thousands separators and the decimal point put in a numeric string
fn apply_lconv(s: &str, spec: &Spec) -> (String, i64) {
    let b = s.as_bytes();
    let mut i = 0;
    let sign = if matches!(b.first(), Some(b'+' | b'-' | b' ')) {
        i = 1;
        Some(b[0] as char)
    } else {
        None
    };
    let istart = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let intpart = &s[istart..i];
    let dot = if b.get(i) == Some(&b'.') {
        i += 1;
        true
    } else {
        false
    };
    let rest = &s[i..];
    if !dot && (spec.sep.is_empty() || spec.grouping.is_empty()) {
        return (s.to_string(), s.chars().count() as i64);
    }
    // built back to front, as libmpdec does
    let mut parts: Vec<String> = vec![rest.to_string()];
    let mut nchars = rest.chars().count() as i64;
    if dot {
        parts.push(spec.dot.clone());
        nchars += (!spec.dot.is_empty()) as i64;
    }
    let n_sign = sign.is_some() as i64;
    let g = |k: usize| spec.grouping.get(k).copied().unwrap_or(0) as i64;
    let mut gi = 0;
    let mut consume = g(0);
    let mut n_src = intpart.len() as i64;
    let mut pad = false;
    loop {
        if g(gi) == 0 || g(gi) == 127 || consume > n_src {
            consume = n_src;
        }
        n_src -= consume;
        if pad {
            parts.push("0".repeat(consume as usize));
        } else {
            parts.push(intpart[n_src as usize..(n_src + consume) as usize].to_string());
        }
        nchars += consume;
        if n_src == 0 {
            if spec.align == 'z' && nchars + n_sign < spec.min_width {
                n_src = spec.min_width - (nchars + n_sign);
                consume = g(gi) - consume;
                pad = true;
                continue;
            }
            break;
        }
        if !spec.sep.is_empty() {
            if pad && n_src > 1 {
                n_src -= 1;
            }
            parts.push(spec.sep.clone());
            nchars += 1;
        }
        if g(gi) != 0 && g(gi + 1) != 0 {
            gi += 1;
        }
        consume = g(gi);
    }
    if let Some(sc) = sign {
        parts.push(sc.to_string());
        nchars += 1;
    }
    parts.reverse();
    (parts.concat(), nchars)
}

/// libmpdec's mpd_qformat_spec; None (with a status) when it cannot
pub fn format(dec: &Dec, spec: &Spec, ctx: &Ctx, st: &mut u32) -> Option<String> {
    if spec.min_width > MAX_PREC {
        *st |= INVALID_OPERATION;
        return None;
    }
    let mut spec = spec.clone();
    let mut typ = spec.typ;
    let mut flags = 0;
    let mut dplace = 1;
    let mut dec = dec.clone();
    if typ.is_ascii_uppercase() {
        typ = typ.to_ascii_lowercase();
        flags |= FMT_UPPER;
    }
    if spec.sign == ' ' {
        flags |= FMT_SIGN_SPACE;
    } else if spec.sign == '+' {
        flags |= FMT_SIGN_PLUS;
    }
    if dec.is_special() {
        if spec.align == 'z' {
            spec.fill = ' ';
            spec.align = '>';
        }
        if typ == '%' {
            flags |= FMT_PERCENT;
        }
    } else {
        let mut ws = 0;
        match typ {
            'g' => flags |= FMT_TOSCI,
            'e' => flags |= FMT_EXP,
            '%' => {
                flags |= FMT_PERCENT | FMT_FIXED;
                dec.exp += 2;
                typ = 'f';
            }
            _ => flags |= FMT_FIXED,
        }
        if spec.prec >= 0 {
            if spec.prec > MAX_PREC {
                *st |= INVALID_OPERATION;
                return None;
            }
            match typ {
                'g' => {
                    let p = if spec.prec == 0 { 1 } else { spec.prec };
                    if dec.digits > p {
                        dec = fmt_round(&dec, p, ctx, &mut ws);
                    }
                }
                'e' => {
                    if dec.zero_coef() {
                        dplace = 1 - spec.prec;
                    } else {
                        dec = fmt_round(&dec, spec.prec + 1, ctx, &mut ws);
                    }
                }
                _ => dec = rescale(&dec, -spec.prec, ctx, &mut ws),
            }
        }
        if typ == 'f' && dec.is_zero() && dec.exp > 0 {
            dec = rescale(&dec, 0, ctx, &mut ws);
        }
        if ws & ERRORS != 0 {
            *st |= ws & ERRORS;
            return None;
        }
    }
    let Some(mut s) = to_string(&dec, flags, dplace) else {
        *st |= MALLOC_ERROR;
        return None;
    };
    let mut nchars = s.len() as i64;
    if !spec.dot.is_empty() && !dec.is_special() {
        if nchars > MAX_PREC + 36 {
            *st |= INVALID_OPERATION;
            return None;
        }
        (s, nchars) = apply_lconv(&s, &spec);
    }
    if spec.min_width > nchars {
        let add = (spec.min_width - nchars) as usize;
        if too_big(add as i64) {
            *st |= MALLOC_ERROR;
            return None;
        }
        let align = if spec.align == 'z' { '=' } else { spec.align };
        let (lpad, rpad) = match align {
            '<' => (0, add),
            '>' | '=' => (add, 0),
            _ => (add / 2, add - add / 2),
        };
        let fill = |n: usize| std::iter::repeat_n(spec.fill, n).collect::<String>();
        let mut out = String::new();
        let mut body = s.as_str();
        if align == '=' && matches!(body.chars().next(), Some('-' | '+' | ' ')) {
            out.push_str(&body[..1]);
            body = &body[1..];
        }
        out.push_str(&fill(lpad));
        out.push_str(body);
        out.push_str(&fill(rpad));
        s = out;
    }
    Some(s)
}
