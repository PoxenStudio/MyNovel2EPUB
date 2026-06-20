import type { Chapter } from "../../types";

interface Props {
  chapters: Chapter[];
  onTitleChange: (id: number, title: string) => void;
}

export function ChapterTable({ chapters, onTitleChange }: Props) {
  if (chapters.length === 0) return null;

  const totalWords = chapters.reduce((sum, c) => sum + c.wordCount, 0);

  return (
    <div className="w-full max-w-3xl">
      <div className="max-h-96 overflow-y-auto rounded-box border border-base-content/10">
        <table className="table table-pin-rows">
          <thead>
            <tr>
              <th>#</th>
              <th>章节标题（可编辑）</th>
              <th>起始行</th>
              <th>结束行</th>
              <th>字数</th>
            </tr>
          </thead>
          <tbody>
            {chapters.map((chapter) => (
              <tr key={chapter.id}>
                <td>{chapter.id}</td>
                <td>
                  <input
                    type="text"
                    className="input input-sm input-bordered w-full"
                    value={chapter.title}
                    onChange={(e) => onTitleChange(chapter.id, e.target.value)}
                  />
                </td>
                <td>{chapter.startLine}</td>
                <td>{chapter.endLine}</td>
                <td>{chapter.wordCount.toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="mt-2 text-sm text-base-content/70">
        共 {chapters.length} 章 ｜ 总字数：{totalWords.toLocaleString()}
      </p>
    </div>
  );
}
