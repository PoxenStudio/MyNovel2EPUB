import { useBookContext } from "../../context/BookContext";
import { useGenrePresets } from "../../hooks/useGenrePresets";
import { AiPromptPanel } from "./AiPromptPanel";
import type { CoverConfig as CoverConfigData } from "../../types";

const FONT_OPTIONS = [
  "Huiwen GangHei",
  "Huiwen FangSong",
  "QingLiu ShuShi Ti",
  "HengShan MaoBi CaoShu",
  "SanJi PoMo Ti",
  "YanShi ChunFeng Kai",
  "Slidefu Regular",
];
const MODE_OPTIONS = [
  { value: "generate" as const, label: "合成封面" },
  { value: "ai_prompt" as const, label: "AI 提示词" },
  { value: "no_text" as const, label: "无文字" },
];
const COLOR_OPTIONS = [
  { value: "#000000", label: "黑色" },
  { value: "#ffffff", label: "白色" },
  { value: "#ff0000", label: "红色" },
  { value: "#0000ff", label: "蓝色" },
  { value: "#ffff00", label: "黄色" },
];

export function CoverConfig() {
  const { state, dispatch } = useBookContext();
  const { presets } = useGenrePresets();
  const genrePreset = presets.find((p) => p.id === state.book.genre);
  const noText = state.book.cover.mode === "no_text";

  function updateCover(patch: Partial<CoverConfigData>) {
    dispatch({
      type: "UPDATE_BOOK",
      book: { cover: { ...state.book.cover, ...patch } },
    });
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-3 gap-4">
        <div className="flex flex-row gap-2">
          <label className="form-control flex-1">
            <span className="label-text text-xs">封面模式</span>
            <select
              className="select select-bordered select-sm"
              value={state.book.cover.mode}
              onChange={(e) =>
                updateCover({ mode: e.target.value as CoverConfigData["mode"] })
              }
            >
              {MODE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </label>

          <label className="form-control flex-[2]">
            <span className="label-text text-xs">字体</span>
            <select
              className="select select-bordered select-sm"
              value={state.book.cover.fontFamily}
              onChange={(e) => updateCover({ fontFamily: e.target.value })}
              disabled={noText}
            >
              <option value="" disabled>
                选择字体
              </option>
              {FONT_OPTIONS.map((font) => (
                <option key={font} value={font}>
                  {font}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="form-control">
          <span className="label-text text-xs">标题中心位置 X/Y / 字号% / 颜色</span>
          <div className="flex gap-1">
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="X%"
              min={0}
              max={100}
              value={Math.round(state.book.cover.titlePositionRatio.x * 100)}
              onChange={(e) =>
                updateCover({
                  titlePositionRatio: {
                    ...state.book.cover.titlePositionRatio,
                    x: Number(e.target.value) / 100,
                  },
                })
              }
              disabled={noText}
            />
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="Y%"
              min={0}
              max={100}
              value={Math.round(state.book.cover.titlePositionRatio.y * 100)}
              onChange={(e) =>
                updateCover({
                  titlePositionRatio: {
                    ...state.book.cover.titlePositionRatio,
                    y: Number(e.target.value) / 100,
                  },
                })
              }
              disabled={noText}
            />
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="字号%"
              min={1}
              max={100}
              value={Math.round(state.book.cover.titleFontRatio * 100)}
              onChange={(e) =>
                updateCover({ titleFontRatio: Number(e.target.value) / 100 })
              }
              disabled={noText}
            />
            <select
              className="select select-bordered select-sm w-20"
              value={state.book.cover.titleColor || "#ffffff"}
              onChange={(e) => updateCover({ titleColor: e.target.value })}
              disabled={noText}
            >
              {COLOR_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="form-control">
          <span className="label-text text-xs">作者中心位置 X/Y / 字号% / 颜色</span>
          <div className="flex gap-1">
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="X%"
              min={0}
              max={100}
              value={Math.round(state.book.cover.authorPositionRatio.x * 100)}
              onChange={(e) =>
                updateCover({
                  authorPositionRatio: {
                    ...state.book.cover.authorPositionRatio,
                    x: Number(e.target.value) / 100,
                  },
                })
              }
              disabled={noText}
            />
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="Y%"
              min={0}
              max={100}
              value={Math.round(state.book.cover.authorPositionRatio.y * 100)}
              onChange={(e) =>
                updateCover({
                  authorPositionRatio: {
                    ...state.book.cover.authorPositionRatio,
                    y: Number(e.target.value) / 100,
                  },
                })
              }
              disabled={noText}
            />
            <input
              type="number"
              className="input input-bordered input-sm w-16"
              placeholder="字号%"
              min={1}
              max={100}
              value={Math.round(state.book.cover.authorFontRatio * 100)}
              onChange={(e) =>
                updateCover({ authorFontRatio: Number(e.target.value) / 100 })
              }
              disabled={noText}
            />
            <select
              className="select select-bordered select-sm w-20"
              value={state.book.cover.authorColor || "#ffffff"}
              onChange={(e) => updateCover({ authorColor: e.target.value })}
              disabled={noText}
            >
              {COLOR_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {state.book.cover.mode === "ai_prompt" && (
        <AiPromptPanel
          title={state.book.title}
          author={state.book.author}
          genrePreset={genrePreset}
          onTemplateUploaded={(templatePath) => updateCover({ templatePath })}
        />
      )}
    </div>
  );
}
