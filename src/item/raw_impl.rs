use crate::item::{RawNumber, RawValue};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::IntErrorKind;
use tracing::warn;

pub enum ParseError {
    IntError(IntErrorKind),

    FloatError,

    BoolError,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IntError(e) => write!(f, "Invalid integer: {:?}", e),
            ParseError::FloatError => write!(f, "Invalid float"),
            ParseError::BoolError => write!(f, "Invalid boolean"),
        }
    }
}

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

impl AsRef<RawNumber> for RawNumber {
    fn as_ref(&self) -> &RawNumber {
        self
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

impl TryFrom<&RawNumber> for i32 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i as i32),
            RawNumber::SignedInt(i) => Ok(*i as i32),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<i64> for RawNumber {

    fn from(value: i64) -> Self {
        Self::SignedInt(value)
    }
}

impl TryFrom<&RawNumber> for i64 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i as i64),
            RawNumber::SignedInt(i) => Ok(*i),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<u32> for RawNumber {

    fn from(value: u32) -> Self {
        Self::UnsignedInt(value as u64)
    }
}

impl TryFrom<&RawNumber> for u32 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i as u32),
            RawNumber::SignedInt(i) => Ok(*i as u32),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<u64> for RawNumber {

    fn from(value: u64) -> Self {
        Self::UnsignedInt(value)
    }
}

impl TryFrom<&RawNumber> for u64 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i),
            RawNumber::SignedInt(i) => Ok(*i as u64),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<f32> for RawNumber {

    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}

impl TryFrom<&RawNumber> for f32 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::FloatError),
            RawNumber::UnsignedInt(i) => Ok(*i as f32),
            RawNumber::SignedInt(i) => Ok(*i as f32),
            RawNumber::Float(f) => Ok(*f as f32),
        }
    }
}

impl From<f64> for RawNumber {

    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl TryFrom<&RawNumber> for f64 {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::FloatError),
            RawNumber::UnsignedInt(i) => Ok(*i as f64),
            RawNumber::SignedInt(i) => Ok(*i as f64),
            RawNumber::Float(f) => Ok(*f),
        }
    }
}

impl From<isize> for RawNumber {

    fn from(value: isize) -> Self {
        Self::SignedInt(value as i64)
    }
}

impl TryFrom<&RawNumber> for isize {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i as isize),
            RawNumber::SignedInt(i) => Ok(*i as isize),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<usize> for RawNumber {

    fn from(value: usize) -> Self {
        Self::UnsignedInt(value as u64)
    }
}

impl TryFrom<&RawNumber> for usize {
    type Error = ParseError;

    fn try_from(value: &RawNumber) -> Result<Self, Self::Error> {
        match value {
            RawNumber::Undefined => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawNumber::UnsignedInt(i) => Ok(*i as usize),
            RawNumber::SignedInt(i) => Ok(*i as usize),
            RawNumber::Float(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
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

impl Display for RawValue {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RawValue::Null => write!(f, "null"),
            RawValue::Text(s) => s.fmt(f),
            RawValue::Number(n) => n.to_string().fmt(f),
            RawValue::Bool(b) => b.to_string().fmt(f),
            RawValue::Object(m) => {
                let s = m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<String>>().join(", ");
                write!(f, "{{{}}}", s)
            },
            RawValue::Array(arr) => arr.iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .fmt(f)
        }
    }
}

impl AsRef<RawValue> for RawValue {
    fn as_ref(&self) -> &RawValue {
        self
    }
}

impl From<i32> for RawValue {
    fn from(value: i32) -> Self {
        Self::Number(RawNumber::SignedInt(value as i64))
    }
}

impl TryFrom<&RawValue> for i32 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawValue::Text(s) => s.parse::<i32>().map_err(|e| ParseError::IntError(e.kind().clone())),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1) } else { Ok(0) },
            RawValue::Object(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
            RawValue::Array(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<i64> for RawValue {
    fn from(value: i64) -> Self {
        Self::Number(RawNumber::SignedInt(value))
    }
}

impl TryFrom<&RawValue> for i64 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawValue::Text(s) => s.parse::<i64>().map_err(|e| ParseError::IntError(e.kind().clone())),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1) } else { Ok(0) },
            RawValue::Object(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
            RawValue::Array(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<u32> for RawValue {
    fn from(value: u32) -> Self {
        Self::Number(RawNumber::UnsignedInt(value as u64))
    }
}

impl TryFrom<&RawValue> for u32 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawValue::Text(s) => s.parse::<u32>().map_err(|e| ParseError::IntError(e.kind().clone())),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1) } else { Ok(0) },
            RawValue::Object(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
            RawValue::Array(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<u64> for RawValue {
    fn from(value: u64) -> Self {
        Self::Number(RawNumber::UnsignedInt(value))
    }
}

impl TryFrom<&RawValue> for u64 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawValue::Text(s) => s.parse::<u64>().map_err(|e| ParseError::IntError(e.kind().clone())),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1) } else { Ok(0) },
            RawValue::Object(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
            RawValue::Array(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<f32> for RawValue {
    fn from(value: f32) -> Self {
        Self::Number(RawNumber::Float(value as f64))
    }
}

impl TryFrom<&RawValue> for f32 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::FloatError),
            RawValue::Text(s) => s.parse::<f32>().map_err(|e| ParseError::FloatError),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1.0) } else { Ok(0.0) },
            RawValue::Object(_) => Err(ParseError::FloatError),
            RawValue::Array(_) => Err(ParseError::FloatError),
        }
    }
}

impl From<f64> for RawValue {
    fn from(value: f64) -> Self {
        Self::Number(RawNumber::Float(value))
    }
}

impl TryFrom<&RawValue> for f64 {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::FloatError),
            RawValue::Text(s) => s.parse::<f64>().map_err(|e| ParseError::FloatError),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1.0) } else { Ok(0.0) },
            RawValue::Object(_) => Err(ParseError::FloatError),
            RawValue::Array(_) => Err(ParseError::FloatError),
        }
    }
}

impl From<usize> for RawValue {
    fn from(value: usize) -> Self {
        Self::Number(RawNumber::UnsignedInt(value as u64))
    }
}

impl TryFrom<&RawValue> for usize {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::IntError(IntErrorKind::Empty)),
            RawValue::Text(s) => s.parse::<usize>().map_err(|e| ParseError::IntError(e.kind().clone())),
            RawValue::Number(n) => n.try_into(),
            RawValue::Bool(b) => if *b { Ok(1) } else { Ok(0) },
            RawValue::Object(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
            RawValue::Array(_) => Err(ParseError::IntError(IntErrorKind::InvalidDigit)),
        }
    }
}

impl From<&str> for RawValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<&RawValue> for String {

    fn from(value: &RawValue) -> Self {
        value.to_string()
    }
}

impl From<bool> for RawValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl TryFrom<&RawValue> for bool {
    type Error = ParseError;

    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Null => Err(ParseError::BoolError),
            RawValue::Text(_) => Err(ParseError::BoolError),
            RawValue::Number(n) => match n {
                RawNumber::Undefined => Ok(false),
                RawNumber::UnsignedInt(i) => Ok(*i > 0),
                RawNumber::SignedInt(i) => Ok(*i > 0),
                RawNumber::Float(f) => Ok(*f > 0.0),
            }
            RawValue::Bool(b) => Ok(*b),
            RawValue::Object(_) => Err(ParseError::BoolError),
            RawValue::Array(_) => Err(ParseError::BoolError),
        }
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