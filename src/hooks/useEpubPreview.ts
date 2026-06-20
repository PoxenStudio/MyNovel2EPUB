import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Chapter } from "../types";

interface Params {
  sourcePath: string | null;
  chapter: Chapter | null;
  chapterIndex: number;
  bookTitle: string;
  chapterStyleId: string | undefined;
}

export function useEpubPreview({
  sourcePath,
  chapter,
  chapterIndex,
  bookTitle,
  chapterStyleId,
}: Params) {
  const [epubData, setEpubData] = useState<ArrayBuffer | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!sourcePath || !chapter || !chapterStyleId) {
      setEpubData(null);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    invoke<number[]>("generate_preview_epub", {
      sourcePath,
      chapter: {
        title: chapter.title,
        startLine: chapter.startLine,
        endLine: chapter.endLine,
      },
      bookTitle: bookTitle || "未命名书籍",
      chapterStyleId,
      chapterIndex,
    })
      .then((bytes) => {
        if (cancelled) return;
        setEpubData(new Uint8Array(bytes).buffer);
      })
      .catch((e) => {
        if (cancelled) return;
        setError(String(e));
        setEpubData(null);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [sourcePath, chapter, chapterIndex, bookTitle, chapterStyleId]);

  return { epubData, isLoading, error };
}
