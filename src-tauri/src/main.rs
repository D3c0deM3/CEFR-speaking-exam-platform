#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_attempt,
            get_random_questions,
            save_response,
            finish_attempt,
            get_attempts,
            save_audio_file,
            get_audio_file,
            add_question,
            get_questions,
            delete_question,
            rate_response,
            save_image_file,
            get_image_file,
            get_recordings,
            get_response_audio,
            delete_response,
            delete_attempt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
