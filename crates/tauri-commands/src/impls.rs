use crate::{CommandHandler, FromInvoke, InvokeArgs, TauriState, TauriWindow, TauriStateManager};
use tauri::{Invoke, Runtime};

#[cfg(not(feature = "codegen"))]
mod simple;

#[cfg(feature = "codegen")]
mod codegen;

macro_rules! impl_invoke_args {
    (
        $({$($arg:ident),*})*
    ) => {
        $(
            impl<R, $($arg,)*> InvokeArgs<R> for ($($arg,)*) where R: Runtime, $($arg: FromInvoke<R>),* {
                #[allow(unused_variables, clippy::unused_unit)]
                fn invoke_args(invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError> {
                    Ok(($(
                        $arg::from_invoke(stringify!($arg), invoke)?,
                    )*))
                }

                #[cfg(feature = "codegen")]
                #[allow(unused_variables, clippy::unused_unit)]
                fn args(gen: &mut schemars::gen::SchemaGenerator) -> Vec<crate::codegen::CommandArg> {
                    [$(
                        $arg::schema(gen).map(|schema| crate::codegen::CommandArg {hidden: false, name: std::borrow::Cow::Borrowed(stringify!($arg)), schema})
                        .unwrap_or_else(|| crate::codegen::CommandArg {hidden: true, name: std::borrow::Cow::Borrowed(stringify!($arg)), schema: schemars::schema::Schema::Bool(false)}),
                    )*].into_iter().collect()
                }
            }
        )*
    };
}

impl_invoke_args! {
    {}
    { _1 }
    { _1, _2 }
    { _1, _2, _3 }
    { _1, _2, _3, _4 }
    { _1, _2, _3, _4, _5 }
    { _1, _2, _3, _4, _5, _6 }
    { _1, _2, _3, _4, _5, _6, _7 }
    { _1, _2, _3, _4, _5, _6, _7, _8 }
    { _1, _2, _3, _4, _5, _6, _7, _8, _9 }
    { _1, _2, _3, _4, _5, _6, _7, _8, _9, _10 }
}

macro_rules! impl_fn_handler {
    ($({$($arg:ident),*})*) => {
        $(
            impl<F, O, $($arg,)*> CommandHandler<($($arg,)*)> for F
            where
                F: Fn($($arg),*) -> O + 'static,
            {
                type Output = O;

                #[inline]
                #[allow(non_snake_case)]
                fn handle(&self, ($($arg,)*): ($($arg,)*)) -> Self::Output {
                    (self)($($arg,)*)
                }
            }
        )*
    };
}

impl_fn_handler! {
    {}
    { _1 }
    { _1, _2 }
    { _1, _2, _3 }
    { _1, _2, _3, _4 }
    { _1, _2, _3, _4, _5 }
    { _1, _2, _3, _4, _5, _6 }
    { _1, _2, _3, _4, _5, _6, _7 }
    { _1, _2, _3, _4, _5, _6, _7, _8 }
    { _1, _2, _3, _4, _5, _6, _7, _8, _9 }
    { _1, _2, _3, _4, _5, _6, _7, _8, _9, _10 }
}

impl<R: Runtime> FromInvoke<R> for TauriWindow<R> {
    fn from_invoke(_arg_name: &str, invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError> {
        Ok(TauriWindow(invoke.message.window()))
    }
}

impl<R: Runtime> FromInvoke<R> for TauriStateManager {
    fn from_invoke(_arg_name: &str, invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError> {
        Ok(TauriStateManager(invoke.message.state()))
    }
}

impl<R: Runtime, T: Send + Sync + Clone + 'static> FromInvoke<R> for TauriState<T> {
    fn from_invoke(_arg_name: &str, invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError> {
        Ok(TauriState((*invoke.message.state().get::<T>()).clone()))
    }
}
