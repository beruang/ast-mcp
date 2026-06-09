//! Lenient serde deserializers that accept number-or-number-string and bool-or-bool-string.
//!
//! LLMs frequently emit numeric and boolean values as JSON strings (e.g. `"14"` for line numbers,
//! `"true"` for flags). The standard `serde_json::from_value` rejects these with a type-mismatch
//! error. These helpers coerce strings into the target type when the native type fails.

use serde::de::{self, Unexpected, Visitor};
use serde::Deserializer;
use std::fmt;

// ── u32 ──

pub fn deserialize_lenient_u32<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = u32;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a u32 or a string that parses to u32")
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u32, E> {
            u32::try_from(v).map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<u32, E> {
            u32::try_from(v).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<u32, E> {
            v.parse::<u32>().map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<u32, E> {
            if v >= 0.0 && v <= u32::MAX as f64 && v.fract() == 0.0 {
                Ok(v as u32)
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

pub fn deserialize_lenient_opt_u32<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<u32>, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<u32>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("null, a u32, or a string that parses to u32")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<u32>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<u32>, E> {
            Ok(None)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<u32>, E> {
            u32::try_from(v).map(Some).map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<u32>, E> {
            u32::try_from(v).map(Some).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<u32>, E> {
            v.parse::<u32>().map(Some).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<u32>, E> {
            if v >= 0.0 && v <= u32::MAX as f64 && v.fract() == 0.0 {
                Ok(Some(v as u32))
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

// ── u64 ──

pub fn deserialize_lenient_u64<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = u64;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a u64 or a string that parses to u64")
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
            Ok(v)
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
            u64::try_from(v).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
            v.parse::<u64>().map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<u64, E> {
            if v >= 0.0 && v <= u64::MAX as f64 && v.fract() == 0.0 {
                Ok(v as u64)
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

pub fn deserialize_lenient_opt_u64<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<u64>, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<u64>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("null, a u64, or a string that parses to u64")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<u64>, E> {
            Ok(Some(v))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<u64>, E> {
            u64::try_from(v).map(Some).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<u64>, E> {
            v.parse::<u64>().map(Some).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<u64>, E> {
            if v >= 0.0 && v <= u64::MAX as f64 && v.fract() == 0.0 {
                Ok(Some(v as u64))
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

// ── usize ──

pub fn deserialize_lenient_usize<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = usize;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a usize or a string that parses to usize")
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<usize, E> {
            usize::try_from(v).map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<usize, E> {
            usize::try_from(v).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<usize, E> {
            v.parse::<usize>().map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<usize, E> {
            if v >= 0.0 && v <= usize::MAX as f64 && v.fract() == 0.0 {
                Ok(v as usize)
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

pub fn deserialize_lenient_opt_usize<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<usize>, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<usize>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("null, a usize, or a string that parses to usize")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<usize>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<usize>, E> {
            Ok(None)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<usize>, E> {
            usize::try_from(v)
                .map(Some)
                .map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<usize>, E> {
            usize::try_from(v).map(Some).map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<usize>, E> {
            v.parse::<usize>().map(Some).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<usize>, E> {
            if v >= 0.0 && v <= usize::MAX as f64 && v.fract() == 0.0 {
                Ok(Some(v as usize))
            } else {
                Err(E::invalid_value(Unexpected::Float(v), &self))
            }
        }
    }
    d.deserialize_any(V)
}

// ── bool ──

pub fn deserialize_lenient_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = bool;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a boolean or a string 'true'/'false'")
        }
        fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
            Ok(v)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
            match v.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(true),
                "false" | "0" | "no" => Ok(false),
                _ => Err(E::invalid_value(Unexpected::Str(v), &self)),
            }
        }
    }
    d.deserialize_any(V)
}

pub fn deserialize_lenient_opt_bool<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<bool>, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<bool>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("null, a boolean, or a string 'true'/'false'")
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<bool>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<bool>, E> {
            Ok(None)
        }
        fn visit_bool<E: de::Error>(self, v: bool) -> Result<Option<bool>, E> {
            Ok(Some(v))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<bool>, E> {
            match v.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Some(true)),
                "false" | "0" | "no" => Ok(Some(false)),
                _ => Err(E::invalid_value(Unexpected::Str(v), &self)),
            }
        }
    }
    d.deserialize_any(V)
}
