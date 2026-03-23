// #![windows_subsystem = "windows"]

mod gui;

use std::path::PathBuf;
use dwt_watermark::algorithm::dwt_extract_from_rgba;
use chrono::{Local, TimeZone};
use serde_json::Value;
use winio::prelude::*;
use windows_core::HSTRING;

use gui::ui_helpers::*;
use gui::window_hooks::enable_paste_and_drag_hooks;

fn init_logger() {
    use std::io::Write;
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("dwt_watermark", log::LevelFilter::Debug) // 强制开启库的日志
        .format(|buf, record| {
            writeln!(buf, "[{}] {}", record.level(), record.args())
        })
        .try_init();
}

type Result<T> = std::result::Result<T, winio::Error>;

fn main() -> Result<()> {
    // 强制分配一个控制台窗口
    unsafe {
        let _ = windows::Win32::System::Console::AllocConsole();
    }
    
    init_logger();
    log::info!("启动 DWT 水印解密工具，已开启控制台日志...");
    App::new("com.dwtwatermark.winui3")?.run_until_event::<MainModel>(())
}

struct MainModel {
    window: Child<Window>,
    upload_button: Child<Button>,
    remove_button: Child<Button>,
    reparse_button: Child<Button>,
    copy_button: Child<Button>,
    image_info_label: Child<Label>,
    result_title: Child<Label>,
    result_text: Child<TextBox>,
    current_image_path: Option<PathBuf>,
    current_result: String,
}

#[derive(Debug)]
enum Message {
    Noop,
    Close,
    Redraw,
    UploadImage,
    RemoveImage,
    ReparseImage,
    CopyResult,
    ImportPath(PathBuf),
    DoExtract(PathBuf),
    UpdateImageInfo(String),
    ExtractComplete(Option<(String, String)>, String),
}

