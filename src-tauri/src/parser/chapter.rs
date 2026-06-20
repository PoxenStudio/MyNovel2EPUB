use fancy_regex::Regex;
use tauri::{AppHandle, Emitter};

use crate::models::{Chapter, ParseProgress};

const PROGRESS_EMIT_INTERVAL: usize = 500;
/// 章节标题行的最大字符数。正文段落（如含有“……一回家就……”之类、恰好命中
/// 数字+回/章/节等标记字符的长句）远超此长度，借此与真正的章节标题区分。
const MAX_TITLE_LINE_CHARS: usize = 60;

/// 旧版内置正则，覆盖最常见的数字+章/节/卷/回/集场景，已知存在“一回家”一类的误判。
/// 仅保留用于回归测试对比，不再作为前端默认值。
#[cfg(test)]
const CALIBRE_REGEX: &str =
    r"^第?[0-9一二三四五六七八九十百千]+[章节卷回集].*|^章[0-9]+.*|^引子|^楔子|^尾声";

/// 新版默认正则：覆盖序章/序言/卷首语/番外/上中下册等更多中文小说标题形态，
/// 并通过否定预查规避“回头/回去”一类正文句子被误判为章节标题。
/// 依赖 fancy-regex 支持的 (?=...)/(?!...) 预查语法，标准 `regex` crate 不支持。
pub const FULL_REGEX: &str = r"^(?:(?:序章|序言|引子|前言|卷首语|扉页|楔子|正文卷?|终章|后记|附录|尾声|番外篇?)(?=$|[\s：:，,、．.·\-—（(【\[「《0-9零一二三四五六七八九十第])|番外之|[上中下][部册](?=$|[\s：:])|第?\s{0,4}[\d〇零一二两三四五六七八九十百千万壹贰叁肆伍陆柒捌玖拾佰仟]+?\s{0,4}(?:章|节(?!课)|卷|集(?![合和])|幕(?![前后布])|回(?![合访忆顾应答音到头来去了过])|部(?![分赛游])|篇(?!张))).*";

fn word_count_for(lines: &[&str], start_line: usize, end_line: usize) -> usize {
    lines[start_line - 1..end_line]
        .iter()
        .map(|line| line.chars().filter(|c| !c.is_whitespace()).count())
        .sum()
}

/// 按正则切分章节的核心逻辑，通过回调上报进度，不依赖 Tauri 运行时，便于单元测试。
pub fn parse_chapters_with_progress(
    content: &str,
    pattern: &str,
    mut on_progress: impl FnMut(ParseProgress),
) -> Result<Vec<Chapter>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("正则表达式无效: {e}"))?;
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len().max(1);

    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current_start: Option<usize> = None;
    let mut current_title = String::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        let is_title = trimmed.chars().count() <= MAX_TITLE_LINE_CHARS
            && re
                .is_match(line.trim_start())
                .map_err(|e| format!("正则匹配失败: {e}"))?;
        if is_title {
            if let Some(start) = current_start {
                chapters.push(Chapter {
                    id: chapters.len() as u32 + 1,
                    title: current_title.clone(),
                    start_line: start,
                    end_line: line_no - 1,
                    word_count: word_count_for(&lines, start, line_no - 1),
                });
            }
            current_start = Some(line_no);
            current_title = line.trim().to_string();
        }

        if line_no % PROGRESS_EMIT_INTERVAL == 0 || line_no == total_lines {
            let percent = ((line_no as f64 / total_lines as f64) * 100.0) as u8;
            on_progress(ParseProgress {
                percent,
                current_chapter: current_title.clone(),
                total_chapters: chapters.len(),
            });
        }
    }

    if let Some(start) = current_start {
        chapters.push(Chapter {
            id: chapters.len() as u32 + 1,
            title: current_title.clone(),
            start_line: start,
            end_line: total_lines,
            word_count: word_count_for(&lines, start, total_lines),
        });
    }

    Ok(chapters)
}

