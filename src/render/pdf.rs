use image::{DynamicImage, RgbaImage};
use pdfium_render::prelude::*;

use crate::{CasterError, Result};

pub struct PdfRenderer {
    pdfium: Pdfium,
}

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        // Initialize Pdfium
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .map_err(|e| CasterError::Render(format!("Failed to initialize Pdfium: {}", e)))?,
        );

        Ok(Self { pdfium })
    }

    pub fn render_page(&mut self, pdf_data: &[u8], page_num: u32) -> Result<DynamicImage> {
        // Validate page number to prevent overflow when converting to u16
        if page_num == 0 {
            return Err(CasterError::Render("Page number must be at least 1".into()));
        }
        if page_num > u16::MAX as u32 {
            return Err(CasterError::Render(format!(
                "Page number {} exceeds maximum supported page number {}",
                page_num,
                u16::MAX
            )));
        }

        // Load PDF document
        let document = self
            .pdfium
            .load_pdf_from_byte_slice(pdf_data, None)
            .map_err(|e| CasterError::Render(format!("Failed to load PDF: {}", e)))?;

        // Get the page (convert 1-based to 0-based)
        let page_index = (page_num - 1) as u16;
        let page = document
            .pages()
            .get(page_index)
            .map_err(|e| CasterError::Render(format!("Failed to get page {}: {}", page_num, e)))?;

        // Render at 2x resolution for better quality
        let scale = 2.0;
        let width = (page.width().value * scale) as u32;
        let height = (page.height().value * scale) as u32;

        // Render page to bitmap
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(width as i32)
                    .set_target_height(height as i32)
                    .rotate_if_landscape(PdfPageRenderRotation::None, false),
            )
            .map_err(|e| CasterError::Render(format!("Failed to render page: {}", e)))?;

        // Convert to image
        let buffer = bitmap.as_raw_bytes();
        let rgba_image = RgbaImage::from_raw(width, height, buffer.to_vec())
            .ok_or_else(|| CasterError::Render("Failed to create image from bitmap".into()))?;

        Ok(DynamicImage::ImageRgba8(rgba_image))
    }

    pub fn get_page_count(&self, pdf_data: &[u8]) -> Result<u32> {
        let document = self
            .pdfium
            .load_pdf_from_byte_slice(pdf_data, None)
            .map_err(|e| CasterError::Render(format!("Failed to load PDF: {}", e)))?;

        Ok(document.pages().len() as u32)
    }
}
