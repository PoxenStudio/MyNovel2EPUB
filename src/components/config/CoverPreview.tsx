import { useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { CoverConfig } from "../../types";

interface Props {
  title: string;
  author: string;
  cover: CoverConfig;
}

const CANVAS_WIDTH = 600;
const CANVAS_HEIGHT = 900;

// 已加载的字体缓存
const loadedFonts = new Set<string>();

// 注册字体：复用 Rust 端合成封面所用的同一份字体文件（src-tauri/resources/fonts），
// 避免在 public/fonts 下重复存放字体二进制。
async function loadFont(fontFamily: string): Promise<void> {
  // 检查是否已加载
  if (loadedFonts.has(fontFamily)) {
    console.log(`Font ${fontFamily} already loaded`);
    // 字体已加载，等待其 ready 状态以确保 canvas 可用
    await document.fonts.ready;
    return;
  }

  try {
    const fontPath = await invoke<string>("get_font_resource_path", { fontFamily });
    const fontUrl = convertFileSrc(fontPath);
    console.log(`Loading font: ${fontFamily} from ${fontUrl}`);
    const font = new FontFace(fontFamily, `url(${fontUrl})`);
    await font.load();
    document.fonts.add(font);
    loadedFonts.add(fontFamily);
    console.log(`Font ${fontFamily} loaded successfully`);
  } catch (e) {
    console.error(`Failed to load font ${fontFamily}:`, e);
  }
}

export function CoverPreview({ title, author, cover }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  // null 表示未初始化，需要根据字体是否已缓存来决定何时绘制
  const [loadedFontKey, setLoadedFontKey] = useState<string | null>(null);

  useEffect(() => {
    const targetFont = cover.fontFamily;
    const isCached = loadedFonts.has(targetFont);

    if (isCached) {
      // 字体已缓存，立即更新 key 并绘制
      setLoadedFontKey(targetFont);
    } else {
      // 字体未缓存，先清空 key 暂停绘制，等待加载完成
      setLoadedFontKey(null);
      loadFont(targetFont).then(() => {
        setLoadedFontKey(targetFont);
      });
    }
  }, [cover.fontFamily]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // 未初始化或字体不匹配时不绘制
    if (loadedFontKey === null || loadedFontKey !== cover.fontFamily) return;

    const textCtx = ctx;

    // 测量指定字号下文本的像素宽度
    function measureWidth(text: string, fontSize: number): number {
      textCtx.font = `${fontSize}px "${cover.fontFamily}"`;
      return textCtx.measureText(text).width;
    }

    // 若文本宽度超出 maxWidth，则从中间向外查找一个可用的拆分点，分成两行
    function splitTitleForWidth(text: string, fontSize: number, maxWidth: number): string[] {
      if (text.length <= 1 || measureWidth(text, fontSize) <= maxWidth) return [text];

      const mid = Math.floor(text.length / 2);
      for (let offset = 0; offset < text.length; offset++) {
        for (const idx of [mid - offset, mid + offset]) {
          if (idx < 1 || idx >= text.length) continue;
          const left = text.slice(0, idx);
          const right = text.slice(idx);
          if (measureWidth(left, fontSize) <= maxWidth && measureWidth(right, fontSize) <= maxWidth) {
            return [left, right];
          }
        }
      }
      return [text.slice(0, mid), text.slice(mid)];
    }

    function drawText(
      titleX: number,
      titleY: number,
      titleFontSize: number,
      authorX: number,
      authorY: number,
      authorFontSize: number,
    ) {
      textCtx.textBaseline = "middle";  // 改为中心对齐
      textCtx.textAlign = "center";
      textCtx.font = `${titleFontSize}px "${cover.fontFamily}"`;
      textCtx.fillStyle = cover.titleColor || "#ffffff";

      // 标题过长时换行显示两行，始终保持居中
      const marginPx = CANVAS_WIDTH * 0.04;
      const halfAvail = Math.min(titleX, CANVAS_WIDTH - titleX) - marginPx;
      const maxTitleWidth = halfAvail > 0 ? halfAvail * 2 : CANVAS_WIDTH * 0.84;
      const titleLines = splitTitleForWidth(title, titleFontSize, maxTitleWidth);
      const lineHeight = titleFontSize * 1.15;
      titleLines.forEach((line, i) => {
        const centerY = titleY - ((titleLines.length - 1) * lineHeight) / 2 + i * lineHeight;
        textCtx.fillText(line, titleX, centerY);
      });

      textCtx.font = `${authorFontSize}px "${cover.fontFamily}"`;
      textCtx.fillStyle = cover.authorColor || "#ffffff";
      textCtx.fillText(author, authorX, authorY);
    }

    function draw() {
      textCtx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
      textCtx.fillStyle = "#1f2937";
      textCtx.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

      // 根据比例计算像素位置（基于预览尺寸）
      const titleX = cover.titlePositionRatio.x * CANVAS_WIDTH;
      const titleY = cover.titlePositionRatio.y * CANVAS_HEIGHT;
      const titleFontSize = cover.titleFontRatio * CANVAS_HEIGHT;
      const authorX = cover.authorPositionRatio.x * CANVAS_WIDTH;
      const authorY = cover.authorPositionRatio.y * CANVAS_HEIGHT;
      const authorFontSize = cover.authorFontRatio * CANVAS_HEIGHT;

      const noText = cover.mode === "no_text";

      if (!cover.templatePath) {
        if (!noText) {
          drawText(titleX, titleY, titleFontSize, authorX, authorY, authorFontSize);
        }
        return;
      }

      const image = new Image();
      image.onload = () => {
        // 使用图片实际尺寸绘制
        textCtx!.drawImage(image, 0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
        if (!noText) {
          drawText(titleX, titleY, titleFontSize, authorX, authorY, authorFontSize);
        }
      };
      image.src = convertFileSrc(cover.templatePath);
    }

    draw();
  }, [title, author, cover, loadedFontKey]);

  return (
    <canvas
      ref={canvasRef}
      width={CANVAS_WIDTH}
      height={CANVAS_HEIGHT}
      className="h-[540px] w-[360px] rounded-box border border-base-content/10"
    />
  );
}
