use crate::component_manager::{Component, ComponentManager};
use crate::{ExtractContext, ExtractError, ExtractedContent, Extractor, enforce_limits_str};
use anyhow::Result;
use core_types::DocKey;
use std::path::Path;
use std::process::Command;
use tracing::warn;

pub struct OcrExtractor {
    manager: ComponentManager,
    tesseract_component: Component,
}

impl OcrExtractor {
    pub fn new(manager: ComponentManager) -> Self {
        // Define the standard Tesseract component we expect
        // In a real app, this might come from config or a remote manifest
        let tesseract_component = Component {
            id: "tesseract".to_string(),
            version: "5.3.3".to_string(),
            // Placeholder URL - in production this would be a real release asset
            url: "https://github.com/UB-Mannheim/tesseract/releases/download/v5.3.3/tesseract-ocr-w64-setup-v5.3.3.20231005.exe".to_string(), 
            // Placeholder hash
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            executable_name: if cfg!(windows) { "tesseract.exe" } else { "tesseract" }.to_string(),
        };

        Self {
            manager,
            tesseract_component,
        }
    }

    fn get_tesseract_path(&self) -> Option<std::path::PathBuf> {
        // Check component manager first
        if let Some(path) = self.manager.get_executable_path(&self.tesseract_component) {
            return Some(path);
        }

        // Fallback to system path
        // This simple check assumes 'tesseract' is in PATH
        if which::which("tesseract").is_ok() {
            return Some("tesseract".into());
        }

        None
    }
}

impl Extractor for OcrExtractor {
    fn name(&self) -> &'static str {
        "ocr-tesseract"
    }

    fn supports(&self, ctx: &ExtractContext) -> bool {
        // Check if extension is an image
        if let Some(ext) = super::resolve_ext(ctx) {
            match ext.as_str() {
                "png" | "jpg" | "jpeg" | "tiff" | "bmp" | "webp" => true,
                _ => false, // PDF OCR handled by Extractous usually, or specialized PDF pipeline
            }
        } else {
            false
        }
    }

    fn extract(&self, ctx: &ExtractContext, key: DocKey) -> Result<ExtractedContent, ExtractError> {
        let tesseract_bin = self
            .get_tesseract_path()
            .ok_or_else(|| ExtractError::Failed("tesseract binary not found".into()))?;

        let input_path = Path::new(ctx.path);

        // Run tesseract: tesseract <image> stdout
        let output = Command::new(tesseract_bin)
            .arg(input_path)
            .arg("stdout") // Write to stdout
            .arg("-l")
            .arg("eng") // Default to English for now, config later
            .output()
            .map_err(|e| ExtractError::Failed(format!("failed to spawn tesseract: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("tesseract failed for {:?}: {}", input_path, stderr);
            return Err(ExtractError::Failed("tesseract exited with error".into()));
        }

        let text_raw = String::from_utf8_lossy(&output.stdout);
        let (text, truncated, used_bytes) = enforce_limits_str(&text_raw, ctx);

        Ok(ExtractedContent {
            key,
            text,
            lang: Some("eng".into()), // Assumption
            truncated,
            content_lang: None,
            bytes_processed: used_bytes,
        })
    }
}
