use serde::de::{self, Deserializer, Visitor};
use serde::ser::{self, Serializer};

pub mod secure_id {
    use super::*;
    use crate::crypto::{decrypt_number, encrypt_number};

    struct SecureIdVisitor;

    impl<'de> Visitor<'de> for SecureIdVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a secure id to be a string")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(s.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    pub fn serialize<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = value {
            let value = encrypt_number(*value).map_err(ser::Error::custom)?;
            serializer.serialize_str(&value)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = deserializer.deserialize_string(SecureIdVisitor)?;

        if let Some(value) = value {
            if let Ok(value) = value.parse::<i64>() {
                Ok(Some(value))
            } else {
                let value = decrypt_number(&value).map_err(de::Error::custom)?;
                Ok(Some(value))
            }
        } else {
            Ok(None)
        }
    }
}
