import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useBookContext } from "../../context/BookContext";

interface Props {
  onExportStart: () => void;
  onExportDone: (error: string | null) => void;
}

// 预览基准尺寸（与 CoverPreview 的 CANVAS_WIDTH/CANVAS_HEIGHT 一致）
const PREVIEW_WIDTH = 600;
const PREVIEW_HEIGHT = 900;

export function ExportButton({ onExportStart, onExportDone }: Props) {
  const { state } = useBookContext();
  const [isExporting, setIsExporting] = useState(false);

  async function handleExport() {
    if (!state.sourcePath) return;

    const savePath = await save({
      defaultPath: `${state.book.title || "未命名书籍"}.epub`,
      filters: [{ name: "EPUB", extensions: ["epub"] }],
    });
    if (!savePath) return;

    setIsExporting(true);
    onExportStart();
    try {
      await invoke("generate_epub", {
        sourcePath: state.sourcePath,
        book: {
          title: state.book.title,
          author: state.book.author,
          chapterStyleId: state.book.chapterStyleId ?? state.book.genre,
          cover: {
            mode: state.book.cover.mode,
            templatePath: state.book.cover.templatePath,
            // 传递相对比例
            titlePositionRatio: state.book.cover.titlePositionRatio,
            authorPositionRatio: state.book.cover.authorPositionRatio,
            titleFontRatio: state.book.cover.titleFontRatio,
            authorFontRatio: state.book.cover.authorFontRatio,
            previewWidth: PREVIEW_WIDTH,
            previewHeight: PREVIEW_HEIGHT,
            fontFamily: state.book.cover.fontFamily,
            titleColor: state.book.cover.titleColor,
            authorColor: state.book.cover.authorColor,
          },
          chapters: state.book.chapters.map((c) => ({
            title: c.title,
            startLine: c.startLine,
            endLine: c.endLine,
          })),
        },
        savePath,
      });
      onExportDone(null);
    } catch (e) {
      onExportDone(String(e));
    } finally {
      setIsExporting(false);
    }
  }

  return (
    <button
      type="button"
      className="btn btn-success"
      disabled={!state.sourcePath || state.book.chapters.length === 0 || isExporting}
      onClick={handleExport}
    >
      {isExporting ? "导出中..." : "📦 选择保存路径并导出 EPUB"}
    </button>
  );
}
