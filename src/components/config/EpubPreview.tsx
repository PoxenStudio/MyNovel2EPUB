import { useEffect, useRef, useState } from "react";
import Epub from "epubjs";

interface Props {
  epubData: ArrayBuffer | null;
  isLoading: boolean;
  error: string | null;
}

export function EpubPreview({ epubData, isLoading, error }: Props) {
  const viewerRef = useRef<HTMLDivElement>(null);
  const [renderError, setRenderError] = useState<string | null>(null);

  useEffect(() => {
    if (!epubData || !viewerRef.current) return;
    setRenderError(null);

    const book = Epub(epubData);
    const rendition = book.renderTo(viewerRef.current, {
      width: 360,
      height: 540,
      spread: "none",
    });
    rendition.display().catch((e: unknown) => setRenderError(String(e)));

    return () => {
      book.destroy();
    };
  }, [epubData]);

  return (
    <div className="flex flex-col items-center gap-2">
      <div
        ref={viewerRef}
        className="h-[540px] w-[360px] rounded-box border border-base-content/10 bg-white"
      />
      {isLoading && <span className="loading loading-spinner loading-sm" />}
      {(error || renderError) && (
        <div className="alert alert-error w-full max-w-xs text-sm">
          {error ?? renderError}
        </div>
      )}
    </div>
  );
}
