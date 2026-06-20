import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { buildAiPrompt } from "../../utils/aiPrompt";
import type { GenrePreset } from "../../types";

interface Props {
  title: string;
  author: string;
  genrePreset: GenrePreset | undefined;
  onTemplateUploaded: (path: string) => void;
}

export function AiPromptPanel({ title, author, genrePreset, onTemplateUploaded }: Props) {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const prompt = useMemo(() => {
    if (!genrePreset) return "";
    return buildAiPrompt(title, author, genrePreset);
  }, [title, author, genrePreset]);

  async function handleCopy() {
    await navigator.clipboard.writeText(prompt);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="flex w-full flex-col items-center gap-2">
      <textarea
        readOnly
        className="textarea textarea-bordered h-20 w-full text-sm leading-relaxed"
        value={prompt}
      />
      <div className="flex gap-2">
        <button type="button" className="btn btn-sm" onClick={handleCopy}>
          {copied ? "✅ 已复制" : "📋 一键复制"}
        </button>
      </div>
      {error && <div className="alert alert-error text-sm">{error}</div>}
    </div>
  );
}
