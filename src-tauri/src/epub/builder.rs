use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use serde::Serialize;
use tauri::AppHandle;
use tera::Tera;

use crate::commands::genre::{list_header_images, resource_path};

/// 解析章节样式目录：自定义样式传入的是用户选择的绝对路径，直接使用；
/// 否则按内置样式 id 解析到 app 资源包内的固定目录。
fn resolve_style_dir(app: &AppHandle, chapter_style_id: &str) -> Result<PathBuf, String> {
    let custom_path = Path::new(chapter_style_id);
    if custom_path.is_absolute() {
        return Ok(custom_path.to_path_buf());
    }
    resource_path(
        app,
        &format!("resources/epub-assets/chapter-styles/{chapter_style_id}"),
    )
}

const CHAPTER_TEMPLATE: &str = include_str!("../../templates/chapter.xhtml.tera");
const COVER_TEMPLATE: &str = include_str!("../../templates/cover.xhtml.tera");

#[derive(Serialize)]
struct ChapterRenderContext {
    chapter_id: String,
    title: String,
    variant_class: String,
    header_image_href: String,
    header_ornament_href: Option<String>,
    footer_ornament_href: Option<String>,
    paragraphs: Vec<String>,
}

#[derive(Serialize)]
struct CoverRenderContext {
    title: String,
    cover_image_href: String,
    width: u32,
    height: u32,
}

fn render_template<T: Serialize>(name: &str, template: &str, ctx: &T) -> Result<String, String> {
    let mut tera = Tera::default();
    tera.add_raw_template(name, template)
        .map_err(|e| format!("加载模板 {name} 失败: {e}"))?;
    let context =
        tera::Context::from_serialize(ctx).map_err(|e| format!("构建模板上下文失败: {e}"))?;
    tera.render(name, &context)
        .map_err(|e| format!("渲染模板 {name} 失败: {e}"))
}

fn render_chapter_xhtml(ctx: &ChapterRenderContext) -> Result<String, String> {
    render_template("chapter.xhtml", CHAPTER_TEMPLATE, ctx)
}

fn render_cover_xhtml(ctx: &CoverRenderContext) -> Result<String, String> {
    render_template("cover.xhtml", COVER_TEMPLATE, ctx)
}

/// 读取样式目录下可选的装饰线 SVG（header-line.svg / footer-line.svg）并写入 EPUB。
/// 自定义样式目录可以不提供装饰线，此时返回 None，模板会跳过对应的 `<img>` 渲染。
fn add_optional_ornament(
    builder: &mut EpubBuilder<ZipLibrary>,
    style_dir: &Path,
    filename: &str,
) -> Result<Option<String>, String> {
    let path = style_dir.join(filename);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("读取{filename}失败: {e}"))?;
    let mime_type = path
        .extension()
        .map(|ext| match ext.to_string_lossy().to_lowercase().as_str() {
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            _ => "image/png",
        })
        .unwrap_or("image/png");
    builder
        .add_resource(
            format!("Styles/Decorations/{filename}"),
            bytes.as_slice(),
            mime_type,
        )
        .map_err(|e| e.to_string())?;
    Ok(Some(format!("../Styles/Decorations/{filename}")))
}

