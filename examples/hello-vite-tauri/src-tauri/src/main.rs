#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};
use tauri_commands::{command, CommandResult, Commands, TauriWindow};

/// The request data.
#[derive(Deserialize, JsonSchema)]
struct HelloRequest {
    /// This message is printed to stdout.
    message: String,
}

/// A reply for hello.
#[derive(Serialize, JsonSchema)]
struct HelloReply {
    /// The message to be written to the console.
    message: String,
}

/// Send a friendly message and receive a reply.
#[command]
async fn hello(request: HelloRequest) -> CommandResult<HelloReply> {
    println!("{}", request.message);
    Ok(HelloReply {
        message: "hello from tauri!".into(),
    })
}

/// Commands defined as functions have to be generic over the runtime.
#[command]
async fn show_window<R: Runtime>(window: TauriWindow<R>) -> CommandResult<()> {
    window.show().unwrap();
    Ok(())
}

fn main() {
    let mut commands = Commands::new();

    commands.command(hello).command(show_window).handler(
        "add numbers",
        "adds numbers",
        |a: i32, b: i32| async move { Ok(a + b) },
    );

    if cfg!(debug_assertions) {
        commands
            .write_typescript(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/tauri.ts"))
            .unwrap();
    }

    tauri::Builder::default()
        .invoke_handler(commands.into_invoke_handler())
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
