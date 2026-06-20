import { useEffect, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

interface Props {
  selectedPath?: string;
  onSelect: (path: string) => void;
}

export function CoverTemplateCarousel({ selectedPath, onSelect }: Props) {
  const [templates, setTemplates] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<string[]>("get_cover_templates")
      .then(setTemplates)
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return <div className="alert alert-error">底图模板加载失败：{error}</div>;
  }

  return (
    <div className="carousel carousel-center w-full max-w-xl gap-3 rounded-box bg-base-200 p-2">
      {templates.map((path) => (
        <button
          key={path}
          type="button"
          onClick={() => onSelect(path)}
          className={`carousel-item h-32 w-24 overflow-hidden rounded-md border-2 ${
            selectedPath === path ? "border-primary" : "border-transparent"
          }`}
        >
          <img
            src={convertFileSrc(path)}
            alt="封面底图模板"
            className="h-full w-full object-cover"
          />
        </button>
      ))}
    </div>
  );
}