impl Component for MainModel {
    type Error = winio::Error;
    type Event = ();
    type Init<'a> = ();
    type Message = Message;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "DWT 盲水印解密工具",
                size: Size::new(900.0, 600.0),
            },
            upload_button: Button = (&window) => {
                text: "点击上传图片",
            },
            remove_button: Button = (&window) => {
                text: "删除图片",
            },
            reparse_button: Button = (&window) => {
                text: "重新解析",
            },
            copy_button: Button = (&window) => {
                text: "一键复制",
            },
            image_info_label: Label = (&window) => {
                text: "图片信息：暂无",
            },
            result_title: Label = (&window) => {
                text: "水印详情",
            },
            result_text: TextBox = (&window),
        }

        window.set_backdrop(Backdrop::Mica)?;
        window.show()?;
        enable_paste_and_drag_hooks(&window, _sender.clone())?;

        let _ = apply_glyph_icon_to_button(&upload_button, "E8B5", "点击选择或上传图片");
        let _ = apply_compact_glyph_button(&remove_button, "E74D", "删除图片");
        let _ = apply_compact_glyph_button(&reparse_button, "E72C", "重新解析");
        let _ = apply_compact_glyph_button(&copy_button, "E8C8", "一键复制");
        let _ = apply_label_font_size(&result_title, 18.0);
        let _ = apply_textbox_font_size(&result_text, 13.0);

        Ok(Self {
            window,
            upload_button,
            remove_button,
            reparse_button,
            copy_button,
            image_info_label,
            result_title,
            result_text,
            current_image_path: None,
            current_result: String::new(),
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: Message::Noop,
            self.window => {
                WindowEvent::Close => Message::Close,
                WindowEvent::Resize | WindowEvent::ThemeChanged => Message::Redraw,
            },
            self.upload_button => {
                ButtonEvent::Click => Message::UploadImage,
            },
            self.remove_button => {
                ButtonEvent::Click => Message::RemoveImage,
            },
            self.reparse_button => {
                ButtonEvent::Click => Message::ReparseImage,
            },
            self.copy_button => {
                ButtonEvent::Click => Message::CopyResult,
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.upload_button,
            self.remove_button,
            self.reparse_button,
            self.copy_button,
            self.image_info_label,
            self.result_title,
            self.result_text
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            Message::Noop => Ok(false),
            Message::Close => {
                sender.output(());
                Ok(false)
            }
            Message::Redraw => Ok(true),
            Message::UploadImage => {
                if let Some(path) =
                    rfd::FileDialog::new().add_filter("图片文件", &["png", "jpg", "jpeg", "webp"]).pick_file()
                {
                    self.current_image_path = Some(path.clone());
                    if let Some(path_str) = path.to_str() {
                        let _ = apply_image_to_button(&self.upload_button, path_str);
                        let _ = self.result_text.set_text("正在解密，请稍候...");
                        let _ = self.image_info_label.set_text("图片信息：正在读取...");
                    }
                    sender.post(Message::DoExtract(path));
                }
                Ok(true)
            }
            Message::RemoveImage => {
                self.current_image_path = None;
                self.current_result = String::new();
                let _ = apply_glyph_icon_to_button(&self.upload_button, "E8B5", "点击选择或上传图片");
                let _ = self.result_text.set_text("图片已移除，请重新上传");
                let _ = self.image_info_label.set_text("图片信息：暂无");
                Ok(true)
            }
            Message::ReparseImage => {
                if let Some(path) = self.current_image_path.clone() {
                    let _ = self.result_text.set_text("正在重新解密，请稍候...");
                    let _ = self.image_info_label.set_text("图片信息：正在读取...");
                    sender.post(Message::DoExtract(path));
                } else {
                    let _ = self.result_text.set_text("暂无可重新解析的图片，请先上传");
                }
                Ok(true)
            }
            Message::CopyResult => {
                let info_text = self.image_info_label.text().unwrap_or_default();
                let result_text = self.current_result.clone();
                let combined = format!("{}\n\n{}", info_text, result_text);
                let hwnd = windows::Win32::Foundation::HWND(self.window.as_window().handle().unwrap_or(std::ptr::null_mut()));
                let _ = set_clipboard_text(hwnd, &combined);
                Ok(true)
            }
            Message::ImportPath(path) => {
                self.current_image_path = Some(path.clone());
                if let Some(path_str) = path.to_str() {
                    let _ = apply_image_to_button(&self.upload_button, path_str);
                    let _ = self.result_text.set_text("正在解密，请稍候...");
                    let _ = self.image_info_label.set_text("图片信息：正在读取...");
                }
                sender.post(Message::DoExtract(path));
                Ok(true)
            }
            Message::DoExtract(path) => {
                let sender = sender.clone();
                std::thread::spawn(move || {
                    let start_time = std::time::Instant::now();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    
                    let mut info_str_saved = String::new();
                    
                    // 先快速获取图片基础信息并通知前台
                    if let Some(path_str) = path.to_str() {
                        let metadata = std::fs::metadata(&path).ok();
                        let file_size = metadata.map(|m| m.len()).unwrap_or(0);
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("未知").to_uppercase();
                        let size_mb = file_size as f64 / 1024.0 / 1024.0;
                        let size_str = if size_mb >= 1.0 {
                            format!("{:.2} MB", size_mb)
                        } else {
                            format!("{:.2} KB", file_size as f64 / 1024.0)
                        };

                        let dimensions = rt.block_on(async {
                            use windows::Storage::{StorageFile, FileAccessMode};
                            use windows::Graphics::Imaging::BitmapDecoder;
                            let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path_str)).ok()?.await.ok()?;
                            let stream = file.OpenAsync(FileAccessMode::Read).ok()?.await.ok()?;
                            let decoder = BitmapDecoder::CreateAsync(&stream).ok()?.await.ok()?;
                            Some((decoder.PixelWidth().ok()?, decoder.PixelHeight().ok()?))
                        });

                        if let Some((w, h)) = dimensions {
                            info_str_saved = format!("图片信息：宽 {}px, 高 {}px, 大小 {}, 格式 {}", w, h, size_str, ext);
                            sender.post(Message::UpdateImageInfo(info_str_saved.clone()));
                        }
                    }

                    // 然后再进行极其耗时的像素解码和 DWT 解析
                    let extracted_info = rt.block_on(async move {
                        if let Some(path_str) = path.to_str() {
                            if let Some((bytes, width, height)) = decode_image(path_str).await {
                                let result = if let Some(text) = dwt_extract_from_rgba(&bytes, width, height) {
                                    render_watermark_text(&text)
                                } else {
                                    "未检测到水印".to_string()
                                };
                                return Some((info_str_saved, result));
                            }
                        }
                        None
                    });
                    
                    let elapsed = start_time.elapsed();
                    let elapsed_secs = elapsed.as_secs_f64();
                    let time_str = if elapsed_secs >= 60.0 {
                        let mins = (elapsed_secs / 60.0).floor() as u32;
                        let secs = (elapsed_secs % 60.0).floor() as u32;
                        format!("耗时：{}分{}秒", mins, secs)
                    } else if elapsed_secs >= 1.0 {
                        format!("耗时：{:.2}秒", elapsed_secs)
                    } else {
                        format!("耗时：{}ms", elapsed.as_millis())
                    };

                    sender.post(Message::ExtractComplete(extracted_info, time_str));
                });
                Ok(true)
            }
            Message::UpdateImageInfo(info) => {
                let _ = self.image_info_label.set_text(&info);
                Ok(true)
            }
            Message::ExtractComplete(info, time_str) => {
                if let Some((info_str, result_str)) = info {
                    let full_info = format!("{} | {}", info_str, time_str);
                    let _ = self.image_info_label.set_text(&full_info);
                    self.current_result = result_str.clone();
                    let _ = self.result_text.set_text(&result_str);
                } else {
                    let _ = self.image_info_label.set_text(&format!("图片信息：读取或解析失败 | {}", time_str));
                    self.current_result = "解析图片失败".to_string();
                    let _ = self.result_text.set_text("解析图片失败");
                }
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let client = self.window.client_size()?;
        let width = client.width;
        let height = client.height;
        let margin = 32.0; // 增大边距，Apple 风格喜欢大留白
        
        let title_height = 40.0;
        
        let content_y = margin + title_height + 8.0;
        let content_height = height - content_y - margin;
        
        // 调整比例：左侧图片区占 50%，右侧占 50%，留出呼吸感
        let left_width = (width - margin * 3.0) * 0.48;
        let right_width = (width - margin * 3.0) * 0.52;
        let right_x = margin * 2.0 + left_width;
        
        let action_height = 42.0; // 按钮稍微加高
        let result_title_height = 36.0;
        let panel_spacing = 16.0; // 增加间距
        
        self.upload_button.set_loc(Point::new(margin, content_y))?;
        self.upload_button.set_size(Size::new(left_width, content_height))?;
        
        let action_y = content_y;
        let action_gap = 12.0;
        let action_width = (right_width - action_gap * 2.0) / 3.0;
        
        self.remove_button.set_loc(Point::new(right_x, action_y))?;
        self.remove_button.set_size(Size::new(action_width, action_height))?;
        self.reparse_button.set_loc(Point::new(right_x + action_width + action_gap, action_y))?;
        self.reparse_button.set_size(Size::new(action_width, action_height))?;
        self.copy_button.set_loc(Point::new(right_x + (action_width + action_gap) * 2.0, action_y))?;
        self.copy_button.set_size(Size::new(action_width, action_height))?;

        let info_y = action_y + action_height + panel_spacing;
        let info_height = 48.0; // 给图片信息留两行的空间，防止换行后被裁
        self.image_info_label.set_loc(Point::new(right_x, info_y))?;
        self.image_info_label.set_size(Size::new(right_width, info_height))?;
        
        let result_title_y = info_y + info_height + 4.0;
        self.result_title.set_loc(Point::new(right_x, result_title_y))?;
        self.result_title.set_size(Size::new(right_width, result_title_height))?;
        
        let text_y = result_title_y + result_title_height + 4.0;
        self.result_text.set_loc(Point::new(right_x, text_y))?;
        self.result_text.set_size(Size::new(right_width, content_y + content_height - text_y))?;

        Ok(())
    }
}

