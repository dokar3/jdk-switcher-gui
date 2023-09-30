import {
  ComputerDesktopIcon,
  CubeTransparentIcon,
  CursorArrowRippleIcon,
  ExclamationTriangleIcon,
  FolderOpenIcon,
  InformationCircleIcon,
  MoonIcon,
  PlusIcon,
  SunIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { Toaster, toast } from "sonner";
import "./App.css";
import JdkDirSelectorDialog from "./JdkDirSelectorDialog";
import CircularLoader from "./component/CircularLoader";
import useTauriEvents from "./hook/useTauriEvents";
import { AppTheme, AppUiState } from "./model/AppUiState";
import { Jdk } from "./model/Jdk";
import applyAppTheme from "./hook/applyAppTheme";

enum ToastDuration {
  Short = 1500,
  Normal = 2500,
  Long = 5000,
  Infinity = 999_999_999,
}

function App() {
  const uiState = useTauriEvents<AppUiState>("ui-state-stream", {
    jdks: [],
    settings: { theme: AppTheme.Unknown, skip_dir_selection_hint: false },
  });

  const theme = uiState.settings.theme;

  const [isLoadingJdks, setLoadingJdks] = useState(true);

  const [operatingMessage, setOperatingMessage] = useState<string | null>(null);

  const [isShowJdkDirSelectorDialog, setShowJdkDirSelectorDialog] =
    useState(false);

  const [skipDirSelectionHint, setSkipDirSelectionHint] = useState(false);

  const [currentJdk, jdks] = useMemo(() => {
    const curr = uiState.jdks.find((item) => item.is_current);
    return [curr, uiState.jdks.filter((item) => !item.is_current)];
  }, [uiState.jdks]);

  const toggleTheme = async () => {
    let nextTheme: AppTheme;
    switch (theme) {
      case AppTheme.Dark: {
        nextTheme = AppTheme.Light;
        break;
      }
      case AppTheme.Light: {
        nextTheme = AppTheme.Default;
        break;
      }
      default: {
        nextTheme = AppTheme.Dark;
        break;
      }
    }
    await invoke("update_app_theme", { theme: nextTheme });
  };

  const removeJdk = (jdk: Jdk) => {
    const name = `${jdk.name} ${jdk.version}`;
    setOperatingMessage(`Removing JDK '${name}'`);
    invoke("remove_jdk_by_path", { path: jdk.path })
      .then(() => {
        toast.success(`Removed JDK '${name}'`, {
          duration: ToastDuration.Short,
        });
      })
      .catch((e) => {
        toast.error(`Failed to remove JDK '${name}', error: ${e}`, {
          duration: ToastDuration.Infinity,
        });
      })
      .finally(() => setOperatingMessage(null));
  };

  const switchToJdk = (jdk: Jdk) => {
    setOperatingMessage(`Switching to '${jdk.name} ${jdk.version}'`);
    invoke("switch_to_jdk", { jdk: jdk })
      .then(() => {
        const message = `Switched to JDK '${jdk.name} ${jdk.version}'`;
        toast.success(message, { duration: ToastDuration.Long });
        console.log(message);
      })
      .catch((e) => {
        const message = `Cannot switch to JDK '${jdk.name} ${jdk.version}', error: ${e}`;
        console.error(message);
        toast.error(message, { duration: ToastDuration.Infinity });
      })
      .finally(() => {
        setOperatingMessage(null);
      });
  };

  const selectJdkDir = async () => {
    await invoke("update_skip_dir_selection_hint", {
      value: skipDirSelectionHint,
    });
    const dir = await open({
      directory: true,
    });
    if (dir != null) {
      setOperatingMessage("Looking for JDK(s)...");
      invoke<number>("add_jdks_from_dir", { dir: dir })
        .then((count) => {
          const message = `Added ${count} JDK(s).`;
          toast.success(message, { duration: ToastDuration.Normal });
        })
        .catch((e) => {
          const message = "Failed to add jdks: " + e;
          console.error(message);
          toast.error(message, { duration: ToastDuration.Infinity });
        })
        .finally(() => setOperatingMessage(null));
    }
  };

  useEffect(() => {
    let cancelLoadingTid = -1;

    const setup = async () => {
      setLoadingJdks(true);
      await invoke("load_jdks");
      cancelLoadingTid = setTimeout(() => {
        setLoadingJdks(false);
      }, 100);
      // Blocking call
      await invoke("listen_ui_state_stream");
    };
    setup();

    return () => {
      clearTimeout(cancelLoadingTid);
    };
  }, []);

  applyAppTheme(theme);

  return (
    <div className="w-full min-h-screen text-gray-900 dark:text-white relative bg-white dark:bg-gray-900 select-none">
      <Toaster richColors closeButton />

      <div className="h-screen flex flex-col overflow-hidden">
        <div className="mb-4 px-4 pt-4 flex justify-between items-center shrink-0">
          <h1 className="text-2xl font-bold">JDK Switcher</h1>

          <div className="flex items-center gap-2">
            <div
              className="w-8 h-8 p-[0.3rem] rounded-full hover:bg-gray-100 dark:hover:bg-gray-600 cursor-pointer"
              onClick={openAboutDialog}
            >
              <InformationCircleIcon />
            </div>

            <div
              className="w-8 h-8 p-[0.3rem] rounded-full hover:bg-gray-100 dark:hover:bg-gray-600 cursor-pointer"
              onClick={toggleTheme}
            >
              {theme === AppTheme.Default && <ComputerDesktopIcon />}
              {theme === AppTheme.Light && <SunIcon />}
              {theme === AppTheme.Dark && <MoonIcon />}
            </div>

            <div
              className="w-8 h-8 p-1 rounded-full hover:bg-gray-100 dark:hover:bg-gray-600 cursor-pointer"
              onClick={() => {
                if (uiState.settings.skip_dir_selection_hint !== true) {
                  setShowJdkDirSelectorDialog(true);
                } else {
                  selectJdkDir();
                }
              }}
            >
              <PlusIcon />
            </div>
          </div>
        </div>

        <div className="px-4 overflow-auto">
          {uiState.jdks.length === 0 && isLoadingJdks && (
            <div className="py-12 flex flex-col items-center">
              <CircularLoader className="mb-4" />
              <p>Loading JDKs...</p>
            </div>
          )}

          {uiState.jdks.length === 0 && !isLoadingJdks && (
            <div className="py-12 flex flex-col items-center">
              <CubeTransparentIcon className="w-20 h-20 mb-4 stroke-violet-400" />
              <p className="mb-4 text-2xl font-bold">No JDk found</p>
              <p>
                Click the '+' button on the top right corner to add your JDKs.
              </p>
            </div>
          )}

          {currentJdk != null && (
            <>
              <p className="mb-2">Current</p>
              <JdkList
                list={[currentJdk]}
                className="mb-4"
                onSwitchToJdkClick={() => {}}
                onRemoveJdkClick={() => {}}
              />
            </>
          )}

          {jdks.length > 0 && (
            <>
              <p className="mb-2">Saved</p>
              <JdkList
                list={jdks}
                onSwitchToJdkClick={switchToJdk}
                onRemoveJdkClick={removeJdk}
              />
            </>
          )}
        </div>
      </div>

      {operatingMessage != null && (
        <div
          className="w-screen h-screen absolute left-0 top-0 bg-black/30 z-50 flex items-center justify-center"
          onClick={(e) => {
            e.stopPropagation();
            e.preventDefault();
          }}
        >
          <div className="p-4 bg-white dark:bg-gray-700 rounded-lg overflow-hidden flex items-center">
            <CircularLoader className="mr-4" />
            <p>{operatingMessage}</p>
          </div>
        </div>
      )}

      <JdkDirSelectorDialog
        open={isShowJdkDirSelectorDialog}
        doNotShowAgain={skipDirSelectionHint}
        onClose={() => setShowJdkDirSelectorDialog(false)}
        onSelectClick={() => {
          selectJdkDir();
          setShowJdkDirSelectorDialog(false);
        }}
        onUpdateDoNotShowAgain={setSkipDirSelectionHint}
      />
    </div>
  );
}

function JdkList({
  list,
  className,
  onSwitchToJdkClick,
  onRemoveJdkClick,
}: {
  list: Jdk[];
  className?: string;
  onSwitchToJdkClick: (jdk: Jdk) => void;
  onRemoveJdkClick: (jdk: Jdk) => void;
}) {
  const [expandedJdkPath, setExpandedJdkPath] = useState<string | null>(null);

  return (
    <ul className={className}>
      {list.map((item) => {
        const isCurrent = item.is_current;
        const isValid = item.is_valid;
        const isExpanded = expandedJdkPath === item.path;

        return (
          <li
            key={item.path}
            className={
              "rounded-lg mb-4 p-2 cursor-pointer overflow-hidden " +
              (isCurrent
                ? "bg-violet-500 hover:bg-violet-600 text-white shadow-lg shadow-violet-500/30"
                : "bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700")
            }
            onClick={() => {
              if (isCurrent) {
                return;
              }
              if (isExpanded) {
                setExpandedJdkPath(null);
              } else {
                setExpandedJdkPath(item.path);
              }
            }}
          >
            <div
              className={
                "mb-1 flex items-center " + (isValid ? "" : "opacity-70")
              }
            >
              <p className="mr-2 font-bold">
                {item.name} {item.version}
              </p>
              <span
                className={
                  "px-2 text-sm rounded-full " +
                  (isCurrent ? "bg-white/20" : "bg-violet-500/30")
                }
              >
                {item.arch}
              </span>
            </div>

            <div className="flex items-center">
              <p className={"mr-2 " + (isValid ? "" : "opacity-70")}>
                {item.path}
              </p>
              {isValid && (
                <div
                  className={
                    "w-6 h-6 p-1 rounded-full cursor-pointer " +
                    (isCurrent
                      ? "hover:bg-violet-400"
                      : "hover:bg-gray-300 dark:hover:bg-gray-500")
                  }
                  onClick={(e) => {
                    e.stopPropagation();
                    openFolder(item.path);
                  }}
                >
                  <FolderOpenIcon />
                </div>
              )}
              {!isValid && (
                <div className="w-6 h-6 p-1">
                  <ExclamationTriangleIcon className="stroke-orange-500" />
                </div>
              )}
            </div>

            <div
              className={
                "border-t border-gray-300 dark:border-gray-500 transition-[height,opacity] " +
                (isExpanded && !isCurrent
                  ? "mt-2 pt-2 h-[5.6rem] opacity-100"
                  : "h-0 opacity-0")
              }
            >
              <div
                className={
                  "w-fit h-10 px-2 flex items-center rounded-full " +
                  (isValid ? "hover:bg-violet-500/30" : "opacity-70")
                }
                onClick={(e) => {
                  if (isExpanded && isValid) {
                    e.stopPropagation();
                    onSwitchToJdkClick(item);
                  }
                }}
              >
                <CursorArrowRippleIcon className="w-5 h-5 mr-2" />
                <p>Switch to this JDK</p>
              </div>

              <div
                className="w-fit h-10 px-2 flex items-center rounded-full hover:bg-rose-500/30"
                onClick={(e) => {
                  if (isExpanded) {
                    e.stopPropagation();
                    onRemoveJdkClick(item);
                  }
                }}
              >
                <TrashIcon className="w-5 h-5 mr-2" />
                <p>Remove from list</p>
              </div>
            </div>
          </li>
        );
      })}
    </ul>
  );
}

async function openAboutDialog() {
  await invoke("open_about_dialog");
}

async function openFolder(path: string) {
  await invoke("open_folder", { path: path });
}

export default App;
