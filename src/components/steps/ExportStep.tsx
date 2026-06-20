import { useCallback, useState } from "react";
import { useBookContext } from "../../context/BookContext";
import { useTauriEvent } from "../../hooks/useTauriEvent";
import { ExportSummary } from "../export/ExportSummary";
import { ExportButton } from "../export/ExportButton";
import { MyBooksPublishButton } from "../export/MyBooksPublishButton";
import { ExportLog } from "../export/ExportLog";
import type { ExportProgress } from "../../types";

export function ExportStep() {
  const { state, dispatch } = useBookContext();
  const [logEntries, setLogEntries] = useState<ExportProgress[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const source = state.book.source;

  useTauriEvent<ExportProgress>(
    "export-progress",
    useCallback(
      (event) => setLogEntries((entries) => [...entries, event.payload]),
      [],
    ),
  );

  function handleExportStart() {
    setLogEntries([]);
    setError(null);
    setDone(false);
  }

  function handleExportDone(exportError: string | null) {
    setError(exportError);
    setDone(exportError === null);
  }

  return (
    <div className="flex flex-col items-center gap-4 py-10">
      <ExportSummary />

      <div className="flex items-center gap-3">
        <ExportButton onExportStart={handleExportStart} onExportDone={handleExportDone} />
        {source.type === "disabled" && (
          <MyBooksPublishButton
            bookId={source.bookId}
            hadExistingEpub={source.hadExistingEpub}
            onExportStart={handleExportStart}
            onExportDone={handleExportDone}
          />
        )}
      </div>

      {done && (
        <div className="alert alert-success w-full max-w-xl">
          {source.type === "mybooks" ? "EPUB 已成功更新到 MyBooks 书库。" : "EPUB 已成功导出。"}
        </div>
      )}
      {error && <div className="alert alert-error w-full max-w-xl">导出失败：{error}</div>}

      <ExportLog entries={logEntries} />

      <button
        type="button"
        className="btn"
        onClick={() => dispatch({ type: "SET_STEP", step: "config" })}
      >
        ← 上一步
      </button>
    </div>
  );
}
