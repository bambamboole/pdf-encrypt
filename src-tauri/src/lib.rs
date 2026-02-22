use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use lopdf::encryption::crypt_filters::{Aes256CryptFilter, CryptFilter};
use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use lopdf::Document;
use rand::Rng;

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
            match encrypt_pdf_file(&file, output.to_string_lossy().as_ref(), &password) {
                Ok(_) => EncryptResult {
                    file: file.clone(),
                    success: true,
                    message: format!("Saved as {}", output.display()),
                },
                Err(e) => EncryptResult {
                    file: file.clone(),
                    success: false,
                    message: format!("Error: {}", e),
                },
            }
        })
        .collect()
}

fn encrypt_pdf_file(input: &str, output: &str, password: &str) -> Result<(), String> {
    let mut doc = Document::load(input).map_err(|e| format!("Failed to load PDF: {}", e))?;

    let crypt_filter: Arc<dyn CryptFilter> = Arc::new(Aes256CryptFilter);

    let mut file_encryption_key = [0u8; 32];
    let mut rng = rand::rng();
    rng.fill(&mut file_encryption_key);

    let version = EncryptionVersion::V5 {
        encrypt_metadata: true,
        crypt_filters: BTreeMap::from([(b"StdCF".to_vec(), crypt_filter)]),
        file_encryption_key: &file_encryption_key,
        stream_filter: b"StdCF".to_vec(),
        string_filter: b"StdCF".to_vec(),
        owner_password: password,
        user_password: password,
        permissions: Permissions::all(),
    };

    let state = EncryptionState::try_from(version)
        .map_err(|e| format!("Failed to create encryption state: {}", e))?;

    doc.encrypt(&state)
        .map_err(|e| format!("Failed to encrypt: {}", e))?;

    doc.save(output)
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(())
}

/// Generate the output path for an encrypted PDF.
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
        .invoke_handler(tauri::generate_handler![encrypt_pdfs])
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
        assert_eq!(result, PathBuf::from("test_encrypted.pdf"));
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
    fn test_encrypt_nonexistent_file() {
        let results = encrypt_pdfs(vec!["/nonexistent/path/fake.pdf".into()], "pass".into());
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
    }

    #[test]
    fn test_encrypt_real_pdf() {
        use lopdf::content::{Content, Operation};
        use lopdf::{dictionary, Object, Stream};
        use std::fs;

        let dir = std::env::temp_dir().join("pdf_encrypt_test_rust");
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("test.pdf");

        // Create a minimal valid PDF using lopdf
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![100.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal("Test PDF")]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.save(&input).unwrap();

        // Now encrypt it
        let results = encrypt_pdfs(
            vec![input.to_string_lossy().into()],
            "testpass".into(),
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].success, "Failed: {}", results[0].message);
        assert!(dir.join("test_encrypted.pdf").exists());

        // Verify the encrypted file can be loaded and is encrypted
        let encrypted_doc = Document::load(dir.join("test_encrypted.pdf")).unwrap();
        assert!(encrypted_doc.is_encrypted());

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }
}
