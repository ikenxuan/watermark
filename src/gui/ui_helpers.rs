use winio::prelude::*;
use windows_core::HSTRING;
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, Markup::XamlReader, TextWrapping};
use windows::Win32::Foundation::{HWND, HANDLE};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use std::ptr::copy_nonoverlapping;

pub fn set_clipboard_text(hwnd: HWND, text: &str) -> windows_core::Result<()> {
    unsafe {
        if OpenClipboard(Some(hwnd)).is_err() { return Ok(()); }
        let _ = EmptyClipboard();
        
        let mut utf16: Vec<u16> = text.encode_utf16().collect();
        utf16.push(0); // null terminator
        let size = utf16.len() * 2;
        
        let hglobal = GlobalAlloc(GMEM_MOVEABLE, size)?;
        let ptr = GlobalLock(hglobal) as *mut u16;
        if !ptr.is_null() {
            copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
            let _ = GlobalUnlock(hglobal);
            let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(HANDLE(hglobal.0 as _)));
        }
        let _ = CloseClipboard();
    }
    Ok(())
}

pub fn apply_compact_glyph_button(button: &Button, glyph: &str, text: &str) -> windows_core::Result<()> {
    use windows_core::Interface;
    let xaml = format!(
        "<StackPanel xmlns='http://schemas.microsoft.com/winfx/2006/xaml/presentation' Orientation='Horizontal' Spacing='8' HorizontalAlignment='Center'>
            <FontIcon Glyph='&#x{};' FontSize='14' Margin='0,2,0,0' />
            <TextBlock Text='{}' FontSize='14' VerticalAlignment='Center' />
        </StackPanel>",
        glyph, text
    );
    let content = XamlReader::Load(&HSTRING::from(xaml))?;
    let native_button = button.as_widget().as_winui().cast::<MUXC::Button>()?;
    native_button.SetContent(&content)?;
    Ok(())
}

pub fn apply_glyph_icon_to_button(button: &Button, glyph: &str, text: &str) -> windows_core::Result<()> {
    use windows_core::Interface;
    let xaml = format!(
        "<StackPanel xmlns='http://schemas.microsoft.com/winfx/2006/xaml/presentation' Orientation='Vertical' Spacing='12' HorizontalAlignment='Center' VerticalAlignment='Center'>
            <FontIcon Glyph='&#x{};' FontSize='48' Foreground='#808080' />
            <TextBlock Text='{}' FontSize='16' Foreground='#606060' HorizontalAlignment='Center' />
        </StackPanel>",
        glyph, text
    );
    let content = XamlReader::Load(&HSTRING::from(xaml))?;
    let native_button = button.as_widget().as_winui().cast::<MUXC::Button>()?;
    native_button.SetContent(&content)?;
    Ok(())
}

pub fn apply_label_font_size(label: &Label, size: f64) -> windows_core::Result<()> {
    use windows_core::Interface;
    let native_label = label.as_widget().as_winui().cast::<MUXC::TextBlock>()?;
    native_label.SetFontSize(size)?;
    native_label.SetTextWrapping(TextWrapping::Wrap)?;
    Ok(())
}

pub fn apply_textbox_font_size(text_box: &TextBox, size: f64) -> windows_core::Result<()> {
    use windows_core::Interface;
    let native_text_box = text_box.as_widget().as_winui().cast::<MUXC::TextBox>()?;
    native_text_box.SetFontSize(size)?;
    Ok(())
}

pub fn apply_image_to_button(button: &Button, image_path: &str) -> windows_core::Result<()> {
    use windows_core::Interface;
    let path = image_path.replace("\\", "/");
    let xaml = format!(
        "<Grid xmlns='http://schemas.microsoft.com/winfx/2006/xaml/presentation' Background='Transparent' CornerRadius='8'>
            <Image Source='file:///{}' Stretch='Uniform' HorizontalAlignment='Center' VerticalAlignment='Center' Margin='8' />
        </Grid>",
        path
    );
    let content = XamlReader::Load(&HSTRING::from(xaml))?;
    let native_button = button.as_widget().as_winui().cast::<MUXC::Button>()?;
    native_button.SetContent(&content)?;
    Ok(())
}