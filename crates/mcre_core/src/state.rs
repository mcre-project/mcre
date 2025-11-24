use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, Unexpected, Visitor},
};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StateValue {
    Int(u8),
    Bool(bool),
    String(String),
}

impl<'de> Deserialize<'de> for StateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StateValueVisitor;

        impl<'de> Visitor<'de> for StateValueVisitor {
            type Value = StateValue;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a bool, integer, or string containing one")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StateValue::Bool(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let v: u8 = v.try_into().map_err(|_| {
                    E::invalid_value(Unexpected::Unsigned(v), &"a u8 value (0–255)")
                })?;
                Ok(StateValue::Int(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Try bool
                if v == "true" {
                    return Ok(StateValue::Bool(true));
                }
                if v == "false" {
                    return Ok(StateValue::Bool(false));
                }

                // Try integer
                if let Ok(n) = v.parse::<u8>() {
                    return Ok(StateValue::Int(n));
                }

                // Fallback: string
                Ok(StateValue::String(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(StateValueVisitor)
    }
}
