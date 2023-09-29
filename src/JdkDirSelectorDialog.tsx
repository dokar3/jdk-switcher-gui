import { XMarkIcon } from "@heroicons/react/24/outline";
import { useEffect, useRef, useState } from "react";

export default function JdkDirSelectorDialog({
  open,
  doNotShowAgain,
  onClose,
  onSelectClick,
  onUpdateDoNotShowAgain,
}: {
  open: boolean;
  doNotShowAgain: boolean;
  onClose: () => void;
  onSelectClick: () => void;
  onUpdateDoNotShowAgain: (value: boolean) => void;
}) {
  const ref = useRef<HTMLDialogElement>(null);

  const [visible, setVisible] = useState(false);

  const keyDirClassName = "px-1 font-bold";

  useEffect(() => {
    const dialog = ref.current;
    if (dialog == null) {
      return;
    }
    if (open) {
      dialog.showModal();
      setVisible(true);
      const clickListener = (ev: MouseEvent) => {
        const bounds = dialog.getBoundingClientRect();
        const x = ev.x;
        const y = ev.y;
        if (
          x < bounds.left ||
          x > bounds.right ||
          y < bounds.top ||
          y > bounds.bottom
        ) {
          // Backdrop clicked
          onClose();
        }
      };
      dialog.addEventListener("click", clickListener);
      return () => {
        dialog.removeEventListener("click", clickListener);
      };
    } else {
      dialog.close();
      setVisible(false);
    }
  }, [open, onClose]);

  return (
    <dialog
      ref={ref}
      className={
        "w-full max-w-[500px] relative p-4 rounded-xl backdrop:bg-black/30 bg-white dark:bg-gray-700 text-gray-900 dark:text-white transition-[transform,opacity] duration-300 ease-bouncy " +
        (visible
          ? "translate-y-0 scale-100 opacity-100"
          : "translate-y-48 scale-75 opacity-0")
      }
    >
      <XMarkIcon
        className="stroke-gray-400 hover:stroke-red-500 w-6 h-6 absolute right-2 top-2"
        onClick={onClose}
      />

      <p className="mt-2 text-xl font-bold">Select your JDK(s) directory</p>

      <p className="mt-2">
        Scanning is faster if your picked folder is closer to the{" "}
        <span className={keyDirClassName + " font-mono text-green-500"}>
          bin
        </span>{" "}
        dir.
      </p>

      <p className="mt-2 px-2 font-mono bg-gray-400/30 rounded">
        /.../<span className={keyDirClassName + " text-orange-500"}>data</span>/
        <span className={keyDirClassName + " text-yellow-500"}>jdks</span>/
        <span className={keyDirClassName + " text-lime-500"}>open-jdk-21</span>/
        <span className={keyDirClassName + " text-green-500"}>bin</span>
        /java.exe
      </p>

      <div className="mt-4 flex justify-between">
        <div
          className="flex items-center cursor-pointer"
          onClick={() => onUpdateDoNotShowAgain(!doNotShowAgain)}
        >
          <input
            type="checkbox"
            className="mr-2 accent-violet-500"
            checked={doNotShowAgain}
            onChange={(e) => onUpdateDoNotShowAgain(e.target.checked)}
          />
          <p>Don't show this again</p>
        </div>
        <button
          className="shrink-0 bg-violet-500 hover:bg-violet-600 text-white px-4 py-1 rounded"
          onClick={onSelectClick}
        >
          Select
        </button>
      </div>
    </dialog>
  );
}
