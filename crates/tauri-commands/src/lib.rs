use std::{borrow::Cow, collections::HashMap, sync::Arc};
use anyhow::anyhow;
use tauri::{Invoke, InvokeResolver, Runtime};

mod impls;

#[cfg(feature = "codegen")]
pub mod codegen;

pub use tauri_commands_macros::command;

pub type CommandResult<T> = Result<T, anyhow::Error>;

/// Workaround to access [`Invoke`] Tauri items, as [`FromInvoke`] cannot be implemented
/// for them due to blanket impls and orphan rules.
#[repr(transparent)]
pub struct TauriStateManager(Arc<tauri::StateManager>);

impl TauriStateManager {
    pub fn into_inner(self) -> Arc<tauri::StateManager> {
        self.0
    }
}

impl std::ops::Deref for TauriStateManager {
    type Target = tauri::StateManager;

    fn deref(&self) -> &Self::Target {
        &(*self.0)
    }
}

/// Workaround to access [`Invoke`] Tauri items, as [`FromInvoke`] cannot be implemented
/// for them due to blanket impls and orphan rules.
#[repr(transparent)]
pub struct TauriWindow<R: Runtime>(tauri::Window<R>);

impl<R: Runtime> TauriWindow<R> {
    pub fn into_inner(self) -> tauri::Window<R> {
        self.0
    }
}

impl<R: Runtime> std::ops::Deref for TauriWindow<R> {
    type Target = tauri::Window<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Workaround to access [`Invoke`] Tauri items, as [`FromInvoke`] cannot be implemented
/// for them due to blanket impls and orphan rules.
#[repr(transparent)]
pub struct TauriState<T>(T)
where
    T: Send + Sync + Clone + 'static;

impl<T> TauriState<T>
where
    T: Send + Sync + Clone + 'static,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for TauriState<T>
where
    T: Send + Sync + Clone + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Command<R: Runtime> {
    handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
    #[cfg(feature = "codegen")]
    pub meta: codegen::CommandMeta,
}

pub trait InvokeArgs<R: Runtime>: Sized {
    fn invoke_args(invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError>;

    #[cfg(feature = "codegen")]
    #[doc(hidden)]
    fn args(_gen: &mut schemars::gen::SchemaGenerator) -> Vec<codegen::CommandArg> {
        Vec::default()
    }
}

trait FromInvoke<R: Runtime>: Sized {
    fn from_invoke(arg_name: &str, invoke: &Invoke<R>) -> Result<Self, tauri::InvokeError>;

    #[cfg(feature = "codegen")]
    fn schema(_gen: &mut schemars::gen::SchemaGenerator) -> Option<schemars::schema::Schema> {
        None
    }
}

pub trait InvokeReply<R: Runtime> {
    fn reply(self, resolver: InvokeResolver<R>);

    #[cfg(feature = "codegen")]
    fn schema(_gen: &mut schemars::gen::SchemaGenerator) -> Option<schemars::schema::Schema> {
        None
    }
}

pub trait CommandHandler<Args> {
    type Output;
    fn handle(&self, args: Args) -> Self::Output;
}

pub struct Commands<R: Runtime> {
    #[cfg(feature = "codegen")]
    #[doc(hidden)]
    pub schema_gen: schemars::gen::SchemaGenerator,
    commands: HashMap<Cow<'static, str>, Command<R>>,
}

impl<R: Runtime> Commands<R> {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "codegen")]
            schema_gen: schemars::gen::SchemaSettings::default().into_generator(),
            commands: Default::default(),
        }
    }

    pub fn into_invoke_handler(self) -> impl Fn(Invoke<R>) + Send + Sync + 'static {
        move |invoke: Invoke<R>| {
            let cmd_name = invoke.message.command();

            match self.commands.get(cmd_name) {
                Some(c) => (c.handler)(invoke),
                None => invoke
                    .resolver
                    .invoke_error(tauri::InvokeError::from_anyhow(anyhow!(
                        "no handler found for {cmd_name}"
                    ))),
            }
        }
    }

    pub fn command(&mut self, command: impl IntoCommand) -> &mut Self {
        let (name, cmd) = command.into_command(self);
        if self.commands.contains_key(&name) {
            panic!("command handler for command `{name}` already exists");
        }
        self.commands.insert(name, cmd);
        self
    }

    pub fn handler<Args, F>(&mut self, command_name: &str, description: &str, handler: F) -> &mut Self
    where
        Args: InvokeArgs<R>,
        F: CommandHandler<Args> + Send + Sync + 'static,
        F::Output: InvokeReply<R>,
    {
        let mut cmd = self.create_command(handler);

        #[cfg(feature = "codegen")]
        {
            cmd.meta.docs = Cow::Owned(description.to_string());
        }

        if self
            .commands
            .insert(Cow::Owned(command_name.to_string()), cmd)
            .is_some()
        {
            panic!("command handler for command `{command_name}` already exists");
        }

        self
    }

    #[doc(hidden)]
    pub fn create_command<Args, F>(&mut self, handler: F) -> Command<R>
    where
        Args: InvokeArgs<R>,
        F: CommandHandler<Args> + Send + Sync + 'static,
        F::Output: InvokeReply<R>,
    {

        let handler = Box::new(move |invoke| match Args::invoke_args(&invoke) {
            Ok(args) => {
                handler.handle(args).reply(invoke.resolver);
            }
            Err(err) => {
                invoke.resolver.invoke_error(err);
            }
        });

        #[cfg(feature = "codegen")]
        {
            Command {
                handler,
                meta: codegen::CommandMeta {
                    docs: "".into(),
                    args: Args::args(&mut self.schema_gen),
                    output_schema: F::Output::schema(&mut self.schema_gen),
                },
            }
        }

        #[cfg(not(feature = "codegen"))]
        {
            Command {
                handler,
            }
        }
    }
}

impl<R: Runtime> Default for Commands<R> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub trait IntoCommand {
    fn into_command<R: Runtime>(
        self,
        commands: &mut Commands<R>,
    ) -> (Cow<'static, str>, Command<R>);
}
