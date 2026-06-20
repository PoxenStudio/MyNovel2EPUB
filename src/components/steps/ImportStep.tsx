import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBookContext } from "../../context/BookContext";
import { FileDropZone } from "../import/FileDropZone";
import { RegexInput } from "../import/RegexInput";
import { ParseProgress } from "../import/ParseProgress";
import { ChapterTable } from "../import/ChapterTable";
import { MyBooksEntryButton } from "../import/MyBooksEntryButton";
import { useTauriEvent } from "../../hooks/useTauriEvent";
import type { Chapter, ParseProgress as ParseProgressData } from "../../types";

function guessTitleAuthorFromFilename(filename: string): { title: string; author: string | null } {
  if (!filename) return { title: filename, author: null };

  let title = filename.trim();
  let author: string | null = null;

  if (title.includes("作者")) {
    const parts = title.split(/作者[:：]/, 2);
    if (parts.length >= 2) {
      title = parts[0].trim();
      author = parts[1].trim();
      // 去除 title 尾部的 (、[、（，、【 四种符号
      title = title.replace(/[\s\(\)\[\]【】（）,,、]+$/, "");
      // 去除 author 首尾的 (、[、（，、【 和 )、】、】、） 四种符号
      author = author.replace(/^[\s\(\)\[\]【】（]+/, "").replace(/[\s\)\]】）]+$/, "");
    }
  }

  // 如果书名以《开头且以》结尾，去除首尾字符
  if (title.startsWith("《") && title.endsWith("》")) {
    title = title.slice(1, -1);
  }

  return { title, author };
}

function getFileNameFromPath(filePath: string): string {
  // 移除文件扩展名
  const name = filePath.replace(/\.[^/.]+$/, "");
  // 获取路径的最后一部分
  const parts = name.split(/[/\\]/);
  return parts[parts.length - 1] || name;
}

export function ImportStep() {
  const { state, dispatch } = useBookContext();
  const [isParsing, setIsParsing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useTauriEvent<ParseProgressData>(
    "parse-progress",
    useCallback(
      (event) =>
        dispatch({ type: "SET_PARSE_PROGRESS", progress: event.payload }),
      [dispatch],
    ),
  );

  function handleFileSelected(path: string) {
    dispatch({ type: "SET_SOURCE_PATH", path });
    dispatch({ type: "UPDATE_BOOK", book: { source: { type: "local" } } });

    // 从文件名推断书名和作者
    const filename = getFileNameFromPath(path);
    const { title, author } = guessTitleAuthorFromFilename(filename);
    dispatch({
      type: "UPDATE_BOOK",
      book: {
        title,
        author: author || "",
      },
    });

    setError(null);
    // 自动开始分析，直接传递路径确保立即执行
    handleParse(path);
  }

  async function handleParse(sourcePath?: string) {
    const path = sourcePath || state.sourcePath;
    if (!path) return;
    setIsParsing(true);
    setError(null);
    dispatch({ type: "SET_PARSE_PROGRESS", progress: null });
    try {
      const chapters = await invoke<Chapter[]>("parse_txt_file", {
        path,
        regex: state.regex,
      });
      if (chapters.length === 0) {
        setError("未匹配到任何章节标题，请检查正则表达式是否符合该文件的章节格式");
      }
      dispatch({ type: "SET_CHAPTERS", chapters });
    } catch (e) {
      setError(String(e));
    } finally {
      setIsParsing(false);
      dispatch({ type: "SET_PARSE_PROGRESS", progress: null });
    }
  }

  const hasChapters = state.book.chapters.length > 0;

  return (
    <div className="flex flex-col items-center gap-6 py-10">
      <div className="flex w-full max-w-xl flex-col items-center gap-3">
        <FileDropZone onFileSelected={handleFileSelected} />
        <MyBooksEntryButton onBookSelected={(path) => handleParse(path)} />
      </div>
      {state.sourcePath && (
        <p className="text-sm text-base-content/60">
          已选择文件：{state.sourcePath}
        </p>
      )}

      <RegexInput
        regex={state.regex}
        onChange={(regex) => dispatch({ type: "SET_REGEX", regex })}
      />

      <div className="flex items-center gap-3">
        <button
          type="button"
          className="btn btn-primary"
          disabled={!state.sourcePath || isParsing}
          onClick={handleParse}
        >
          {isParsing ? "解析中..." : "🔍 开始分析"}
        </button>

        <button
          type="button"
          className="btn btn-primary"
          disabled={!hasChapters}
          onClick={() => dispatch({ type: "SET_STEP", step: "config" })}
        >
          下一步 →
        </button>
      </div>

      {error && <div className="alert alert-error w-full max-w-xl">{error}</div>}

      <ParseProgress progress={state.parseProgress} />

      <ChapterTable
        chapters={state.book.chapters}
        onTitleChange={(id, title) =>
          dispatch({ type: "UPDATE_CHAPTER_TITLE", id, title })
        }
      />
    </div>
  );
}
