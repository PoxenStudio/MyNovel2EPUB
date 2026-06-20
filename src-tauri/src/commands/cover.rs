use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use tauri::{AppHandle, Manager};

use super::genre::resource_path;
use crate::models::{CoverTemplate, SynthesizeCoverRequest};

const TEMPLATE_IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

/// 各底图模板的中文名称，定义在 covers/template-names.json 中。
fn load_template_names(covers_dir: &Path) -> std::collections::HashMap<String, String> {
    let names_path = covers_dir.join("template-names.json");
    let Ok(raw_json) = fs::read_to_string(&names_path) else {
        return std::collections::HashMap::new();
    };
    serde_json::from_str(&raw_json).unwrap_or_default()
}

#[tauri::command]
pub fn get_cover_templates(app: AppHandle) -> Result<Vec<CoverTemplate>, String> {
    let covers_dir = resource_path(&app, "resources/covers")?;
    let Ok(read_dir) = fs::read_dir(&covers_dir) else {
        return Ok(Vec::new());
    };

    let template_names = load_template_names(&covers_dir);

    let mut templates: Vec<PathBuf> = read_dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .map(|ext| TEMPLATE_IMAGE_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .collect();
    templates.sort();

    Ok(templates
        .into_iter()
        .map(|path| {
            let filename = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let name = template_names
                .get(&filename)
                .cloned()
                .unwrap_or(filename);
            CoverTemplate {
                path: path.to_string_lossy().to_string(),
                name,
            }
        })
        .collect())
}

#[tauri::command]
pub fn import_cover_image(app: AppHandle, source_path: String) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png");

    let dest_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("无法获取应用缓存目录: {e}"))?
        .join("imported-covers");
    fs::create_dir_all(&dest_dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let dest_path = dest_dir.join(format!("cover-{timestamp}.{extension}"));

    fs::copy(&source, &dest_path).map_err(|e| format!("拷贝图片失败: {e}"))?;
    Ok(dest_path.to_string_lossy().to_string())
}

pub(crate) fn resolve_font_path(app: &AppHandle, font_family: &str) -> Result<PathBuf, String> {
    let filename = match font_family {
        "Huiwen GangHei" => "HuiwenGangHei.ttf",
        "Huiwen FangSong" => "HuiwenFangSong.ttf",
        "QingLiu ShuShi Ti" => "QingLiuShuShiTi.ttf",
        "HengShan MaoBi CaoShu" => "HengShanMaoBiCaoShu-2.ttf",
        "SanJi PoMo Ti" => "SanJiPoMoTi-2.ttf",
        "YanShi ChunFeng Kai" => "YanShiChunFengKai-2.ttf",
        "Slidefu Regular" => "Slidefu-Regular-2.ttf",
        other => return Err(format!("未知字体: {other}")),
    };
    resource_path(app, &format!("resources/fonts/{filename}"))
}

/// 供前端 Canvas 预览复用同一份字体文件，避免在 public/fonts 下重复存放。
#[tauri::command]
pub fn get_font_resource_path(app: AppHandle, font_family: String) -> Result<String, String> {
    resolve_font_path(&app, &font_family).map(|path| path.to_string_lossy().to_string())
}

/// 使用 ab_glyph 计算文本宽度：遍历所有字符的 glyph 并累加水平偏移
fn measure_text_width<FF: Font, F: ScaleFont<FF>>(scale_font: &F, text: &str) -> f32 {
    text.chars()
        .map(|c| scale_font.glyph_id(c))
        .fold(0.0, |acc, gid| acc + scale_font.h_advance(gid))
}

/// 若标题宽度超出 max_width，则从中间向外查找一个可用的拆分点，分成两行
/// （与前端 CoverPreview 的 splitTitleForWidth 逻辑保持一致）
fn split_title_for_width<FF: Font, F: ScaleFont<FF>>(scale_font: &F, text: &str, max_width: f32) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 1 || measure_text_width(scale_font, text) <= max_width {
        return vec![text.to_string()];
    }

    let mid = chars.len() / 2;
    for offset in 0..chars.len() {
        for idx in [mid.saturating_sub(offset), mid + offset] {
            if idx < 1 || idx >= chars.len() {
                continue;
            }
            let left: String = chars[..idx].iter().collect();
            let right: String = chars[idx..].iter().collect();
            if measure_text_width(scale_font, &left) <= max_width
                && measure_text_width(scale_font, &right) <= max_width
            {
                return vec![left, right];
            }
        }
    }

    let left: String = chars[..mid].iter().collect();
    let right: String = chars[mid..].iter().collect();
    vec![left, right]
}

