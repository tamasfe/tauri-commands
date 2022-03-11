use std::{borrow::Cow, collections::HashMap};
use tauri::{Invoke, InvokeResolver, Runtime};

mod impls;

#[cfg(feature = "codegen")]
pub mod codegen;

pub use anyhow;
pub use tauri_commands_macros::command;

pub type CommandResult<T> = Result<T, anyhow::Error>;

pub struct Command<R: Runtime> {
    handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
    #[cfg(feature = "codegen")]
    pub meta: codegen::CommandMeta,
}

pub trait InvokeArgs<R: Runtime> {
    fn invoke_args(invoke: &Invoke<R>) -> Self;

    #[cfg(feature = "codegen")]
    #[doc(hidden)]
    fn args(_gen: &mut schemars::gen::SchemaGenerator) -> Vec<codegen::CommandArg> {
        Vec::default()
    }
}

trait FromInvoke<R: Runtime> {
    fn from_invoke(arg_name: &str, invoke: &Invoke<R>) -> Self;

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
                None => panic!("no handler found for {cmd_name}"),
            }
        }
    }

    pub fn add_command(mut self, command: impl IntoCommand) -> Self {
        let (name, cmd) = command.into_command(&mut self);
        if self.commands.contains_key(&name) {
            panic!("command handler for command `{name}` already exists");
        }
        self.commands.insert(name, cmd);
        self
    }

    pub fn handle<Args, F>(mut self, command_name: &str, handler: F) -> Self
    where
        Args: InvokeArgs<R>,
        F: CommandHandler<Args> + Send + Sync + 'static,
        F::Output: InvokeReply<R>,
    {
        let cmd = self.create_command(handler);
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
        #[cfg(feature = "codegen")]
        {
            Command {
                handler: Box::new(move |invoke| {
                    handler
                        .handle(Args::invoke_args(&invoke))
                        .reply(invoke.resolver);
                }),
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
                handler: Box::new(move |invoke| {
                    handler
                        .handle(Args::invoke_args(&invoke))
                        .reply(invoke.resolver);
                }),
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
