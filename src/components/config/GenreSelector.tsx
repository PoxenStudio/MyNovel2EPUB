import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useGenrePresets } from "../../hooks/useGenrePresets";
import { useBookContext } from "../../context/BookContext";

const CUSTOM_GENRE_ID = "custom";

export function GenreSelector() {
  const { presets, error } = useGenrePresets();
  const { state, dispatch } = useBookContext();
  const [customError, setCustomError] = useState<string | null>(null);

  // 初始化时自动设置 chapterStyleId 和封面背景图
  useEffect(() => {
    if (presets.length === 0 || state.book.chapterStyleId) return;
    const preset = presets.find((p) => p.id === state.book.genre);
    if (preset) {
      dispatch({
        type: "UPDATE_BOOK",
        book: { chapterStyleId: preset.chapterStyleId },
      });
      // 如果样式有默认封面背景图，自动导入并设置
      if (preset.coverImagePath) {
        invoke<string>("import_cover_image", {
          sourcePath: preset.coverImagePath,
        })
          .then((importedPath) => {
            dispatch({
              type: "UPDATE_BOOK",
              book: { cover: { ...state.book.cover, templatePath: importedPath } },
            });
          })
          .catch((e) => console.error("导入封面背景图失败:", e));
      }
    }
  }, [presets, state.book.genre, state.book.chapterStyleId, dispatch]);

  async function applyPreset(genreId: string) {
    const preset = presets.find((p) => p.id === genreId);
    if (!preset) return;
    dispatch({
      type: "UPDATE_BOOK",
      book: { genre: preset.id, chapterStyleId: preset.chapterStyleId },
    });
    // 如果样式有默认封面背景图，自动导入并设置
    if (preset.coverImagePath) {
      try {
        const importedPath = await invoke<string>("import_cover_image", {
          sourcePath: preset.coverImagePath,
        });
        dispatch({
          type: "UPDATE_BOOK",
          book: { cover: { ...state.book.cover, templatePath: importedPath } },
        });
      } catch (e) {
        console.error("导入封面背景图失败:", e);
      }
    }
  }

  async function selectCustomStyleDir() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== "string") return;

    setCustomError(null);
    let coverPath: string;
    try {
      coverPath = await invoke<string>("validate_custom_style_dir", { dir });
    } catch (e) {
      setCustomError(String(e));
      return;
    }

    dispatch({
      type: "UPDATE_BOOK",
      book: { genre: "custom", chapterStyleId: dir },
    });

    try {
      const importedPath = await invoke<string>("import_cover_image", {
        sourcePath: coverPath,
      });
      dispatch({
        type: "UPDATE_BOOK",
        book: { cover: { ...state.book.cover, templatePath: importedPath } },
      });
    } catch (e) {
      console.error("导入自定义封面失败:", e);
    }
  }

  function handleChange(genreId: string) {
    if (genreId === CUSTOM_GENRE_ID) {
      selectCustomStyleDir();
      return;
    }
    setCustomError(null);
    applyPreset(genreId);
  }

  if (error) {
    return <div className="alert alert-error">小说样式加载失败：{error}</div>;
  }

  return (
    <label className="form-control">
      <span className="label-text text-xs">📚 小说样式</span>
      <select
        className="select select-bordered select-sm"
        value={state.book.genre}
        onChange={(e) => handleChange(e.target.value)}
      >
        {presets.map((preset) => (
          <option key={preset.id} value={preset.id}>
            {preset.label}
          </option>
        ))}
        <option value={CUSTOM_GENRE_ID}>自定义样式…</option>
      </select>
      {state.book.genre === CUSTOM_GENRE_ID && state.book.chapterStyleId && (
        <span className="label-text-alt text-xs text-base-content/60">
          已选目录：{state.book.chapterStyleId}
        </span>
      )}
      {customError && (
        <span className="label-text-alt text-xs text-error">{customError}</span>
      )}
    </label>
  );
}
