import { useEffect } from "react";
import { AppTheme } from "../model/AppUiState";

export default function applyAppTheme(theme: AppTheme) {
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
        if (
          window.matchMedia &&
          window.matchMedia("(prefers-color-scheme: dark)").matches
        ) {
          dark();
        } else {
          light();
        }
        break;
      }
    }
  }, [theme]);
}
