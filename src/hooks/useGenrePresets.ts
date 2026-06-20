import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GenrePreset } from "../types";

export function useGenrePresets() {
  const [presets, setPresets] = useState<GenrePreset[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    invoke<GenrePreset[]>("get_genre_presets")
      .then((result) => {
        if (!cancelled) setPresets(result);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return { presets, error };
}
