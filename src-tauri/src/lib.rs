use std::time::Instant;
use image::ImageEncoder;

#[derive(serde::Serialize)]
pub struct EmbedResult {
    pub image_bytes: Vec<u8>,
    pub duration_ms: f64,
}

#[derive(serde::Serialize)]
pub struct ExtractResult {
    pub watermark_text: String,
    pub duration_ms: f64,
}

#[tauri::command]
fn embed_watermark(image_bytes: Vec<u8>, watermark_text: String) -> Result<EmbedResult, String> {
    let start = Instant::now();
    let image = image::load_from_memory(&image_bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?;
    let rgba = image.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    let embedded = dwt_watermark_core::algorithm::dwt_embed_to_rgba(
        rgba.as_raw(), width, height, &watermark_text
    ).ok_or("水印嵌入失败")?;

    let mut out = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut out,
        image::codecs::png::CompressionType::Default,
        image::codecs::png::FilterType::NoFilter,
    );
    encoder.write_image(&embedded, width as u32, height as u32, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    Ok(EmbedResult {
        image_bytes: out,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

#[tauri::command]
fn extract_watermark(image_bytes: Vec<u8>) -> Result<ExtractResult, String> {
    let start = Instant::now();
    let image = image::load_from_memory(&image_bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?;
    let rgba = image.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    let text = dwt_watermark_core::algorithm::dwt_extract_from_rgba(
        rgba.as_raw(), width, height
    ).unwrap_or_else(|| "未检测到水印".to_string());

    Ok(ExtractResult {
        watermark_text: text,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

#[tauri::command]
fn save_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, data).map_err(|e| format!("保存文件失败: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![embed_watermark, extract_watermark, save_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
