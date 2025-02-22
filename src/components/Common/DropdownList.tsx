import { useEffect, useRef, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";

import { metaOrCtrlKey, isMetaOrCtrlKey } from "@/utils/keyboardUtils";

interface DropdownListProps {
  selected: (item: any) => void;
  suggests: any[];
  isSearchComplete: boolean;
}

function DropdownList({ selected, suggests }: DropdownListProps) {
  const [selectedItem, setSelectedItem] = useState<number | null>(null);
  const [showIndex, setShowIndex] = useState<boolean>(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);

  const handleOpenURL = async (url: string) => {
    if (!url) return;
    try {
      if (isTauri()) {
        await open(url);
        // console.log("URL opened in default browser");
      }
    } catch (error) {
      console.error("Failed to open URL:", error);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // console.log(
    //   "handleKeyDown",
    //   e.key,
    //   showIndex,
    //   e.key >= "0" && e.key <= "9" && showIndex
    // );
    if (!suggests.length) return;

    if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedItem((prev) =>
        prev === null || prev === 0 ? suggests.length - 1 : prev - 1
      );
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedItem((prev) =>
        prev === null || prev === suggests.length - 1 ? 0 : prev + 1
      );
    } else if (e.key === metaOrCtrlKey()) {
      e.preventDefault();
      setShowIndex(true);
    }

    if (e.key === "Enter" && selectedItem !== null) {
      // console.log("Enter key pressed", selectedItem);
      const item = suggests[selectedItem];
      if (item?.url) {
        handleOpenURL(item?.url);
      } else {
        selected(item);
      }
    }

    if (e.key >= "0" && e.key <= "9" && showIndex) {
      // console.log(`number ${e.key}`);
      const item = suggests[parseInt(e.key, 10)];
      if (item?.url) {
        handleOpenURL(item?.url);
      } else {
        selected(item);
      }
    }
  };

  const handleKeyUp = (e: KeyboardEvent) => {
    // console.log("handleKeyUp", e.key);
    if (!suggests.length) return;

    if (!isMetaOrCtrlKey(e)) {
      setShowIndex(false);
    }
  };

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [showIndex, selectedItem, suggests]);

  // useEffect(() => {
  //   if (suggests.length > 0) {
  //     setSelectedItem(0);
  //   }
  // }, [JSON.stringify(suggests)]);

  useEffect(() => {
    if (selectedItem !== null && itemRefs.current[selectedItem]) {
      itemRefs.current[selectedItem]?.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, [selectedItem]);

  return (
    <div
      ref={containerRef}
      data-tauri-drag-region
      className="max-h-[458px] w-full p-2 flex flex-col rounded-xl overflow-y-auto overflow-hidden custom-scrollbar focus:outline-none"
      tabIndex={0}
    >
      <div className="p-2 text-xs text-[#999] dark:text-[#666]">Results</div>
      {suggests?.map((item, index) => {
        const isSelected = selectedItem === index;
        return (
          <div
            key={item._id}
            ref={(el) => (itemRefs.current[index] = el)}
            onMouseEnter={() => setSelectedItem(index)}
            onClick={() => {
              if (item?.url) {
                handleOpenURL(item?.url);
              } else {
                selected(item);
              }
            }}
            className={`w-full px-2 py-2.5 text-sm flex items-center justify-between rounded-lg transition-colors ${
              isSelected
                ? "bg-[rgba(0,0,0,0.1)] dark:bg-[rgba(255,255,255,0.1)] hover:bg-[rgba(0,0,0,0.1)] dark:hover:bg-[rgba(255,255,255,0.1)]"
                : ""
            }`}
          >
            <div className="flex gap-2 items-center">
              <img className="w-5 h-5" src={item?.icon} alt="icon" />
              <span className="text-[#333] dark:text-[#d8d8d8] truncate w-80 text-left">
                {item?.title}
              </span>
            </div>
            <div className="flex gap-2 items-center relative">
              <span className="text-sm  text-[#666] dark:text-[#666] truncate w-52 text-right">
                {item?.source}
              </span>
              {showIndex && index < 10 ? (
                <div
                  className={`absolute right-0 w-4 h-4 flex items-center justify-center font-normal text-xs text-[#333] leading-[14px] bg-[#ccc] dark:bg-[#6B6B6B] shadow-[-6px_0px_6px_2px_#e6e6e6] dark:shadow-[-6px_0px_6px_2px_#000] rounded-md`}
                >
                  {index}
                </div>
              ) : null}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default DropdownList;
