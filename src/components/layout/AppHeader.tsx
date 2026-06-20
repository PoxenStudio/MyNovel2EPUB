import { useEffect, useState } from "react";
import { AboutDialog } from "./AboutDialog";

const THEME_STORAGE_KEY = "mynovel2epub-theme";

function getInitialTheme(): string {
  return localStorage.getItem(THEME_STORAGE_KEY) ?? "dark";
}

export function AppHeader() {
  const [theme, setTheme] = useState(getInitialTheme);
  const [showAbout, setShowAbout] = useState(false);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  function toggleTheme() {
    setTheme((t) => (t === "dark" ? "light" : "dark"));
  }

  return (
    <header className="navbar bg-base-200 px-6">
      <div className="flex-1">
        <span className="text-xl font-semibold">小说Text文件转EPUB电子书</span> <br/>
        <span className="text-sm text-base-content/70">
          可以搭配 (<a href="https://mybooks.top" target="_blank" rel="noopener noreferrer">私有书库MyBooks</a>) 使用
        </span>
      </div>
      <button
        type="button"
        className="btn btn-ghost btn-circle"
        onClick={toggleTheme}
        aria-label="切换主题"
      >
        {theme === "dark" ? "☀️" : "🌙"}
      </button>
      <button
        type="button"
        className="btn btn-ghost btn-circle"
        onClick={() => setShowAbout(true)}
        aria-label="关于"
      >
        <svg viewBox="0 0 24 24" className="h-5 w-5 fill-current">
          <path d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z" />
        </svg>
      </button>
      {showAbout && <AboutDialog onClose={() => setShowAbout(false)} />}
    </header>
  );
}
