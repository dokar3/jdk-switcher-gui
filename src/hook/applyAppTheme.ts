import { useEffect, useState } from "react";
import { AppTheme } from "../model/AppUiState";

export default function applyAppTheme(theme: AppTheme) {
  const [colorSchemeChanges, setColorSchemeChanges] = useState(0);

  useEffect(() => {
    const dark = () => window.document.body.classList.add("dark");
    const light = () => window.document.body.classList.remove("dark");
    switch (theme) {
      case AppTheme.Dark: {
        dark();
        break;
      }
      case AppTheme.Light: {
        light();
        break;
      }
      case AppTheme.Default: {
        const matchMedia = window.matchMedia;
        if (matchMedia == null) {
          break;
        }
        const darkQueryList = matchMedia("(prefers-color-scheme: dark)");
        if (darkQueryList.matches) {
          dark();
        } else {
          light();
        }
        const listener = () => setColorSchemeChanges((prev) => prev + 1);
        darkQueryList.addEventListener("change", listener);
        return () => {
          darkQueryList.removeEventListener("change", listener);
        };
      }
    }
  }, [theme, colorSchemeChanges]);
}
