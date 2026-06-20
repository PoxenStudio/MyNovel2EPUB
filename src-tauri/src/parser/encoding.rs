use encoding_rs::{GBK, UTF_8};

/// 自动检测编码（先 UTF-8，失败则回退 GBK）并统一转为 UTF-8 字符串。
pub fn decode_to_utf8(bytes: &[u8]) -> String {
    let (text, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return text.into_owned();
    }
    let (text, _, _) = GBK.decode(bytes);
    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_bytes_unchanged() {
        let text = "第一章 你好世界";
        assert_eq!(decode_to_utf8(text.as_bytes()), text);
    }

    #[test]
    fn falls_back_to_gbk_for_non_utf8_bytes() {
        let (gbk_bytes, _, _) = GBK.encode("第一章 你好世界");
        assert_eq!(decode_to_utf8(&gbk_bytes), "第一章 你好世界");
    }
}
