use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use png::Decoder;
use tray_icon::Icon;

// Embed both icon variants at compile time
// Filled = active (ports listening), Outline = inactive (no ports)

// Dark icons (for light mode background / macOS)
static ICON_FILLED_DARK: &[u8] = include_bytes!("../../assets/menubar-icon-filled@2x.png");
static ICON_OUTLINE_DARK: &[u8] = include_bytes!("../../assets/menubar-icon-outline@2x.png");

// Light icons for Windows dark mode (light icons on dark taskbar)
#[cfg(target_os = "windows")]
static ICON_FILLED_LIGHT: &[u8] = include_bytes!("../../assets-light/menubar-icon-filled@2x.png");
#[cfg(target_os = "windows")]
static ICON_OUTLINE_LIGHT: &[u8] = include_bytes!("../../assets-light/menubar-icon-outline@2x.png");

// Cache decoded RGBA data to avoid repeated PNG decoding
struct CachedIconData {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

// Caches for dark theme icons (light mode / macOS)
static ICON_ACTIVE_DARK_CACHE: OnceLock<CachedIconData> = OnceLock::new();
static ICON_INACTIVE_DARK_CACHE: OnceLock<CachedIconData> = OnceLock::new();

// Caches for light theme icons (Windows dark mode)
#[cfg(target_os = "windows")]
static ICON_ACTIVE_LIGHT_CACHE: OnceLock<CachedIconData> = OnceLock::new();
#[cfg(target_os = "windows")]
static ICON_INACTIVE_LIGHT_CACHE: OnceLock<CachedIconData> = OnceLock::new();

/// Icon variant for different states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconVariant {
    /// Outline arrows - no active ports
    Inactive,
    /// Filled arrows - ports are active
    Active,
}

/// Create a template icon for the menu bar.
/// Uses cached decoded RGBA data to avoid repeated PNG decoding.
/// On Windows, selects light or dark icons based on system theme.
/// macOS automatically adapts the color based on menu bar appearance.
#[cfg(target_os = "macos")]
pub fn create_template_icon(variant: IconVariant) -> Result<Icon> {
    let cached = match variant {
        IconVariant::Active => {
            ICON_ACTIVE_DARK_CACHE.get_or_init(|| decode_png_to_rgba(ICON_FILLED_DARK).unwrap())
        }
        IconVariant::Inactive => {
            ICON_INACTIVE_DARK_CACHE.get_or_init(|| decode_png_to_rgba(ICON_OUTLINE_DARK).unwrap())
        }
    };
    Icon::from_rgba(cached.rgba.clone(), cached.width, cached.height)
        .map_err(|e| anyhow!("failed to create icon: {e}"))
}

/// Create a template icon for the system tray.
/// Uses cached decoded RGBA data to avoid repeated PNG decoding.
/// On Windows, selects light icons for dark mode (dark taskbar) and 
/// dark icons for light mode (light taskbar).
#[cfg(target_os = "windows")]
pub fn create_template_icon(variant: IconVariant) -> Result<Icon> {
    use crate::utils::is_windows_dark_mode;
    
    let is_dark_mode = is_windows_dark_mode();
    
    let cached = match (variant, is_dark_mode) {
        // Dark mode: use light icons (visible on dark taskbar)
        (IconVariant::Active, true) => {
            ICON_ACTIVE_LIGHT_CACHE.get_or_init(|| decode_png_to_rgba(ICON_FILLED_LIGHT).unwrap())
        }
        (IconVariant::Inactive, true) => {
            ICON_INACTIVE_LIGHT_CACHE.get_or_init(|| decode_png_to_rgba(ICON_OUTLINE_LIGHT).unwrap())
        }
        // Light mode: use dark icons (visible on light taskbar)
        (IconVariant::Active, false) => {
            ICON_ACTIVE_DARK_CACHE.get_or_init(|| decode_png_to_rgba(ICON_FILLED_DARK).unwrap())
        }
        (IconVariant::Inactive, false) => {
            ICON_INACTIVE_DARK_CACHE.get_or_init(|| decode_png_to_rgba(ICON_OUTLINE_DARK).unwrap())
        }
    };
    
    Icon::from_rgba(cached.rgba.clone(), cached.width, cached.height)
        .map_err(|e| anyhow!("failed to create icon: {e}"))
}

fn decode_png_to_rgba(png_data: &[u8]) -> Result<CachedIconData> {
    let decoder = Decoder::new(png_data);
    let mut reader = decoder
        .read_info()
        .map_err(|e| anyhow!("failed to read PNG header: {e}"))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| anyhow!("failed to decode PNG: {e}"))?;

    let width = info.width;
    let height = info.height;

    // Convert to RGBA if needed
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for chunk in buf[..info.buffer_size()].chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        png::ColorType::GrayscaleAlpha => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for chunk in buf[..info.buffer_size()].chunks(2) {
                let gray = chunk[0];
                let alpha = chunk[1];
                rgba.extend_from_slice(&[gray, gray, gray, alpha]);
            }
            rgba
        }
        png::ColorType::Grayscale => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for &gray in &buf[..info.buffer_size()] {
                rgba.extend_from_slice(&[gray, gray, gray, 255]);
            }
            rgba
        }
        png::ColorType::Indexed => {
            return Err(anyhow!("indexed PNG not supported for menu bar icon"));
        }
    };

    Ok(CachedIconData {
        rgba,
        width,
        height,
    })
}