// --------------------------------------------------
// 以下是核心数据解析和渲染逻辑
// --------------------------------------------------

async fn decode_image(path: &str) -> Option<(Vec<u8>, usize, usize)> {
    use windows::Storage::{StorageFile, FileAccessMode};
    use windows::Graphics::Imaging::{BitmapDecoder, BitmapPixelFormat, BitmapAlphaMode};
    use windows::Storage::Streams::{Buffer, DataReader};

    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path)).ok()?.await.ok()?;
    let stream = file.OpenAsync(FileAccessMode::Read).ok()?.await.ok()?;
    let decoder = BitmapDecoder::CreateAsync(&stream).ok()?.await.ok()?;
    
    // Request RGBA8 to keep the same layout as wasm-side encoding/embedding
    let bitmap = decoder.GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Rgba8, BitmapAlphaMode::Straight).ok()?.await.ok()?;
    
    let width = bitmap.PixelWidth().ok()? as usize;
    let height = bitmap.PixelHeight().ok()? as usize;
    let capacity = (width * height * 4) as u32;
    
    let buffer = Buffer::Create(capacity).ok()?;
    bitmap.CopyToBuffer(&buffer).ok()?;
    
    let reader = DataReader::FromBuffer(&buffer).ok()?;
    let mut bytes = vec![0u8; capacity as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    
    Some((bytes, width, height))
}

fn render_watermark_text(extracted_text: &str) -> String {
    let json_start = extracted_text.find('{');
    let json_end = extracted_text.rfind('}');
    let Some(start) = json_start else {
        return extracted_text.to_string();
    };
    let Some(end) = json_end else {
        return extracted_text.to_string();
    };
    if end < start {
        return extracted_text.to_string();
    }
    let json_text = &extracted_text[start..=end];
    let Ok(value) = serde_json::from_str::<Value>(json_text) else {
        return extracted_text.to_string();
    };
    let timestamp = value.get("a").and_then(Value::as_i64).unwrap_or(0);
    let formatted_timestamp = format_timestamp(timestamp);
    let plugin_version = value.get("b").and_then(Value::as_str).unwrap_or("未知");
    let account = value
        .get("c")
        .and_then(|it| it.as_str().map(ToOwned::to_owned).or_else(|| it.as_i64().map(|num| num.to_string())))
        .unwrap_or_else(|| "未知".to_string());
    
    format!(
        "【原始水印】\n{}\n\n【解析结果】\n  时间戳：{}\n  详细信息：{}\n  账号：{}",
        extracted_text,
        formatted_timestamp,
        plugin_version,
        account
    )
}

fn format_timestamp(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "未知".to_string();
    }
    match Local.timestamp_millis_opt(timestamp_ms).single() {
        Some(value) => value.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "未知".to_string()
    }
}
