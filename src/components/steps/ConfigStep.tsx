import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useBookContext } from "../../context/BookContext";
import { GenreSelector } from "../config/GenreSelector";
import { CoverConfig } from "../config/CoverConfig";
import { CoverPreview } from "../config/CoverPreview";
import { EpubPreview } from "../config/EpubPreview";
import { useEpubPreview } from "../../hooks/useEpubPreview";
import type { CoverConfig as CoverConfigData } from "../../types";

export function ConfigStep() {
  const { state, dispatch } = useBookContext();
  const [chapterIndex, setChapterIndex] = useState(0);
  const chapters = state.book.chapters;
  const chapter = chapters[chapterIndex] ?? null;

  const { epubData, isLoading, error } = useEpubPreview({
    sourcePath: state.sourcePath,
    chapter,
    chapterIndex,
    bookTitle: state.book.title,
    chapterStyleId: state.book.chapterStyleId,
  });

  function updateBook(patch: Partial<typeof state.book>) {
    dispatch({ type: "UPDATE_BOOK", book: patch });
  }

  function updateCover(patch: Partial<CoverConfigData>) {
    dispatch({
      type: "UPDATE_BOOK",
      book: { cover: { ...state.book.cover, ...patch } },
    });
  }

  async function handleUploadCoverImage() {
    const sourcePath = await open({
      multiple: false,
      filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp"] }],
    });
    if (typeof sourcePath !== "string") return;
    try {
      const importedPath = await invoke<string>("import_cover_image", {
        sourcePath,
      });
      updateCover({ templatePath: importedPath });
    } catch (e) {
      console.error("上传封面图片失败:", e);
    }
  }

  return (
    <div className="flex flex-col gap-6 py-8">
      <div className="grid grid-cols-3 gap-4">
        <label className="form-control">
          <span className="label-text text-xs">📖 书名</span>
          <input
            type="text"
            className="input input-bordered input-sm"
            value={state.book.title}
            onChange={(e) => updateBook({ title: e.target.value })}
          />
        </label>

        <label className="form-control">
          <span className="label-text text-xs">✍️ 作者</span>
          <input
            type="text"
            className="input input-bordered input-sm"
            value={state.book.author}
            onChange={(e) => updateBook({ author: e.target.value })}
          />
        </label>

        <GenreSelector />
      </div>

      <CoverConfig />

      <div className="flex justify-center gap-8">
        <div className="flex flex-col items-center gap-3">
          <CoverPreview
            title={state.book.title}
            author={state.book.author}
            cover={state.book.cover}
          />
          <button
            type="button"
            className="btn btn-sm"
            onClick={handleUploadCoverImage}
          >
            上传封面背景图
          </button>
        </div>

        <div className="flex flex-col items-center gap-3">
          <EpubPreview epubData={epubData} isLoading={isLoading} error={error} />
          {chapters.length > 0 && (
            <div className="flex w-[360px] items-center justify-center gap-2">
              <button
                type="button"
                className="btn btn-sm"
                disabled={chapterIndex === 0}
                onClick={() => setChapterIndex((i) => Math.max(0, i - 1))}
              >
                ◀ 上一章
              </button>
              <select
                className="select select-bordered select-sm flex-1"
                value={chapterIndex}
                onChange={(e) => setChapterIndex(Number(e.target.value))}
              >
                {chapters.map((c, index) => (
                  <option key={c.id} value={index}>
                    {c.title}
                  </option>
                ))}
              </select>
              <button
                type="button"
                className="btn btn-sm"
                disabled={chapterIndex === chapters.length - 1}
                onClick={() =>
                  setChapterIndex((i) => Math.min(chapters.length - 1, i + 1))
                }
              >
                下一章 ▶
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="flex justify-center gap-2">
        <button
          type="button"
          className="btn"
          onClick={() => dispatch({ type: "SET_STEP", step: "import" })}
        >
          ← 上一步
        </button>
        <button
          type="button"
          className="btn btn-primary"
          onClick={() => dispatch({ type: "SET_STEP", step: "export" })}
        >
          下一步 →
        </button>
      </div>
    </div>
  );
}
