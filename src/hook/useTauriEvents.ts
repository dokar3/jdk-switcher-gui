import { getCurrent } from "@tauri-apps/plugin-window";
import { useEffect, useState } from "react";

export default function useTauriEvents<T>(
  eventName: string,
  initialValue: T
): T {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    const listen = async () => {
      return await getCurrent().listen<T>(eventName, (event) =>
        setValue(event.payload)
      );
    };
    const unlistenFnPromise = listen();
    return () => {
      const unlisten = async () => (await unlistenFnPromise)();
      unlisten();
    };
  }, []);

  return value;
}
