use crate::{CommandHandler, FromInvoke, InvokeArgs, InvokeReply};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use tauri::{Invoke, InvokeError, InvokeResolver, Runtime};

impl<R, T> FromInvoke<R> for T
where
    R: Runtime,
    T: DeserializeOwned,
{
    fn from_invoke(arg_name: &str, invoke: &Invoke<R>) -> Self {
        serde_json::from_value(invoke.message.payload()[arg_name].clone()).unwrap()
    }
}

impl<R: Runtime, Fut, T> InvokeReply<R> for Fut
where
    Fut: Future<Output = Result<T, anyhow::Error>> + Send + Sync + 'static,
    T: Serialize,
{
    fn reply(self, resolver: InvokeResolver<R>) {
        resolver.respond_async(async move { self.await.map_err(InvokeError::from_anyhow) })
    }
}

