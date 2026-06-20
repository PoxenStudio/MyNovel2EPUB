// ===== 章节 =====
export interface Chapter {
  id: number;
  title: string;
  startLine: number;
  endLine: number;
  wordCount: number;
}

// ===== 小说类型 =====
export type NovelGenre =
  | "fantasy"
  | "xianxia"
  | "wuxia"
  | "urban_romance"
  | "period_romance"
  | "scifi"
  | "mystery"
  | "horror"
  | "custom";

export interface GenrePreset {
  id: NovelGenre;
  label: string;
  promptKeywords: string;
  chapterStyleId: string;
  previewImagePath: string;
  coverImagePath?: string;
}

// ===== 封面配置 =====
// 位置和字体大小使用相对比例 (0-1)，预览和生成时根据实际尺寸等比例计算
export interface CoverConfig {
  mode: "generate" | "ai_prompt" | "no_text";
  templatePath?: string;
  titlePositionRatio: { x: number; y: number }; // 相对位置 (0-1)
  authorPositionRatio: { x: number; y: number }; // 相对位置 (0-1)
  fontFamily: string;
  titleFontRatio: number; // 相对于封面高度的字号比例 (0-1)
  authorFontRatio: number; // 相对于封面高度的字号比例 (0-1)
  titleColor: string;
  authorColor: string;
  aiPromptText?: string;
}

// ===== 章节样式配置 =====
export interface ChapterStyle {
  id: string;
  name: string;
  headerImages: string[];
  headerOrnament: string;
  footerOrnament: string;
}

// ===== 来源（本地文件 / MyBooks 书库）=====
export type BookSource =
  | { type: "local" }
  | { type: "mybooks"; bookId: number; host: string; hadExistingEpub: boolean }
  | { type: "disabled"; bookId: number; host: string; hadExistingEpub: boolean };

// ===== 完整书籍配置 =====
export interface BookConfig {
  title: string;
  author: string;
  genre: NovelGenre;
  cover: CoverConfig;
  chapterStyleId?: string;
  chapters: Chapter[];
  source: BookSource;
}

// ===== 解析进度 =====
export interface ParseProgress {
  percent: number;
  currentChapter: string;
  totalChapters: number;
}

// ===== 导出进度 =====
export interface ExportProgress {
  phase:
    | "cover"
    | "chapters"
    | "packaging"
    | "checking_format"
    | "deleting_old_format"
    | "uploading";
  percent: number;
  message: string;
}

// ===== MyBooks =====
export interface MyBooksSession {
  host: string;
  username: string;
}

export interface MyBooksBookSummary {
  id: number;
  title: string;
  author: string;
  formats: string[];
}

export type WizardStep = "import" | "config" | "export";
