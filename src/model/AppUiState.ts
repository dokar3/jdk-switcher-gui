import { Jdk } from "./Jdk";

export enum AppTheme {
  Light = "light",
  Dark = "dark",
  Default = "default",
  Unknown = "Unknown",
}

export type AppSettings = {
  theme: AppTheme;
  skip_dir_selection_hint: boolean;
};

export type AppUiState = {
  settings: AppSettings;
  jdks: Jdk[];
};
