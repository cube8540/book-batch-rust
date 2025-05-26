use crate::item::{RawNumber, RawValue};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use tracing::warn;

impl Display for RawNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RawNumber::UnsignedInt(n) => write!(f, "{:?}", n),
            RawNumber::SignedInt(n) => write!(f, "{:?}", n),
            RawNumber::Float(n) => write!(f, "{:?}", n),
            RawNumber::Undefined => write!(f, "Undefined value is NaN or Infinity"),
        }
    }
}

impl PartialEq for RawNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RawNumber::UnsignedInt(a), RawNumber::UnsignedInt(b)) => a == b,
            (RawNumber::SignedInt(a), RawNumber::SignedInt(b)) => a == b,
            // TODO 실수 비교문 변경
            (RawNumber::Float(a), RawNumber::Float(b)) => a == b,
            (RawNumber::Undefined, RawNumber::Undefined) => true,
            _ => false,
        }
    }
}

impl Eq for RawNumber {}

impl From<i32> for RawNumber {

    fn from(value: i32) -> Self {
        Self::SignedInt(value as i64)
    }
}

impl From<i64> for RawNumber {

    fn from(value: i64) -> Self {
        Self::SignedInt(value)
    }
}

impl From<u32> for RawNumber {

    fn from(value: u32) -> Self {
        Self::UnsignedInt(value as u64)
    }
}

impl From<u64> for RawNumber {

    fn from(value: u64) -> Self {
        Self::UnsignedInt(value)
    }
}

impl From<f32> for RawNumber {

    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}

impl From<f64> for RawNumber {

    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<isize> for RawNumber {

    fn from(value: isize) -> Self {
        Self::SignedInt(value as i64)
    }
}

impl From<usize> for RawNumber {

    fn from(value: usize) -> Self {
        Self::UnsignedInt(value as u64)
    }
}

impl From<serde_json::Number> for RawNumber {

    fn from(value: serde_json::Number) -> Self {
        if value.is_i64() {
            Self::SignedInt(value.as_i64().unwrap())
        } else if value.is_u64() {
            Self::UnsignedInt(value.as_u64().unwrap())
        } else if value.is_f64() {
            Self::Float(value.as_f64().unwrap())
        } else {
            warn!("Unknown number type: {:?}", value);
            Self::Undefined
        }
    }
}

impl From<i32> for RawValue {
    fn from(value: i32) -> Self {
        Self::Number(RawNumber::SignedInt(value as i64))
    }
}

impl From<i64> for RawValue {
    fn from(value: i64) -> Self {
        Self::Number(RawNumber::SignedInt(value))
    }
}

impl From<u32> for RawValue {
    fn from(value: u32) -> Self {
        Self::Number(RawNumber::UnsignedInt(value as u64))
    }
}

impl From<u64> for RawValue {
    fn from(value: u64) -> Self {
        Self::Number(RawNumber::UnsignedInt(value))
    }
}

impl From<f32> for RawValue {
    fn from(value: f32) -> Self {
        Self::Number(RawNumber::Float(value as f64))
    }
}

impl From<f64> for RawValue {
    fn from(value: f64) -> Self {
        Self::Number(RawNumber::Float(value))
    }
}

impl From<usize> for RawValue {
    fn from(value: usize) -> Self {
        Self::Number(RawNumber::UnsignedInt(value as u64))
    }
}

impl From<&str> for RawValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<bool> for RawValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<serde_json::Number> for RawValue {
    fn from(value: serde_json::Number) -> Self {
        Self::Number(RawNumber::from(value))
    }
}

impl<T: Into<RawValue>> From<Vec<T>> for RawValue {
    fn from(value: Vec<T>) -> Self {
        let mut arr = Vec::new();
        for v in value {
            arr.push(v.into());
        }
        Self::Array(arr)
    }
}

impl From<serde_json::Map<String, serde_json::Value>> for RawValue {
    fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
        let mut obj = HashMap::new();
        for (k, v) in value {
            obj.insert(k, RawValue::from(v));
        }
        Self::Object(obj)
    }
}

impl From<serde_json::Value> for RawValue {

    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => Self::from(n),
            serde_json::Value::String(s) => Self::from(s.as_str()),
            serde_json::Value::Array(arr) => Self::from(arr),
            serde_json::Value::Object(o) => Self::from(o)
        }
    }
}

impl Display for RawValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RawValue::Null => write!(f, "null"),
            RawValue::Text(text) => write!(f, "{}", text),
            RawValue::Number(number) => write!(f, "{}", number),
            RawValue::Bool(bool) => write!(f, "{}", bool),
            RawValue::Object(object) => write!(f, "{:?}", object),
            RawValue::Array(array) => write!(f, "{:?}", array),
        }
    }
}

impl From<RawNumber> for serde_json::Value {

    fn from(value: RawNumber) -> Self {
        match value {
            RawNumber::Undefined => serde_json::Value::Null,
            RawNumber::UnsignedInt(n) => serde_json::Value::Number(serde_json::Number::from(n.clone())),
            RawNumber::SignedInt(n) => serde_json::Value::Number(serde_json::Number::from(n.clone())),
            RawNumber::Float(n) => {
                let n = serde_json::Number::from_f64(n.clone());
                if n.is_some() {
                    serde_json::Value::from(n.unwrap())
                } else {
                    serde_json::Value::Null
                }
            }
        }
    }
}

impl From<RawValue> for serde_json::Value {

    fn from(value: RawValue) -> Self {
        match value {
            RawValue::Null => Self::Null,
            RawValue::Text(s) => Self::String(s),
            RawValue::Number(n) => Self::from(n),
            RawValue::Bool(b) => Self::Bool(b),
            RawValue::Array(arr) => Self::from(arr),
            RawValue::Object(o) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in o {
                    obj.insert(k, Self::from(v));
                }
                serde_json::Value::Object(obj)
            },
        }
    }
}