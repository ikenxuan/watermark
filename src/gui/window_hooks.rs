use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use winio::prelude::*;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_V};
use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard};
use windows::Win32::UI::Shell::{DefSubclassProc, DragAcceptFiles, DragFinish, DragQueryFileW, HDROP, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{WM_DROPFILES, WM_KEYDOWN, WM_NCDESTROY, WM_PASTE};

use crate::Message;

pub fn enable_paste_and_drag_hooks(window: &Child<Window>, sender: ComponentSender<crate::MainModel>) -> winio::Result<()> {
    let hwnd = HWND(window.as_window().handle().unwrap_or(std::ptr::null_mut()));
    unsafe {
        DragAcceptFiles(hwnd, true);
        let ptr = Box::into_raw(Box::new(sender));
        let _ = SetWindowSubclass(hwnd, Some(subclass_proc), 1, ptr as usize);
    }
    Ok(())
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uidsubclass: usize,
    dwrefdata: usize,
) -> LRESULT {
    let sender = unsafe { &*(dwrefdata as *const ComponentSender<crate::MainModel>) };
    match msg {
        WM_NCDESTROY => {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), uidsubclass);
                let _ = Box::from_raw(dwrefdata as *mut ComponentSender<crate::MainModel>);
            }
        }
        WM_DROPFILES => {
            let hdrop = HDROP(wparam.0 as _);
            let count = unsafe { DragQueryFileW(hdrop, u32::MAX, None) };
            if count > 0 {
                let mut buf = [0u16; 260];
                if unsafe { DragQueryFileW(hdrop, 0, Some(&mut buf)) } > 0 {
                    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                    let path = OsString::from_wide(&buf[..len]);
                    sender.post(Message::ImportPath(PathBuf::from(path)));
                }
            }
            unsafe { DragFinish(hdrop) };
            return LRESULT(0);
        }
        WM_KEYDOWN => {
            if wparam.0 as u16 == VK_V.0 {
                if unsafe { (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 } {
                    if let Some(path) = handle_paste(hwnd) {
                        sender.post(Message::ImportPath(path));
                    }
                    return LRESULT(0);
                }
            }
        }
        WM_PASTE => {
            if let Some(path) = handle_paste(hwnd) {
                sender.post(Message::ImportPath(path));
            }
            return LRESULT(0);
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

fn handle_paste(hwnd: HWND) -> Option<PathBuf> {
    unsafe {
        let cf_hdrop = 15; // CF_HDROP
        if IsClipboardFormatAvailable(cf_hdrop).is_err() {
            return None;
        }
        if OpenClipboard(Some(hwnd)).is_err() {
            return None;
        }
        let handle = GetClipboardData(cf_hdrop).ok()?;
        let hdrop = HDROP(handle.0 as _);
        let count = DragQueryFileW(hdrop, u32::MAX, None);
        let mut result = None;
        if count > 0 {
            let mut buf = [0u16; 260];
            if DragQueryFileW(hdrop, 0, Some(&mut buf)) > 0 {
                let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                let path = OsString::from_wide(&buf[..len]);
                result = Some(PathBuf::from(path));
            }
        }
        let _ = CloseClipboard();
        result
    }
}