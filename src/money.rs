//! Centralized monetary value type for MiniERP.
//!
//! This module replaces all `f64` usage for monetary values with
//! `Money`, a newtype wrapper around `rust_decimal::Decimal` that provides
//! exact decimal arithmetic, eliminates floating-point rounding errors,
//! and integrates directly with rusqlite for database operations.
//!
//! # Usage
//!
//! ```ignore
//! use crate::money::{Money, money};
//!
//! let price = money("19.99");
//! let total = Money::from(100) * price;
//! assert_eq!(total, money("1999.00"));
//! ```

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Monetary value — a newtype around `Decimal` with exact arithmetic
/// and rusqlite integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(Decimal);

impl Money {
    /// Zero constant.
    pub const ZERO: Money = Money(Decimal::ZERO);

    /// Round to 2 decimal places (half-up).
    pub fn round_dp2(self) -> Self {
        Money(self.0.round_dp(2))
    }

    /// Round to N decimal places.
    pub fn round_dp(self, dp: u32) -> Self {
        Money(self.0.round_dp(dp))
    }

    /// Get the absolute value.
    pub fn abs(self) -> Self {
        Money(self.0.abs())
    }

    /// Check if the value is zero.
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Check if the value is positive.
    pub fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    /// Check if the value is negative.
    pub fn is_negative(self) -> bool {
        self.0.is_sign_negative()
    }

    /// Get the sign of the value (-1, 0, or 1).
    pub fn signum(self) -> i32 {
        self.0.signum().to_string().parse::<i32>().unwrap_or(0)
    }

    /// Get the inner Decimal value.
    pub fn inner(&self) -> &Decimal {
        &self.0
    }

    /// Convert to f64 (use sparingly — loses precision).
    pub fn to_f64(self) -> f64 {
        self.0.to_string().parse::<f64>().unwrap_or(0.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default & Sum
// ─────────────────────────────────────────────────────────────────────────────

impl Default for Money {
    fn default() -> Self {
        Money(Decimal::ZERO)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money(Decimal::ZERO), |acc, v| acc + v)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Money(Decimal::ZERO), |acc, v| acc + *v)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display & Debug
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// From / Into conversions
// ─────────────────────────────────────────────────────────────────────────────

impl From<i32> for Money {
    fn from(v: i32) -> Self {
        Money(Decimal::from(v))
    }
}

impl From<i64> for Money {
    fn from(v: i64) -> Self {
        Money(Decimal::from(v))
    }
}

impl From<u32> for Money {
    fn from(v: u32) -> Self {
        Money(Decimal::from(v))
    }
}

impl From<u64> for Money {
    fn from(v: u64) -> Self {
        Money(Decimal::from(v))
    }
}

impl From<f64> for Money {
    fn from(v: f64) -> Self {
        Money(Decimal::try_from(v).unwrap_or(Decimal::ZERO))
    }
}

impl From<Decimal> for Money {
    fn from(v: Decimal) -> Self {
        Money(v)
    }
}

impl From<Money> for Decimal {
    fn from(m: Money) -> Self {
        m.0
    }
}

impl TryFrom<&str> for Money {
    type Error = rust_decimal::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Money(Decimal::from_str(s)?))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic traits — Money op Money
// ─────────────────────────────────────────────────────────────────────────────

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Money(self.0 - rhs.0)
    }
}

impl Mul for Money {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Money(self.0 * rhs.0)
    }
}

