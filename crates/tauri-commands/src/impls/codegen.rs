use crate::{FromInvoke, InvokeReply};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use tauri::{Invoke, InvokeError, InvokeResolver, Runtime};

impl<R, T> FromInvoke<R> for T
where
    R: Runtime,
    T: DeserializeOwned + schemars::JsonSchema,
{
    fn from_invoke(arg_name: &str, invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError> {
        serde_json::from_value(invoke.message.payload()[arg_name].clone())
            .map_err(|err| tauri::InvokeError::from_anyhow(err.into()))
    }

    fn schema(gen: &mut schemars::gen::SchemaGenerator) -> Option<schemars::schema::Schema> {
        Some(gen.subschema_for::<Self>())
    }
}

impl<R: Runtime, Fut, T> InvokeReply<R> for Fut
where
    Fut: Future<Output = Result<T, anyhow::Error>> + Send  + 'static,
    T: Serialize + schemars::JsonSchema,
{
    fn reply(self, resolver: InvokeResolver<R>) {
        resolver.respond_async(async move { self.await.map_err(InvokeError::from_anyhow) })
    }

    fn schema(gen: &mut schemars::gen::SchemaGenerator) -> Option<schemars::schema::Schema> {
        Some(gen.subschema_for::<T>())
    }
}
