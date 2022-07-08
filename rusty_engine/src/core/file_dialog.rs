#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// Win32
use windows::Win32::UI::Controls::Dialogs::*;
use windows::Win32::Foundation::*;

pub fn OpenFile(mut filters: String) -> String
{
    unsafe
    {
        let mut openFileDialog = OPENFILENAMEA::default();
        let mut fileNameBuffer = String::from_utf8(vec![0; 256]).unwrap();
        openFileDialog.lStructSize  = std::mem::size_of::<OPENFILENAMEA>() as u32;
        openFileDialog.hwndOwner    = HWND::default();
        openFileDialog.lpstrFile    = PSTR(fileNameBuffer.as_bytes_mut().as_mut_ptr());
        openFileDialog.nMaxFile     = fileNameBuffer.len() as u32;
        openFileDialog.lpstrFilter  = PSTR(filters.as_bytes_mut().as_mut_ptr());
        openFileDialog.nFilterIndex = 1;
        openFileDialog.Flags        = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        if GetOpenFileNameA(&mut openFileDialog).as_bool()
        {
            return String::from(fileNameBuffer.trim_matches('\0'));
        }

        return String::new();
    }
}

pub fn SaveFile(mut filters: String) -> String
{
    unsafe
    {
        let mut saveFileDialog = OPENFILENAMEA::default();
        let mut fileNameBuffer = String::from_utf8(vec![0; 256]).unwrap();
        saveFileDialog.lStructSize  = std::mem::size_of::<OPENFILENAMEA>() as u32;
        saveFileDialog.hwndOwner    = HWND::default();
        saveFileDialog.lpstrFile    = PSTR(fileNameBuffer.as_bytes_mut().as_mut_ptr());
        saveFileDialog.nMaxFile     = fileNameBuffer.len() as u32;
        saveFileDialog.lpstrFilter  = PSTR(filters.as_bytes_mut().as_mut_ptr());
        saveFileDialog.nFilterIndex = 1;
        saveFileDialog.Flags        = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        if GetSaveFileNameA(&mut saveFileDialog).as_bool()
        {
            return String::from(fileNameBuffer.trim_matches('\0'));
        }

        return String::new();
    }
}