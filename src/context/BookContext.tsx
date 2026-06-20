import { createContext, useContext, useReducer, type ReactNode } from "react";
import type {
  BookConfig,
  Chapter,
  ParseProgress,
  WizardStep,
} from "../types";
import { DEFAULT_CHAPTER_REGEX } from "../utils/constants";

interface BookState {
  step: WizardStep;
  sourcePath: string | null;
  regex: string;
  parseProgress: ParseProgress | null;
  book: BookConfig;
}

type BookAction =
  | { type: "SET_STEP"; step: WizardStep }
  | { type: "SET_SOURCE_PATH"; path: string }
  | { type: "SET_REGEX"; regex: string }
  | { type: "SET_PARSE_PROGRESS"; progress: ParseProgress | null }
  | { type: "SET_CHAPTERS"; chapters: Chapter[] }
  | { type: "UPDATE_CHAPTER_TITLE"; id: number; title: string }
  | { type: "UPDATE_BOOK"; book: Partial<BookConfig> };

const initialBook: BookConfig = {
  title: "",
  author: "",
  genre: "fantasy",
  cover: {
    mode: "generate",
    // 位置和字号使用相对比例 (0-1)
    // 预览尺寸 600x900，比例 = 像素值 / 尺寸
    titlePositionRatio: { x: 0.5, y: 0.133 }, // 50%水平居中, 120/900
    authorPositionRatio: { x: 0.5, y: 0.844 }, // 50%水平居中, 760/900
    fontFamily: "Huiwen GangHei",
    titleFontRatio: 0.050,
    authorFontRatio: 0.050,
    titleColor: "#ffffff",
    authorColor: "#ffffff",
  },
  chapters: [],
  source: { type: "local" },
};

const initialState: BookState = {
  step: "import",
  sourcePath: null,
  regex: DEFAULT_CHAPTER_REGEX,
  parseProgress: null,
  book: initialBook,
};

function bookReducer(state: BookState, action: BookAction): BookState {
  switch (action.type) {
    case "SET_STEP":
      return { ...state, step: action.step };
    case "SET_SOURCE_PATH":
      return { ...state, sourcePath: action.path };
    case "SET_REGEX":
      return { ...state, regex: action.regex };
    case "SET_PARSE_PROGRESS":
      return { ...state, parseProgress: action.progress };
    case "SET_CHAPTERS":
      return { ...state, book: { ...state.book, chapters: action.chapters } };
    case "UPDATE_CHAPTER_TITLE":
      return {
        ...state,
        book: {
          ...state.book,
          chapters: state.book.chapters.map((c) =>
            c.id === action.id ? { ...c, title: action.title } : c,
          ),
        },
      };
    case "UPDATE_BOOK":
      return { ...state, book: { ...state.book, ...action.book } };
    default:
      return state;
  }
}

interface BookContextValue {
  state: BookState;
  dispatch: React.Dispatch<BookAction>;
}

const BookContext = createContext<BookContextValue | null>(null);

export function BookProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(bookReducer, initialState);
  return (
    <BookContext.Provider value={{ state, dispatch }}>
      {children}
    </BookContext.Provider>
  );
}

export function useBookContext() {
  const ctx = useContext(BookContext);
  if (!ctx) {
    throw new Error("useBookContext must be used within a BookProvider");
  }
  return ctx;
}
