use comrak::{markdown_to_html, Options};

use crate::{Result, CasterError};

pub struct RenderEngine {
    // Placeholder for now
}

impl RenderEngine {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
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
    
    pub async fn render_3d_model(&mut self, _model_path: &str) -> Result<()> {
        // TODO: Implement 3D model rendering with Bevy
        Err(CasterError::Render("3D rendering not yet implemented".into()))
    }
}