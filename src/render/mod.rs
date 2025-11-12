use comrak::{markdown_to_html, Options};
use image::DynamicImage;

use crate::{Result, CasterError};

pub mod pdf;
pub mod audio;
pub mod wasm;
pub mod mirror;

pub use pdf::PdfRenderer;
pub use audio::AudioRenderer;
pub use wasm::WasmRunner;
pub use mirror::ScreenMirror;

pub struct RenderEngine {
    pdf_renderer: Option<PdfRenderer>,
    pdf_init_error: Option<String>,
    audio_renderer: AudioRenderer,
    wasm_runner: WasmRunner,
    screen_mirror: Option<ScreenMirror>,
}

impl RenderEngine {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            pdf_renderer: None,
            pdf_init_error: None,
            audio_renderer: AudioRenderer::new(),
            wasm_runner: WasmRunner::new()?,
            screen_mirror: None,
        })
    }

    pub fn render_markdown(&self, markdown: &str, theme: Option<&str>) -> Result<String> {
        let mut options = Options::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.render.unsafe_ = true;

        let html = markdown_to_html(markdown, &options);

        // Wrap with theme CSS
        let theme_css = match theme {
            Some("dark") => include_str!("themes/dark.css"),
            Some("light") => include_str!("themes/light.css"),
            _ => include_str!("themes/dark.css"),
        };

        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>{}</style>
</head>
<body>
    <div class="markdown-body">
        {}
    </div>
</body>
</html>"#,
            theme_css, html
        ))
    }

    pub fn render_pdf(&mut self, data: &[u8], page: u32) -> Result<DynamicImage> {
        // If we previously failed to initialize, return that error
        if let Some(ref error) = self.pdf_init_error {
            return Err(CasterError::Render(format!("PDF renderer initialization failed: {}", error)));
        }
        
        // Try to initialize PDF renderer if not already done
        if self.pdf_renderer.is_none() {
            match PdfRenderer::new() {
                Ok(renderer) => {
                    self.pdf_renderer = Some(renderer);
                }
                Err(e) => {
                    // Store the error to avoid repeated initialization attempts
                    let error_msg = e.to_string();
                    self.pdf_init_error = Some(error_msg.clone());
                    return Err(CasterError::Render(format!("Failed to initialize PDF renderer: {}", error_msg)));
                }
            }
        }

        if let Some(ref mut renderer) = self.pdf_renderer {
            renderer.render_page(data, page)
        } else {
            Err(CasterError::Render("PDF renderer not initialized".into()))
        }
    }

    pub fn render_audio_waveform(&self, samples: &[f32], width: u32, height: u32) -> Result<DynamicImage> {
        self.audio_renderer.render_waveform(samples, width, height)
    }

    pub async fn run_wasm(&mut self, wasm_bytes: &[u8], entry_point: Option<&str>) -> Result<Vec<u8>> {
        self.wasm_runner.run(wasm_bytes, entry_point).await
    }

    pub async fn start_screen_mirror(&mut self, display_id: Option<String>) -> Result<()> {
        self.screen_mirror = Some(ScreenMirror::new(display_id)?);
        Ok(())
    }

    pub fn capture_screen_frame(&mut self) -> Result<Option<DynamicImage>> {
        if let Some(ref mut mirror) = self.screen_mirror {
            mirror.capture_frame()
        } else {
            Ok(None)
        }
    }

    pub async fn render_3d_model(&mut self, _model_path: &str) -> Result<()> {
        // TODO: Implement 3D model rendering with Bevy
        Err(CasterError::Render("3D rendering not yet implemented".into()))
    }
}