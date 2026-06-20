import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMyBooksSession } from "../../hooks/useMyBooksSession";
import { useBookContext } from "../../context/BookContext";
import { MyBooksLoginModal } from "./MyBooksLoginModal";
import { MyBooksSearchPanel } from "./MyBooksSearchPanel";
import type { MyBooksBookSummary } from "../../types";

type Dialog = "none" | "login" | "search";

export function MyBooksEntryButton({ onBookSelected }: { onBookSelected?: (path: string) => void }) {
  const { dispatch } = useBookContext();
  const { session, isRestoring, login } = useMyBooksSession();
  const [dialog, setDialog] = useState<Dialog>("none");
  const [error, setError] = useState<string | null>(null);

  function handleOpen() {
    setError(null);
    setDialog(session ? "search" : "login");
  }

  async function handleLogin(host: string, username: string, password: string) {
    await login(host, username, password);
    setDialog("search");
  }

  async function handleSelectBook(book: MyBooksBookSummary) {
    if (!session) return;
    setError(null);
    try {
      const path = await invoke<string>("mybooks_fetch_txt", { bookId: book.id });
      dispatch({ type: "SET_SOURCE_PATH", path });
      dispatch({
        type: "UPDATE_BOOK",
        book: {
          title: book.title,
          author: book.author,
          source: {
            type: "mybooks",
            bookId: book.id,
            host: session.host,
            hadExistingEpub: book.formats.some(
              (format) => format.toLowerCase() === "epub",
            ),
          },
        },
      });
      setDialog("none");
      // 通知父组件书籍已选择，传递路径确保立即解析
      onBookSelected?.(path);
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <>
      <button
        type="button"
        className="btn btn-outline"
        disabled={isRestoring}
        onClick={handleOpen}
      >
        📚 从 MyBooks 选择书籍处理
      </button>
      {error && <div className="alert alert-error mt-2 text-sm">{error}</div>}

      {dialog === "login" && (
        <MyBooksLoginModal onLogin={handleLogin} onClose={() => setDialog("none")} />
      )}
      {dialog === "search" && (
        <MyBooksSearchPanel onSelect={handleSelectBook} onClose={() => setDialog("none")} />
      )}
    </>
  );
}
