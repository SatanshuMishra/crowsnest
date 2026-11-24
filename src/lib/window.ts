import { getCurrentWindow } from "@tauri-apps/api/window";

export type WindowGesture = "drag" | "toggleMaximize" | null;

const PRIMARY_BUTTON = 0;
const CONTROLS = "button, a, input, select, textarea, summary, [role='button'], [tabindex]:not([tabindex='-1'])";

export function windowGesture(button: number, clicks: number, onControl: boolean): WindowGesture {
  if (button !== PRIMARY_BUTTON || onControl) {
    return null;
  }
  return clicks === 2 ? "toggleMaximize" : "drag";
}

function ignoreFailure(action: Promise<void>): void {
  action.catch(() => undefined);
}

function onMouseDown(event: MouseEvent): void {
  const onControl = event.target instanceof Element && event.target.closest(CONTROLS) !== null;
  const gesture = windowGesture(event.button, event.detail, onControl);
  if (gesture === "drag") {
    ignoreFailure(getCurrentWindow().startDragging());
  } else if (gesture === "toggleMaximize") {
    ignoreFailure(getCurrentWindow().toggleMaximize());
  }
}

export function windowDrag(node: HTMLElement) {
  node.addEventListener("mousedown", onMouseDown);
  return {
    destroy() {
      node.removeEventListener("mousedown", onMouseDown);
    },
  };
}

export const windowFrame = {
  minimize: () => ignoreFailure(getCurrentWindow().minimize()),
  toggleMaximize: () => ignoreFailure(getCurrentWindow().toggleMaximize()),
  close: () => ignoreFailure(getCurrentWindow().close()),
  isMaximized: () => getCurrentWindow().isMaximized(),
  onResized: (handler: () => void) => getCurrentWindow().onResized(handler),
};
