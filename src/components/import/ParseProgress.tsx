import type { ParseProgress as ParseProgressData } from "../../types";

interface Props {
  progress: ParseProgressData | null;
}

export function ParseProgress({ progress }: Props) {
  if (!progress) return null;

  return (
    <div className="w-full max-w-xl">
      <progress
        className="progress progress-primary w-full"
        value={progress.percent}
        max={100}
      />
      <p className="text-sm text-base-content/60">
        解析中... {progress.percent}%（当前：{progress.currentChapter || "—"}）
      </p>
    </div>
  );
}