/// 按正则切分章节，解析过程中通过 `parse-progress` 事件实时推送进度。
pub fn parse_chapters(content: &str, pattern: &str, app: &AppHandle) -> Result<Vec<Chapter>, String> {
    parse_chapters_with_progress(content, pattern, |progress| {
        let _ = app.emit("parse-progress", progress);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_chapters_on_chinese_numerals() {
        let content = "\
第一章 初入江湖
这是开篇的内容。
还有更多内容。
第二章 风云再起
后续剧情发展。
";
        let chapters = parse_chapters_with_progress(content, CALIBRE_REGEX, |_| {}).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 初入江湖");
        assert_eq!(chapters[0].start_line, 1);
        assert_eq!(chapters[0].end_line, 3);
        assert_eq!(chapters[1].title, "第二章 风云再起");
        assert_eq!(chapters[1].start_line, 4);
        assert_eq!(chapters[1].end_line, 5);
    }

    #[test]
    fn recognizes_prologue_and_epilogue_markers() {
        let content = "\
引子
故事开始之前。
第一章 启程
正文。
尾声
故事结束。
";
        let chapters = parse_chapters_with_progress(content, CALIBRE_REGEX, |_| {}).unwrap();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "引子");
        assert_eq!(chapters[1].title, "第一章 启程");
        assert_eq!(chapters[2].title, "尾声");
    }

    #[test]
    fn returns_empty_when_no_heading_matches() {
        let content = "没有任何章节标题的纯文本内容。\n继续没有章节标题。\n";
        let chapters = parse_chapters_with_progress(content, CALIBRE_REGEX, |_| {}).unwrap();
        assert!(chapters.is_empty());
    }

    #[test]
    fn does_not_misdetect_long_paragraph_starting_with_numeral_and_marker_char() {
        // “一回家” 中的“一”“回”恰好命中“数字+回”模式，但整行是叙事长句而非标题，
        // 不应被误判为新章节。
        let content = "\
第一章 启程
一回家就和继母又吵了一架。因为她裁掉的行政人员中，有继母的弟弟。傅太太早就对她有一肚子的不满，只苦于见不到她，听说她回家了，气冲冲的走进客厅：“大小姐回来了？真是稀客，我还以为你一辈子都不见我们傅家人了。”要是从前，她低头就忍了，可是今天她刚在公司盘完账，精疲力竭，回家来听她这样一篇话，好气又好笑：“这是我的家，我回来是天经地义的事情。”
第二章 风云再起
后续剧情发展。
";
        let chapters = parse_chapters_with_progress(content, CALIBRE_REGEX, |_| {}).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 启程");
        assert_eq!(chapters[0].start_line, 1);
        assert_eq!(chapters[0].end_line, 2);
        assert_eq!(chapters[1].title, "第二章 风云再起");
    }

    fn full_regex_matches(line: &str) -> bool {
        Regex::new(FULL_REGEX).unwrap().is_match(line).unwrap()
    }

    #[test]
    fn full_regex_recognizes_extended_title_forms() {
        for title in [
            "序章",
            "序章：风起",
            "前言",
            "卷首语",
            "番外之一",
            "番外篇一",
            "上册",
            "下部",
            "第十章 风起",
            "正文卷 第一章 风起",
        ] {
            assert!(full_regex_matches(title), "应当识别为标题: {title}");
        }
    }

    #[test]
    fn full_regex_rejects_sentences_that_only_resemble_titles() {
        for sentence in [
            "前言不搭后语",
            "三回头看见他",
            "上册第3页写道",
            "中部，丘陵连绵起伏",
        ] {
            assert!(!full_regex_matches(sentence), "不应误判为标题: {sentence}");
        }
    }

    #[test]
    fn full_regex_splits_chapters_with_extended_title_forms() {
        let content = "\
序章
故事开始之前。
番外之一 番外篇
番外内容。
第一章 启程
正文。
";
        let chapters = parse_chapters_with_progress(content, FULL_REGEX, |_| {}).unwrap();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "序章");
        assert_eq!(chapters[1].title, "番外之一 番外篇");
        assert_eq!(chapters[2].title, "第一章 启程");
    }

    #[test]
    fn rejects_invalid_regex() {
        let result = parse_chapters_with_progress("第一章 测试\n", "(", |_| {});
        assert!(result.is_err());
    }
}
