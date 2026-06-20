import { convertFileSrc } from "@tauri-apps/api/core";
import { useGenrePresets } from "../../hooks/useGenrePresets";
import { useBookContext } from "../../context/BookContext";

export function IllustrationConfig() {
  const { state } = useBookContext();
  const { presets } = useGenrePresets();
  const preset = presets.find((p) => p.id === state.book.genre);

  if (state.book.genre === "custom") {
    return (
      <div className="flex w-full max-w-xl items-center gap-4 rounded-box border border-base-content/10 p-4">
        <div className="text-sm text-base-content/70">
          <p>章节插图使用自定义样式目录：{state.book.chapterStyleId || "未选择"}</p>
          <p>正文排版 CSS 为内置固定样式，不可编辑。</p>
        </div>
      </div>
    );
  }

  if (!preset) return null;

  return (
    <div className="flex w-full max-w-xl items-center gap-4 rounded-box border border-base-content/10 p-4">
      {preset.previewImagePath && (
        <img
          src={convertFileSrc(preset.previewImagePath)}
          alt={`${preset.label} 章节头图预览`}
          className="h-20 w-36 rounded object-cover"
        />
      )}
      <div className="text-sm text-base-content/70">
        <p>
          章节插图已按「{preset.label}」类型自动套用（头图 4 张循环 + 头尾装饰线）。
        </p>
        <p>正文排版 CSS 为内置固定样式，不可编辑。</p>
      </div>
    </div>
  );
}
