import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  onFileSelected: (path: string) => void;
}

export function FileDropZone({ onFileSelected }: Props) {
  const [isDragOver, setIsDragOver] = useState(false);

  useEffect(() => {
    const webview = getCurrentWebview();
    const unlistenPromise = webview.onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        setIsDragOver(false);
        const txtPath = event.payload.paths.find((p) =>
          p.toLowerCase().endsWith(".txt"),
        );
        if (txtPath) onFileSelected(txtPath);
      } else if (event.payload.type === "over") {
        setIsDragOver(true);
      } else {
        setIsDragOver(false);
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [onFileSelected]);

  async function handleClick() {
    const path = await open({
      multiple: false,
      filters: [{ name: "TXT 文本", extensions: ["txt"] }],
    });
    if (typeof path === "string") {
      onFileSelected(path);
    }
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleClick}
      onKeyDown={(e) => e.key === "Enter" && handleClick()}
      className={`flex h-40 w-full max-w-xl flex-col items-center justify-center rounded-box border-2 border-dashed transition-colors ${
        isDragOver
          ? "border-primary bg-primary/10"
          : "border-base-content/30 hover:border-primary/60"
      }`}
    >
      <span className="text-3xl">📂</span>
      <p className="mt-2 text-base-content/70">拖拽 TXT 文件至此，或点击选择文件</p>
    </div>
  );
}
