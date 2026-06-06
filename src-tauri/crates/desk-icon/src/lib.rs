use desk_core::db::{DbState, SqliteItemRepo};
use desk_core::domain::item::ItemRepo;
use desk_core::error::AppError;
use std::path::PathBuf;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

// --- 图标状态 ---

pub struct IconState {
    pub repo: Box<dyn ItemRepo>,
    pub app_data_dir: PathBuf,
}

// --- 图标提取服务 ---

mod icon_extractor {
    use desk_core::error::AppError;
    use std::path::{Path, PathBuf};
    use windows::Win32::UI::Shell::ExtractIconW;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, BITMAP,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
    use windows::core::PCWSTR;

    pub fn extract_and_save_icon(
        exe_path: &str,
        icons_dir: &Path,
        item_id: i64,
    ) -> Result<Option<String>, AppError> {
        let icon = unsafe { extract_icon(exe_path)? };
        let Some(icon) = icon else {
            return Ok(None);
        };

        let png_path = icons_dir.join(format!("{}.png", item_id));
        let png_path_str = png_path.to_string_lossy().to_string();

        unsafe {
            save_icon_as_png(icon, &png_path)?;
        }
        unsafe {
            let _ = DestroyIcon(icon);
        }

        Ok(Some(png_path_str))
    }

    unsafe fn extract_icon(
        exe_path: &str,
    ) -> Result<Option<windows::Win32::UI::WindowsAndMessaging::HICON>, AppError> {
        let wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
        let icon = ExtractIconW(None, PCWSTR(wide_path.as_ptr()), 0);
        if icon.is_invalid() || icon.0 == std::ptr::null_mut() {
            return Ok(None);
        }
        Ok(Some(icon))
    }

    unsafe fn save_icon_as_png(
        icon: windows::Win32::UI::WindowsAndMessaging::HICON,
        png_path: &Path,
    ) -> Result<(), AppError> {
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(icon, &mut icon_info).is_err() {
            return Err(AppError::Icon("Failed to get icon info".to_string()));
        }

        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            return Err(AppError::Icon("Failed to create DC".to_string()));
        }

        let old_color = SelectObject(hdc, icon_info.hbmColor.into());

        let mut bmp = BITMAP::default();
        let bmp_size = std::mem::size_of::<BITMAP>() as i32;
        let result = GetObjectW(
            icon_info.hbmColor.into(),
            bmp_size,
            Some(&mut bmp as *mut _ as *mut _),
        );

        if result == 0 {
            let _ = SelectObject(hdc, old_color);
            let _ = DeleteDC(hdc);
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            return Err(AppError::Icon("Failed to get bitmap info".to_string()));
        }

        let width = bmp.bmWidth.max(1).min(256) as u32;
        let height = bmp.bmHeight.abs().max(1).min(256) as u32;
        let size = (width * height * 4) as usize;
        let mut color_pixels: Vec<u8> = vec![0u8; size];

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let dib_result = GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height,
            Some(color_pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        let _ = SelectObject(hdc, old_color);

        if dib_result == 0 {
            let _ = DeleteDC(hdc);
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            return Err(AppError::Icon("Failed to get color bits".to_string()));
        }

        let mut mask_pixels: Vec<u8> = vec![];
        let has_mask = !icon_info.hbmMask.is_invalid()
            && icon_info.hbmMask.0 != std::ptr::null_mut();

        if has_mask {
            let old_mask = SelectObject(hdc, icon_info.hbmMask.into());

            let mask_size = (width as usize * height as usize * 4) as usize;
            mask_pixels = vec![0u8; mask_size];

            let mut mask_bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let _ = GetDIBits(
                hdc,
                icon_info.hbmMask,
                0,
                height,
                Some(mask_pixels.as_mut_ptr() as *mut _),
                &mut mask_bmi,
                DIB_RGB_COLORS,
            );

            let _ = SelectObject(hdc, old_mask);
        }

        let _ = DeleteDC(hdc);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());

        for y in 0..height {
            for x in 0..width {
                let idx = ((y as usize * width as usize + x as usize) * 4) as usize;
                if idx + 3 >= color_pixels.len() {
                    break;
                }

                let b = color_pixels[idx];
                let g = color_pixels[idx + 1];
                let r = color_pixels[idx + 2];

                color_pixels[idx] = r;
                color_pixels[idx + 1] = g;
                color_pixels[idx + 2] = b;

                if has_mask && !mask_pixels.is_empty() {
                    let mask_idx = (y as usize * width as usize + x as usize) * 4;
                    if mask_idx + 3 < mask_pixels.len() {
                        let mask_val = mask_pixels[mask_idx]
                            | mask_pixels[mask_idx + 1]
                            | mask_pixels[mask_idx + 2];
                        if mask_val == 0 {
                            color_pixels[idx + 3] = 0;
                        } else {
                            color_pixels[idx + 3] = 255;
                        }
                    } else {
                        color_pixels[idx + 3] = 255;
                    }
                } else {
                    if r == 0 && g == 0 && b == 0 && color_pixels[idx + 3] == 0 {
                        color_pixels[idx + 3] = 0;
                    } else {
                        color_pixels[idx + 3] = 255;
                    }
                }
            }
        }

        let png_data = encode_png(&color_pixels, width, height);
        std::fs::write(png_path, png_data).map_err(|e| AppError::Icon(e.to_string()))?;

        Ok(())
    }

    fn encode_png(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(pixels).unwrap();
        }
        png_data
    }

    pub fn ensure_icons_dir(app_data_dir: &Path) -> PathBuf {
        let icons_dir = app_data_dir.join("icons");
        let _ = std::fs::create_dir_all(&icons_dir);
        icons_dir
    }
}

// --- Tauri 命令 ---

mod commands {
    use super::{icon_extractor, AppError, IconState};
    use base64::Engine;
    use tauri::State;

    #[tauri::command]
    pub fn extract_icon_for_item(
        item_id: i64,
        state: State<'_, IconState>,
    ) -> Result<Option<String>, AppError> {
        let (path, item_type) = state.repo.get_path_and_type(item_id)?;

        if item_type == "Web" {
            return Ok(None);
        }

        let icons_dir = icon_extractor::ensure_icons_dir(&state.app_data_dir);
        let result = icon_extractor::extract_and_save_icon(&path, &icons_dir, item_id)?;

        if let Some(ref icon_path) = result {
            state.repo.update_icon_path(item_id, icon_path)?;
        }

        Ok(result)
    }

    #[tauri::command]
    pub fn get_item_icon_base64(
        item_id: i64,
        state: State<'_, IconState>,
    ) -> Result<Option<String>, AppError> {
        let item = state.repo.get_by_id(item_id)?;

        let icon_path = match item.and_then(|i| i.icon_path) {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };

        let path = std::path::Path::new(&icon_path);
        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(path).map_err(|e| AppError::Icon(e.to_string()))?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        Ok(Some(format!("data:image/png;base64,{encoded}")))
    }
}

// --- 插件初始化 ---

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-icon")
        .invoke_handler(tauri::generate_handler![
            commands::extract_icon_for_item,
            commands::get_item_icon_base64,
        ])
        .setup(|app, _api| {
            let db_state = app.state::<DbState>().inner().clone();
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Icon(e.to_string()))?;
            let repo = SqliteItemRepo::new(db_state);
            app.manage(IconState {
                repo: Box::new(repo),
                app_data_dir,
            });
            Ok(())
        })
        .build()
}