/// 按空行切分段落，过滤空白行；与最终导出引擎共用同一套切分规则。
pub fn split_paragraphs(body: &str) -> Vec<String> {
    body.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

/// 生成只含一章内容的临时 EPUB，供前端 epub.js 预览使用。
///
/// 已解析好的章节样式目录 / CSS 路径作为入参，与 Tauri `AppHandle` 解耦，便于单元测试。
fn build_preview_epub_from_paths(
    book_title: &str,
    chapter_title: &str,
    body_text: &str,
    style_dir: &Path,
    css_path: &Path,
    chapter_index: usize,
) -> Result<Vec<u8>, String> {
    let header_images = list_header_images(style_dir);
    if header_images.is_empty() {
        return Err(format!(
            "找不到章节样式目录 {} 下的头图",
            style_dir.display()
        ));
    }
    let header_image_path = &header_images[chapter_index % header_images.len()];
    let header_image_ext = header_image_path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "png".to_string());
    let header_image_mime = if header_image_ext == "jpg" || header_image_ext == "jpeg" {
        "image/jpeg"
    } else {
        "image/png"
    };
    let css_bytes = std::fs::read(css_path).map_err(|e| format!("读取样式表失败: {e}"))?;
    let header_image_bytes =
        std::fs::read(header_image_path).map_err(|e| format!("读取头图失败: {e}"))?;

    let mut builder =
        EpubBuilder::new(ZipLibrary::new().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    builder
        .metadata("title", book_title)
        .map_err(|e| e.to_string())?;
    builder.epub_version(EpubVersion::V30);
    builder
        .add_resource("Styles/mybooks.css", css_bytes.as_slice(), "text/css")
        .map_err(|e| e.to_string())?;
    builder
        .add_resource(
            format!("Images/chapter-header.{header_image_ext}"),
            header_image_bytes.as_slice(),
            header_image_mime,
        )
        .map_err(|e| e.to_string())?;

    let header_ornament_href = add_optional_ornament(&mut builder, style_dir, "header-line.png")?;
    let footer_ornament_href = add_optional_ornament(&mut builder, style_dir, "footer-line.png")?;

    let ctx = ChapterRenderContext {
        chapter_id: "chapter1".to_string(),
        title: chapter_title.to_string(),
        variant_class: "chapter-variant-standard".to_string(),
        header_image_href: format!("../Images/chapter-header.{header_image_ext}"),
        header_ornament_href,
        footer_ornament_href,
        paragraphs: split_paragraphs(body_text),
    };
    let xhtml = render_chapter_xhtml(&ctx)?;

    builder
        .add_content(EpubContent::new("Text/chapter1.xhtml", xhtml.as_bytes()).title(chapter_title))
        .map_err(|e| e.to_string())?;

    let mut output = Vec::new();
    builder.generate(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}

/// 生成只含一章内容的临时 EPUB，供前端 epub.js 预览使用。
pub fn build_preview_epub(
    app: &AppHandle,
    book_title: &str,
    chapter_title: &str,
    body_text: &str,
    chapter_style_id: &str,
    chapter_index: usize,
) -> Result<Vec<u8>, String> {
    let style_dir = resolve_style_dir(app, chapter_style_id)?;
    let css_path = resource_path(app, "resources/epub-assets/mybooks.css")?;

    build_preview_epub_from_paths(
        book_title,
        chapter_title,
        body_text,
        &style_dir,
        &css_path,
        chapter_index,
    )
}

/// 一章待导出的内容：标题 + 已按行号从原文切出的正文。
pub struct ExportChapterInput {
    pub title: String,
    pub body: String,
}

fn mime_for_extension(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        _ => "image/png",
    }
}

/// 组装完整 EPUB 3.0（封面页 + 全部章节 + 固定样式/装饰资源），通过回调上报导出阶段进度。
///
/// 路径与封面字节均为已解析好的入参，与 Tauri `AppHandle` 解耦，便于单元测试。
fn build_full_epub_from_paths(
    book_title: &str,
    author: &str,
    style_dir: &Path,
    css_path: &Path,
    cover: Option<(&[u8], u32, u32)>,
    chapters: &[ExportChapterInput],
    mut on_progress: impl FnMut(&str, u8, &str),
) -> Result<Vec<u8>, String> {
    on_progress("cover", 0, "正在准备封面与样式资源...");

    let header_images = list_header_images(style_dir);
    if header_images.is_empty() {
        return Err(format!(
            "找不到章节样式目录 {} 下的头图",
            style_dir.display()
        ));
    }
    let css_bytes = std::fs::read(css_path).map_err(|e| format!("读取样式表失败: {e}"))?;

    let mut builder =
        EpubBuilder::new(ZipLibrary::new().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    builder
        .metadata("title", book_title)
        .map_err(|e| e.to_string())?;
    builder
        .metadata("author", author)
        .map_err(|e| e.to_string())?;
    builder
        .metadata("lang", "zh")
        .map_err(|e| e.to_string())?;
    builder.epub_version(EpubVersion::V30);

    builder
        .add_resource("Styles/mybooks.css", css_bytes.as_slice(), "text/css")
        .map_err(|e| e.to_string())?;

    let header_ornament_href = add_optional_ornament(&mut builder, style_dir, "header-line.png")?;
    let footer_ornament_href = add_optional_ornament(&mut builder, style_dir, "footer-line.png")?;

    let mut header_image_hrefs = Vec::with_capacity(header_images.len());
    for image_path in &header_images {
        let filename = image_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or("头图文件名无效")?;
        let extension = image_path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "png".to_string());
        let bytes = std::fs::read(image_path).map_err(|e| format!("读取头图失败: {e}"))?;
        builder
            .add_resource(
                format!("Images/{filename}"),
                bytes.as_slice(),
                mime_for_extension(&extension),
            )
            .map_err(|e| e.to_string())?;
        header_image_hrefs.push(format!("../Images/{filename}"));
    }

    if let Some((cover_bytes, width, height)) = cover {
        builder
            .add_cover_image("Images/cover.png", cover_bytes, "image/png")
            .map_err(|e| e.to_string())?;
        let cover_ctx = CoverRenderContext {
            title: book_title.to_string(),
            cover_image_href: "../Images/cover.png".to_string(),
            width,
            height,
        };
        let cover_xhtml = render_cover_xhtml(&cover_ctx)?;
        builder
            .add_content(
                EpubContent::new("Text/cover.xhtml", cover_xhtml.as_bytes())
                    .title("封面")
                    .reftype(epub_builder::ReferenceType::Cover),
            )
            .map_err(|e| e.to_string())?;
    }

    let total_chapters = chapters.len().max(1);
    for (index, chapter) in chapters.iter().enumerate() {
        let percent = ((index as f64 / total_chapters as f64) * 100.0) as u8;
        on_progress("chapters", percent, &format!("正在处理章节《{}》...", chapter.title));

        let ctx = ChapterRenderContext {
            chapter_id: format!("chapter{}", index + 1),
            title: chapter.title.clone(),
            variant_class: "chapter-variant-standard".to_string(),
            header_image_href: header_image_hrefs[index % header_image_hrefs.len()].clone(),
            header_ornament_href: header_ornament_href.clone(),
            footer_ornament_href: footer_ornament_href.clone(),
            paragraphs: split_paragraphs(&chapter.body),
        };
        let xhtml = render_chapter_xhtml(&ctx)?;
        builder
            .add_content(
                EpubContent::new(format!("Text/chapter{}.xhtml", index + 1), xhtml.as_bytes())
                    .title(chapter.title.clone()),
            )
            .map_err(|e| e.to_string())?;
    }

    on_progress("packaging", 90, "正在打包 EPUB 容器...");
    let mut output = Vec::new();
    builder.generate(&mut output).map_err(|e| e.to_string())?;
    on_progress("packaging", 100, "导出完成");

    Ok(output)
}

/// 组装完整 EPUB 3.0 并写出，供 `generate_epub` 命令调用。
pub fn build_full_epub(
    app: &AppHandle,
    book_title: &str,
    author: &str,
    chapter_style_id: &str,
    cover: Option<(&[u8], u32, u32)>,
    chapters: &[ExportChapterInput],
    on_progress: impl FnMut(&str, u8, &str),
) -> Result<Vec<u8>, String> {
    let style_dir = resolve_style_dir(app, chapter_style_id)?;
    let css_path = resource_path(app, "resources/epub-assets/mybooks.css")?;

    build_full_epub_from_paths(
        book_title,
        author,
        &style_dir,
        &css_path,
        cover,
        chapters,
        on_progress,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_paragraphs_and_trims_blank_lines() {
        let body = "第一段。\n\n  第二段，带前导空格。  \n\n\n第三段。";
        let paragraphs = split_paragraphs(body);
        assert_eq!(
            paragraphs,
            vec!["第一段。", "第二段，带前导空格。", "第三段。"]
        );
    }

    #[test]
    fn builds_a_valid_epub_zip_from_real_resources() {
        let style_dir = Path::new("resources/epub-assets/chapter-styles/xianxia");
        let css_path = Path::new("resources/epub-assets/mybooks.css");

        let bytes = build_preview_epub_from_paths(
            "测试书名",
            "第一章 测试",
            "这是正文第一段。\n\n这是正文第二段。",
            style_dir,
            css_path,
            0,
        )
        .expect("build_preview_epub_from_paths should succeed against real resources");

        assert_eq!(&bytes[0..2], b"PK", "output should be a valid zip archive");
        assert!(bytes.len() > 1000);
    }

    #[test]
    fn errors_when_chapter_style_directory_is_missing() {
        let style_dir = Path::new("resources/epub-assets/chapter-styles/does-not-exist");
        let css_path = Path::new("resources/epub-assets/mybooks.css");

        let result =
            build_preview_epub_from_paths("书名", "第一章", "正文", style_dir, css_path, 0);
        assert!(result.is_err());
    }

    #[test]
    fn builds_full_epub_with_cover_and_multiple_chapters() {
        let style_dir = Path::new("resources/epub-assets/chapter-styles/fantasy");
        let css_path = Path::new("resources/epub-assets/mybooks.css");
        let cover_bytes = vec![0u8; 16];
        let chapters = vec![
            ExportChapterInput {
                title: "第一章 测试".to_string(),
                body: "第一段。\n\n第二段。".to_string(),
            },
            ExportChapterInput {
                title: "第二章 测试".to_string(),
                body: "另一段内容。".to_string(),
            },
        ];

        let mut phases = Vec::new();
        let bytes = build_full_epub_from_paths(
            "测试书名",
            "测试作者",
            style_dir,
            css_path,
            Some((&cover_bytes, 600, 900)),
            &chapters,
            |phase, _percent, _message| phases.push(phase.to_string()),
        )
        .expect("build_full_epub_from_paths should succeed against real resources");

        assert_eq!(&bytes[0..2], b"PK");
        assert!(bytes.len() > 2000);
        assert!(phases.contains(&"cover".to_string()));
        assert!(phases.contains(&"chapters".to_string()));
        assert!(phases.contains(&"packaging".to_string()));
    }

    #[test]
    fn builds_full_epub_without_cover() {
        let style_dir = Path::new("resources/epub-assets/chapter-styles/scifi");
        let css_path = Path::new("resources/epub-assets/mybooks.css");
        let chapters = vec![ExportChapterInput {
            title: "唯一章节".to_string(),
            body: "正文内容。".to_string(),
        }];

        let bytes = build_full_epub_from_paths(
            "无封面测试书",
            "测试作者",
            style_dir,
            css_path,
            None,
            &chapters,
            |_, _, _| {},
        )
        .expect("should succeed without a cover image");

        assert_eq!(&bytes[0..2], b"PK");
    }
}
