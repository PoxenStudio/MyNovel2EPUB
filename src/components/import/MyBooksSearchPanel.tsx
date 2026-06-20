import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MyBooksBookSummary } from "../../types";

interface Props {
  onSelect: (book: MyBooksBookSummary) => void;
  onClose: () => void;
}

export function MyBooksSearchPanel({ onSelect, onClose }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<MyBooksBookSummary[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const searchInputRef = React.useRef<HTMLInputElement>(null);

  useEffect(() => {
    searchInputRef.current?.focus();
  }, []);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    setIsSearching(true);
    setError(null);
    try {
      const books = await invoke<MyBooksBookSummary[]>("mybooks_search", { query });
      setResults(books);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSearching(false);
    }
  }

  return (
    <div className="modal modal-open">
      <div className="modal-box max-w-2xl">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-bold">从 MyBooks 选择书籍</h3>
          <button
            type="button"
            className="btn btn-sm btn-error w-8 h-8 flex items-center justify-center p-0"
            onClick={onClose}
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        <form onSubmit={handleSearch} className="mt-4 flex gap-2">
          <input
            ref={searchInputRef}
            type="text"
            className="input input-bordered flex-1"
            placeholder="搜索书名 ..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <button type="submit" className="btn btn-primary" disabled={isSearching}>
            {isSearching ? "搜索中..." : "搜索"}
          </button>
        </form>

        {error && <div className="alert alert-error mt-3 text-sm">{error}</div>}

        <ul className="mt-4 max-h-80 overflow-y-auto">
          {results
            .filter((book) =>
              book.formats.some((format) => format.toLowerCase() === "txt"),
            )
            .map((book) => (
              <li
                key={book.id}
                className="flex items-center justify-between gap-2 border-b border-base-content/10 py-2"
              >
                <div>
                  <p className="font-medium">{book.title}</p>
                  <p className="text-sm text-base-content/60">{book.author}</p>
                  <div className="mt-1 flex gap-1">
                    {book.formats.map((format) => (
                      <span key={format} className="badge badge-sm">
                        {format}
                      </span>
                    ))}
                  </div>
                </div>
                <button
                  type="button"
                  className="btn btn-sm"
                  onClick={() => onSelect(book)}
                >
                  选择
                </button>
              </li>
            ))}
        </ul>
      </div>
    </div>
  );
}
