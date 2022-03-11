use serde::de::DeserializeOwned;
use tauri::{Invoke, Runtime};

use crate::FromInvoke;

pub struct Hidden<T>
where
    T: DeserializeOwned,
{
    inner: T,
}

impl<T> std::ops::Deref for Hidden<T>
where
    T: DeserializeOwned,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Hidden<T>
where
    T: DeserializeOwned,
{
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<R, T> FromInvoke<R> for Hidden<T>
where
    T: DeserializeOwned,
    R: Runtime,
{
    fn from_invoke(arg_name: &str, invoke: &Invoke<R>) -> Self {
        Self {
            inner: serde_json::from_value(invoke.message.payload()[arg_name].clone()).unwrap(),
        }
    }

    fn generate_schema(
        _gen: &mut schemars::gen::SchemaGenerator,
    ) -> Option<schemars::schema::Schema> {
        None
    }
}
