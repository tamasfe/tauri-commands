#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_commands::{command, CommandResult, Commands};

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

#[command]
async fn hello(request: HelloRequest) -> CommandResult<HelloReply> {
    println!("{}", request.message);
    Ok(HelloReply {
        message: "hello from tauri!".into(),
    })
}

fn main() {
    let mut commands = Commands::new();
    commands.add_command(hello);

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
