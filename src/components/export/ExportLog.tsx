import { useEffect, useRef } from "react";
import type { ExportProgress } from "../../types";

interface Props {
  entries: ExportProgress[];
}

const PHASE_LABELS: Record<string, string> = {
  cover: "封面",
  chapters: "章节",
  packaging: "打包",
  checking_format: "检查书库格式",
  deleting_old_format: "删除旧格式",
  uploading: "上传",
};

const MAX_ENTRIES = 50;

export function ExportLog({ entries }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [entries]);

  if (entries.length === 0) return null;

  const displayedEntries = entries.slice(-MAX_ENTRIES);

  return (
    <div
      ref={containerRef}
      className="mockup-code w-full max-w-xl text-xs overflow-y-auto max-h-64"
    >
      {displayedEntries.map((entry, index) => (
        <pre key={index} data-prefix=">">
          <code>
            [{PHASE_LABELS[entry.phase] ?? entry.phase} {entry.percent}%] {entry.message}
          </code>
        </pre>
      ))}
    </div>
  );
}
