import { useEffect } from "react";
import { listen, type EventCallback } from "@tauri-apps/api/event";

export function useTauriEvent<T>(eventName: string, handler: EventCallback<T>) {
  useEffect(() => {
    const unlisten = listen<T>(eventName, handler);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName, handler]);
}
