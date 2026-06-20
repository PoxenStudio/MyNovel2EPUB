import type { GenrePreset } from "../types";

export function buildAiPrompt(
  title: string,
  author: string,
  genre: GenrePreset,
): string {
  return [
    `为小说《${title}》（作者：${author}）创作的书籍封面插画。`,
    genre.promptKeywords,
    `竖版构图，2:3比例，电影级光影效果，高度精细的数字绘画风格。`,
    `在上三分之一处留出清晰的空白区域用于放置书名，底部附近留出空间用于放置作者名——图像中不要渲染任何文字或字母。`,
  ].join(" ");
}
