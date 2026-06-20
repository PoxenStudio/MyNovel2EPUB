import { DEFAULT_CHAPTER_REGEX } from "../../utils/constants";

interface Props {
  regex: string;
  onChange: (regex: string) => void;
}

export function RegexInput({ regex, onChange }: Props) {
  const isBuiltin = regex === DEFAULT_CHAPTER_REGEX;

  return (
    <div className="flex w-full max-w-xl items-center gap-2">
      <select
        className="select select-bordered w-1/6"
        value={isBuiltin ? "builtin" : "custom"}
        onChange={(e) => {
          if (e.target.value === "builtin") onChange(DEFAULT_CHAPTER_REGEX);
        }}
      >
        <option value="builtin">内置正则</option>
        <option value="custom">自定义</option>
      </select>
      <input
        type="text"
        className="input input-bordered flex-1"
        value={regex}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
