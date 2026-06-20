import { useBookContext } from "../../context/BookContext";

export function ExportSummary() {
  const { state } = useBookContext();
  const { book } = state;
  const totalWords = book.chapters.reduce((sum, c) => sum + c.wordCount, 0);

  return (
    <div className="card w-full max-w-xl bg-base-200">
      <div className="card-body gap-1 text-sm">
        <p>
          书名：《{book.title || "未命名"}》　作者：{book.author || "未知"}
        </p>
        <p>
          章节数：{book.chapters.length}　总字数：{totalWords.toLocaleString()}
        </p>
        <p>
          封面：
          {book.cover.mode === "generate"
            ? "合成封面"
            : book.cover.mode === "no_text"
              ? "无文字"
              : "AI 提示词"}
          　排版：内置固定样式
        </p>
        <p>格式：EPUB 3.0　编码：UTF-8</p>
        <p>
          来源：{book.source.type === "mybooks" ? `MyBooks（${book.source.host}）` : "本地文件"}
        </p>
      </div>
    </div>
  );
}
