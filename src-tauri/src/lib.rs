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
            let output = encrypted_output_path(&file);

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

/// Generate the output path for an encrypted PDF.
/// Given an input path like `/foo/bar.pdf`, returns `/foo/bar_encrypted.pdf`.
pub fn encrypted_output_path(input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    parent.join(format!("{}_encrypted.pdf", stem))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![encrypt_pdfs, check_qpdf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_path_basic() {
        let result = encrypted_output_path("/tmp/report.pdf");
        assert_eq!(result, PathBuf::from("/tmp/report_encrypted.pdf"));
    }

    #[test]
    fn test_output_path_no_extension() {
        let result = encrypted_output_path("/tmp/report");
        assert_eq!(result, PathBuf::from("/tmp/report_encrypted.pdf"));
    }

    #[test]
    fn test_output_path_nested() {
        let result = encrypted_output_path("/home/user/docs/my file.pdf");
        assert_eq!(
            result,
            PathBuf::from("/home/user/docs/my file_encrypted.pdf")
        );
    }

    #[test]
    fn test_output_path_relative() {
        let result = encrypted_output_path("test.pdf");
        assert_eq!(result, PathBuf::from("./test_encrypted.pdf"));
    }

    #[test]
    fn test_encrypt_result_serialization() {
        let r = EncryptResult {
            file: "test.pdf".into(),
            success: true,
            message: "ok".into(),
        };
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["file"], "test.pdf");
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "ok");
    }

    #[test]
    fn test_check_qpdf_returns_bool() {
        // Just verify it doesn't panic and returns a bool
        let _result: bool = check_qpdf();
    }

    #[test]
    fn test_encrypt_nonexistent_file() {
        let results = encrypt_pdfs(
            vec!["/nonexistent/path/fake.pdf".into()],
            "pass".into(),
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
    }

    #[test]
    #[ignore] // Requires qpdf installed
    fn test_encrypt_real_pdf() {
        use std::fs;
        // Create a minimal valid PDF
        let dir = std::env::temp_dir().join("pdf_encrypt_test");
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("test.pdf");
        // Minimal valid PDF
        fs::write(
            &input,
            b"%PDF-1.0\n1 0 obj<</Pages 2 0 R/Type/Catalog>>endobj\n2 0 obj<</Count 1/Kids[3 0 R]/Type/Pages>>endobj\n3 0 obj<</MediaBox[0 0 612 792]/Parent 2 0 R/Type/Page>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer<</Root 1 0 R/Size 4>>\nstartxref\n190\n%%EOF",
        )
        .unwrap();

        let results = encrypt_pdfs(
            vec![input.to_string_lossy().into()],
            "testpass".into(),
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].success, "Failed: {}", results[0].message);
        assert!(dir.join("test_encrypted.pdf").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