impl Div for Money {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.0.is_zero() {
            Money(Decimal::ZERO)
        } else {
            Money(self.0 / rhs.0)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic traits — Money op i64
// ─────────────────────────────────────────────────────────────────────────────

impl Add<i64> for Money {
    type Output = Self;
    fn add(self, rhs: i64) -> Self {
        Money(self.0 + Decimal::from(rhs))
    }
}

impl Sub<i64> for Money {
    type Output = Self;
    fn sub(self, rhs: i64) -> Self {
        Money(self.0 - Decimal::from(rhs))
    }
}

impl Mul<i64> for Money {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Money(self.0 * Decimal::from(rhs))
    }
}

impl Div<i64> for Money {
    type Output = Self;
    fn div(self, rhs: i64) -> Self {
        if rhs == 0 {
            Money(Decimal::ZERO)
        } else {
            Money(self.0 / Decimal::from(rhs))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic traits — Money op f64
// ─────────────────────────────────────────────────────────────────────────────

impl Add<f64> for Money {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        Money(self.0 + Decimal::try_from(rhs).unwrap_or(Decimal::ZERO))
    }
}

impl Sub<f64> for Money {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        Money(self.0 - Decimal::try_from(rhs).unwrap_or(Decimal::ZERO))
    }
}

impl Mul<f64> for Money {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Money(self.0 * Decimal::try_from(rhs).unwrap_or(Decimal::ZERO))
    }
}

impl Div<f64> for Money {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        let d = Decimal::try_from(rhs).unwrap_or(Decimal::ZERO);
        if d.is_zero() {
            Money(Decimal::ZERO)
        } else {
            Money(self.0 / d)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Assign traits
// ─────────────────────────────────────────────────────────────────────────────

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<i64> for Money {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += Decimal::from(rhs);
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign for Money {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl DivAssign for Money {
    fn div_assign(&mut self, rhs: Self) {
        if !rhs.0.is_zero() {
            self.0 /= rhs.0;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Neg
// ─────────────────────────────────────────────────────────────────────────────

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self {
        Money(-self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Reverse arithmetic (i64 op Money, f64 op Money)
// ─────────────────────────────────────────────────────────────────────────────

impl Add<Money> for i64 {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(Decimal::from(self) + rhs.0)
    }
}

impl Sub<Money> for i64 {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(Decimal::from(self) - rhs.0)
    }
}

impl Mul<Money> for i64 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money(Decimal::from(self) * rhs.0)
    }
}

impl Add<Money> for f64 {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(Decimal::try_from(self).unwrap_or(Decimal::ZERO) + rhs.0)
    }
}

impl Sub<Money> for f64 {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(Decimal::try_from(self).unwrap_or(Decimal::ZERO) - rhs.0)
    }
}

impl Mul<Money> for f64 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money(Decimal::try_from(self).unwrap_or(Decimal::ZERO) * rhs.0)
    }
}

impl Add<Money> for i32 {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(Decimal::from(self) + rhs.0)
    }
}

impl Mul<Money> for i32 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money(Decimal::from(self) * rhs.0)
    }
}

impl Mul<Money> for u32 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money(Decimal::from(self) * rhs.0)
    }
}

