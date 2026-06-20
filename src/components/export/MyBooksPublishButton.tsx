import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBookContext } from "../../context/BookContext";

interface Props {
  bookId: number;
  hadExistingEpub: boolean;
  onExportStart: () => void;
  onExportDone: (error: string | null) => void;
}

// 预览基准尺寸（与 CoverPreview 的 CANVAS_WIDTH/CANVAS_HEIGHT 一致）
const PREVIEW_WIDTH = 600;
const PREVIEW_HEIGHT = 900;

export function MyBooksPublishButton({
  bookId,
  hadExistingEpub,
  onExportStart,
  onExportDone,
}: Props) {
  const { state } = useBookContext();
  const [isPublishing, setIsPublishing] = useState(false);

  async function handlePublish() {
    if (!state.sourcePath) return;
    setIsPublishing(true);
    onExportStart();
    try {
      const epubPath = await invoke<string>("generate_epub_to_cache", {
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
      });

      await invoke("mybooks_publish_epub", {
        bookId,
        epubPath,
        hadExistingEpub,
      });
      onExportDone(null);
    } catch (e) {
      onExportDone(String(e));
    } finally {
      setIsPublishing(false);
    }
  }

  return (
    <div className="flex flex-col items-center gap-1">
      <button
        type="button"
        className="btn btn-success"
        disabled={!state.sourcePath || state.book.chapters.length === 0 || isPublishing}
        onClick={handlePublish}
      >
        {isPublishing ? "更新中..." : "☁️ 更新到 MyBooks 书库"}
      </button>
    </div>
  );
}
