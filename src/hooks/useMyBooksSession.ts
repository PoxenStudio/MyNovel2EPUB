import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MyBooksSession } from "../types";

export function useMyBooksSession() {
  const [session, setSession] = useState<MyBooksSession | null>(null);
  const [isRestoring, setIsRestoring] = useState(true);

  useEffect(() => {
    invoke<MyBooksSession | null>("mybooks_restore_session")
      .then(setSession)
      .finally(() => setIsRestoring(false));
  }, []);

  const login = useCallback(
    async (host: string, username: string, password: string) => {
      const result = await invoke<MyBooksSession>("mybooks_login", {
        host,
        username,
        password,
      });
      setSession(result);
      return result;
    },
    [],
  );

  const logout = useCallback(async () => {
    await invoke("mybooks_logout");
    setSession(null);
  }, []);

  return { session, isRestoring, login, logout };
}