impl Mul<Money> for u64 {
    type Output = Money;
    fn mul(self, rhs: Money) -> Money {
        Money(Decimal::from(self) * rhs.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Serde (for JSON serialization in API responses)
// ─────────────────────────────────────────────────────────────────────────────

impl serde::Serialize for Money {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize as f64 for JSON compatibility
        serializer.serialize_f64(self.to_f64())
    }
}

impl<'de> serde::Deserialize<'de> for Money {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de;

        struct MoneyVisitor;

        impl<'de> de::Visitor<'de> for MoneyVisitor {
            type Value = Money;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a monetary amount (number or string)")
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Money, E> {
                Ok(Money::from(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Money, E> {
                Ok(Money::from(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Money, E> {
                Ok(Money::from(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Money, E> {
                Decimal::from_str(v)
                    .map(Money)
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(MoneyVisitor)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// rusqlite integration (FromSql / ToSql) — native only
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl rusqlite::types::FromSql for Money {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        // Try TEXT first (new format: Decimal string), fall back to REAL (old format: f64)
        match value {
            rusqlite::types::ValueRef::Text(bytes) => {
                let s = std::str::from_utf8(bytes).map_err(|e| rusqlite::types::FromSqlError::InvalidType)?;
                Decimal::from_str(s)
                    .map(Money)
                    .map_err(|e| rusqlite::types::FromSqlError::InvalidType)
            }
            rusqlite::types::ValueRef::Real(f) => Ok(Money::from(f)),
            rusqlite::types::ValueRef::Integer(i) => Ok(Money::from(i)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl rusqlite::types::ToSql for Money {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        // Store as TEXT (Decimal string) for full precision
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(self.0.to_string()),
        ))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Standalone functions (kept for backward compatibility)
// ─────────────────────────────────────────────────────────────────────────────

/// Shorthand to create a Money value from a string literal.
pub fn money(s: &str) -> Money {
    Money::try_from(s).expect("invalid money string")
}

/// Shorthand to create a Money value from an integer.
pub fn money_int(v: i64) -> Money {
    Money::from(v)
}

/// Shorthand to create a Money value from an f64 (use sparingly).
pub fn money_from_f64(v: f64) -> Money {
    Money::from(v)
}

/// Zero constant (module-level for convenience).
pub const ZERO: Money = Money::ZERO;

/// Round a monetary amount to 2 decimal places.
pub fn round_money(v: Money) -> Money {
    v.round_dp2()
}

/// Convert a Money value to f64 for legacy APIs and Dioxus component props.
pub fn to_f64(v: Money) -> f64 {
    v.to_f64()
}

/// Parse a Money value from an optional f64.
pub fn from_optional_f64(v: Option<f64>) -> Money {
    v.map(Money::from).unwrap_or(ZERO)
}

/// Format a Money value as a currency string (e.g., "Rs. 1,234.56").
pub fn format_money(v: Money) -> String {
    let rounded = round_money(v);
    let abs = if rounded < ZERO { -rounded } else { rounded };
    let sign = if rounded < ZERO { "-" } else { "" };

    let s = abs.to_string();
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "00" };

    let int_with_commas: String = int_part
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap_or_default()
        .join(",");

    let dec_padded = match dec_part.len() {
        0 => "00".to_string(),
        1 => format!("{}0", dec_part),
        _ => dec_part[..2].to_string(),
    };

    format!("Rs. {}{}.{}", sign, int_with_commas, dec_padded)
}

/// Format a Money value with custom currency code and decimals.
pub fn format_money_with_code(v: Money, code: &str, decimals: u32) -> String {
    let rounded = v.round_dp(decimals);
    let abs = if rounded < ZERO { -rounded } else { rounded };
    let sign = if rounded < ZERO { "-" } else { "" };

    let s = abs.to_string();
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "" };

    let int_with_commas: String = int_part
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap_or_default()
        .join(",");

    let dec_padded = if decimals > 0 {
        let padded = format!("{:0>width$}", dec_part, width = decimals as usize);
        format!(".{}", &padded[..decimals as usize])
    } else {
        String::new()
    };

    format!("{} {}{}{}", code, sign, int_with_commas, dec_padded)
}

/// Parse a Money from a string, returning ZERO on failure.
pub fn parse_money(s: &str) -> Money {
    Money::try_from(s).unwrap_or(ZERO)
}

/// Parse an optional Money from a string.
pub fn parse_optional_money(s: &str) -> Option<Money> {
    if s.trim().is_empty() {
        return None;
    }
    Money::try_from(s).ok()
}

/// Scale a Money value (multiply by a quantity).
pub fn scale(v: Money, qty: Money) -> Money {
    v * qty
}

/// Calculate a percentage of a Money value.
pub fn percent_of(v: Money, pct: Money) -> Money {
    v * pct / Money::from(100)
}

/// Read a Money value from a rusqlite Row column (stored as REAL in SQLite).
#[cfg(not(target_arch = "wasm32"))]
pub fn row_get_money(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Money> {
    let v: f64 = row.get(idx)?;
    Ok(Money::from(v))
}

/// Read an optional Money value from a rusqlite Row column.
#[cfg(not(target_arch = "wasm32"))]
pub fn row_get_optional_money(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<Option<Money>> {
    let v: Option<f64> = row.get(idx)?;
    Ok(v.map(Money::from))
}

/// Bind a Money value to a rusqlite parameter (converts to f64 for REAL column).
#[cfg(not(target_arch = "wasm32"))]
pub fn bind_money(params: &mut Vec<Box<dyn rusqlite::types::ToSql>>, v: Money) {
    params.push(Box::new(v.to_f64()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_basic() {
        let m = money("19.99");
        assert_eq!(m.to_string(), "19.99");
    }

    #[test]
    fn test_money_no_rounding_errors() {
        let a = money("0.1");
        let b = money("0.2");
        let c = a + b;
        assert_eq!(c.to_string(), "0.3");
    }

    #[test]
    fn test_round_money() {
        let m = money("1.235");
        assert_eq!(round_money(m).to_string(), "1.24");
    }

    #[test]
    fn test_format_money() {
        let m = money("1234567.89");
        assert_eq!(format_money(m), "Rs. 1,234,567.89");
    }

    #[test]
    fn test_format_money_negative() {
        let m = money("-500.5");
        assert_eq!(format_money(m), "Rs. -500.50");
    }

    #[test]
    fn test_format_money_zero() {
        assert_eq!(format_money(ZERO), "Rs. 0.00");
    }

    #[test]
    fn test_percent_of() {
        let base = money("1000");
        let pct = percent_of(base, money("17"));
        assert_eq!(pct.to_string(), "170");
    }

    #[test]
    fn test_arithmetic() {
        let a = money("100");
        let b = money("30");
        assert_eq!(a - b, money("70"));
        assert_eq!(a * b, money("3000"));
        assert_eq!(a / b, money("3.333333333333333333333333333"));
    }

    #[test]
    fn test_reverse_arithmetic() {
        let m = money("50");
        assert_eq!(100 + m, money("150"));
        assert_eq!(100 - m, money("50"));
        assert_eq!(3 * m, money("150"));
        assert_eq!(2.0 + m, money("52"));
        assert_eq!(10.0 * m, money("500"));
    }

    #[test]
    fn test_sum() {
        let values = vec![money("10"), money("20"), money("30")];
        let total: Money = values.into_iter().sum();
        assert_eq!(total, money("60"));
    }

    #[test]
    fn test_default() {
        let m = Money::default();
        assert_eq!(m, Money::ZERO);
    }

    #[test]
    fn test_rust_decimal_conversion() {
        let m = money("42.50");
        let d: Decimal = m.into();
        assert_eq!(d.to_string(), "42.50");
        let m2 = Money::from(d);
        assert_eq!(m, m2);
    }
}