/// 在底图上叠字生成封面，返回 PNG 字节及图片尺寸。
/// 支持相对比例（0-1）和绝对像素值。
/// - 如果传入 title_position_ratio，则基于 preview_width/preview_height 计算实际像素值
/// - 如果传入 fallback_title_position，则使用绝对像素值并按比例缩放
pub(crate) fn rasterize_cover(
    template_path: &Path,
    font_path: &Path,
    title: &str,
    author: &str,
    // 相对比例 (0-1)
    title_position: Option<(f32, f32)>,
    author_position: Option<(f32, f32)>,
    title_font_ratio: Option<f32>,
    author_font_ratio: Option<f32>,
    // 预览基准尺寸
    preview_width: u32,
    preview_height: u32,
    // 绝对像素值（兼容旧版）
    fallback_title_position: Option<(u32, u32)>,
    fallback_author_position: Option<(u32, u32)>,
    fallback_title_font_size: Option<f32>,
    fallback_author_font_size: Option<f32>,
    title_color: Option<&str>,
    author_color: Option<&str>,
    // 是否渲染标题/作者文字；为 false 时仅返回底图本身（用于“无文字”模式）
    render_text: bool,
) -> Result<(Vec<u8>, u32, u32), String> {
    let mut image = image::open(template_path)
        .map_err(|e| format!("无法打开底图: {e}"))?
        .to_rgba8();
    let (width, height) = (image.width(), image.height());

    if !render_text {
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .map_err(|e| format!("编码封面失败: {e}"))?;
        return Ok((bytes, width, height));
    }

    let font_bytes = fs::read(font_path).map_err(|e| format!("读取字体失败: {e}"))?;
    let font = FontVec::try_from_vec(font_bytes).map_err(|e| format!("字体解析失败: {e}"))?;

    // 计算比例因子，用于将预览尺寸映射到实际封面尺寸
    let scale_x = width as f32 / preview_width as f32;
    let scale_y = height as f32 / preview_height as f32;

    // 计算实际像素位置和字号
    let (title_x, title_y, title_size) = if let Some((rx, ry)) = title_position {
        // 使用相对比例
        (
            (rx * width as f32) as i32,
            (ry * height as f32) as i32,
            title_font_ratio.unwrap_or(0.1) * height as f32,
        )
    } else if let Some((px, py)) = fallback_title_position {
        // 使用绝对像素值并缩放
        (
            (px as f32 * scale_x) as i32,
            (py as f32 * scale_y) as i32,
            fallback_title_font_size.unwrap_or(48.0) * scale_y,
        )
    } else {
        return Err("缺少标题位置信息".to_string());
    };

    let (author_x, author_y, author_size) = if let Some((rx, ry)) = author_position {
        // 使用相对比例
        (
            (rx * width as f32) as i32,
            (ry * height as f32) as i32,
            author_font_ratio.unwrap_or(0.05) * height as f32,
        )
    } else if let Some((px, py)) = fallback_author_position {
        // 使用绝对像素值并缩放
        (
            (px as f32 * scale_x) as i32,
            (py as f32 * scale_y) as i32,
            fallback_author_font_size.unwrap_or(24.0) * scale_y,
        )
    } else {
        return Err("缺少作者位置信息".to_string());
    };

    fn parse_color(color_str: &str) -> Rgba<u8> {
        match color_str {
            "#000000" => Rgba([0, 0, 0, 255]),         // 黑色
            "#ffffff" => Rgba([255, 255, 255, 255]),   // 白色
            "#ff0000" => Rgba([255, 0, 0, 255]),       // 红色
            "#0000ff" => Rgba([0, 0, 255, 255]),       // 蓝色
            "#ffff00" => Rgba([255, 255, 0, 255]),     // 黄色
            _ => Rgba([255, 255, 255, 255]),           // 默认白色
        }
    }

    let title_rgba = title_color.map_or(Rgba([255, 255, 255, 255]), parse_color);
    let author_rgba = author_color.map_or(Rgba([255, 255, 255, 255]), parse_color);

    // 创建缩放后的字体对象用于计算文本宽度
    let title_scale_font = font.as_scaled(PxScale::from(title_size));
    let author_scale_font = font.as_scaled(PxScale::from(author_size));

    let author_width = measure_text_width(&author_scale_font, author);

    // 标题过长时换行显示两行，始终保持居中（与前端 CoverPreview 逻辑一致）
    let margin_px = width as f32 * 0.04;
    let half_avail = (title_x as f32).min(width as f32 - title_x as f32) - margin_px;
    let max_title_width = if half_avail > 0.0 { half_avail * 2.0 } else { width as f32 * 0.84 };
    let title_lines = split_title_for_width(&title_scale_font, title, max_title_width);
    let line_height = title_size * 1.15;

    for (i, line) in title_lines.iter().enumerate() {
        let line_width = measure_text_width(&title_scale_font, line);
        let center_y = title_y as f32 - (title_lines.len() as f32 - 1.0) * line_height / 2.0
            + i as f32 * line_height;
        let x = title_x - (line_width / 2.0) as i32;
        let y = (center_y - title_size / 2.0) as i32;
        draw_text_mut(&mut image, title_rgba, x, y, PxScale::from(title_size), &font, line);
    }

    let author_x_center = author_x - (author_width / 2.0) as i32;
    let author_y_center = author_y - (author_size / 2.0) as i32;

    draw_text_mut(
        &mut image,
        author_rgba,
        author_x_center,
        author_y_center,
        PxScale::from(author_size),
        &font,
        author,
    );

    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|e| format!("编码封面失败: {e}"))?;

    Ok((bytes, width, height))
}

