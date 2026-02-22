use std::path::PathBuf;
use std::process::Command;
use tauri;

#[derive(serde::Serialize)]
struct EncryptResult {
    file: String,
    success: bool,
    message: String,
}

#[tauri::command]
fn encrypt_pdfs(files: Vec<String>, password: String) -> Vec<EncryptResult> {
    files
        .into_iter()
        .map(|file| {
            let path = PathBuf::from(&file);
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let parent = path.parent().unwrap_or(std::path::Path::new("."));
            let output = parent.join(format!("{}_encrypted.pdf", stem));

            let result = Command::new("qpdf")
                .arg("--encrypt")
                .arg(&password)
                .arg(&password)
                .arg("256")
                .arg("--")
                .arg(&file)
                .arg(output.to_string_lossy().as_ref())
                .output();

            match result {
                Ok(output_result) if output_result.status.success() => EncryptResult {
                    file: file.clone(),
                    success: true,
                    message: format!("Saved as {}", output.display()),
                },
                Ok(output_result) => EncryptResult {
                    file: file.clone(),
                    success: false,
                    message: String::from_utf8_lossy(&output_result.stderr).to_string(),
                },
                Err(e) => EncryptResult {
                    file: file.clone(),
                    success: false,
                    message: if e.kind() == std::io::ErrorKind::NotFound {
                        "qpdf not found. Please install it: https://github.com/qpdf/qpdf".into()
                    } else {
                        format!("Error: {}", e)
                    },
                },
            }
        })
        .collect()
}

#[tauri::command]
fn check_qpdf() -> bool {
    Command::new("qpdf").arg("--version").output().is_ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![encrypt_pdfs, check_qpdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
