//! Implements automatically calling functions after model events, e.g. when a model is saved, created, etc.
//!
//! This is currently a work in progress.

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::BTreeMap;
use super::{Model};
use std::sync::{Arc};
use tokio::sync::RwLock;
use crate::{model::{FromRow, Query}, prelude::async_trait};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackKind {
    Insert,
    Update,
    Delete
}

impl std::fmt::Display for CallbackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "Insert"),
            Self::Update => write!(f, "Update"),
            Self::Delete => write!(f, "Delete")
        }
    }
}

impl std::str::FromStr for  CallbackKind {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Insert" => Ok(Self::Insert),
            "Update" => Ok(Self::Update),
            "Delete" => Ok(Self::Delete),
            _ => Err("No matching CallbackKind")
        }
    }
}

impl<T: FromRow> TryFrom<&Query<T>> for CallbackKind {
    type Error = &'static str;
    fn try_from(value: &Query<T>) -> Result<Self, Self::Error> {
        match value {
            Query::Insert(_) => Ok(Self::Insert),
            Query::Update(_) => Ok(Self::Update),
            _ => Err("No matching CallbackKind")
        }
    }
}

#[derive(Default)]
pub struct CallbackRegistry {
    inner: Arc<RwLock<BTreeMap<&'static str, BTreeMap<CallbackKind, Vec<Box<dyn InnerCallback>>>>>>
}

impl CallbackRegistry {
    pub async fn add_callback(&self, table: &'static str, kind: CallbackKind, callback: Box<dyn InnerCallback>) {
        let mut map = self.inner.write().await;
        map.entry(table).or_default().entry(kind).or_default().push(callback);
    }
    pub async fn apply<T: Model + for<'de> Deserialize<'de>>(&self, kind: CallbackKind, data: T) -> T {
        let map = self.inner.read().await;
        if let Some(inner_map) = map.get(T::table_name()) {
            if let Some(callbacks) = inner_map.get(&kind) {
                let mut data = data.to_json().unwrap();
                for callback in callbacks.iter() {
                    data = callback.call(data.clone()).await;
                }
                return serde_json::from_value(data).unwrap();
            }
        }
        data
    }
}

pub static CALLBACK_REGISTRY: Lazy<CallbackRegistry> = Lazy::new(|| CallbackRegistry::default());

#[async_trait]
pub trait Callback<T: Model>: Default+Sync+Send {
    async fn callback(mut self, data: T) -> T;
    fn table_name() -> &'static str {T::table_name()}
}

#[async_trait]
pub trait InnerCallback: Sync+Send {
    async fn call(&self, data: serde_json::Value) -> serde_json::Value;
}


#[macro_export]
macro_rules! register_callback {
    ($callback:ident, $kind:path) => {
        #[allow(non_local_definitions)]
        #[async_trait]
        impl $crate::model::callbacks::InnerCallback for $callback {
            async fn call(&self, data: serde_json::Value) -> serde_json::Value {
                $crate::model::callbacks::Callback::callback($callback::default(), serde_json::from_value(data).unwrap()).await.to_json().unwrap()  
            }
        }
        $crate::model::callbacks::CALLBACK_REGISTRY.add_callback($callback::table_name(), $kind, Box::new($callback::default())).await;
    };
}

#[macro_export]
macro_rules! apply_callback {
    ($kind:ident, $value:ident) => {
        if let Ok(kind) = $kind {
            $crate::model::callbacks::CALLBACK_REGISTRY.apply(kind, $value).await
        } else {
            $value
        }
    };
}