#[tauri::command]
pub fn synthesize_cover(app: AppHandle, request: SynthesizeCoverRequest) -> Result<String, String> {
    let font_path = resolve_font_path(&app, &request.font_family)?;
    let (bytes, _, _) = rasterize_cover(
        Path::new(&request.template_path),
        &font_path,
        &request.title,
        &request.author,
        // 使用绝对像素值，设置 preview 与实际相同则无需缩放
        None,
        None,
        None,
        None,
        request.title_position.x as u32,
        request.title_position.y as u32,
        Some((request.title_position.x as u32, request.title_position.y as u32)),
        Some((request.author_position.x as u32, request.author_position.y as u32)),
        Some(request.title_font_size),
        Some(request.author_font_size),
        None,
        None,
        true,
    )?;

    let out_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("无法获取应用缓存目录: {e}"))?
        .join("cover-previews");
    fs::create_dir_all(&out_dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let out_path = out_dir.join(format!("cover-{timestamp}.png"));
    fs::write(&out_path, &bytes).map_err(|e| format!("保存封面失败: {e}"))?;

    Ok(out_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterizes_cover_onto_real_template() {
        let template_path = Path::new("resources/covers/template_01.jpg");
        let font_path = Path::new("resources/fonts/HuiwenGangHei.ttf");

        // 使用相对比例测试
        let (bytes, width, height) = rasterize_cover(
            template_path,
            font_path,
            "测试书名",
            "测试作者",
            Some((0.133, 0.133)),     // title_position_ratio
            Some((0.133, 0.844)),     // author_position_ratio
            Some(0.133),              // title_font_ratio
            Some(0.053),              // author_font_ratio
            600,                      // preview_width
            900,                      // preview_height
            None,
            None,
            None,
            None,
            None,
            None,
            true,
        )
        .expect("rasterize_cover should succeed against real resources");

        assert_eq!((width, height), (600, 900));
        assert!(bytes.len() > 1000);
        assert_eq!(&bytes[1..4], b"PNG");
    }
}
