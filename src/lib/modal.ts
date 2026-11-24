const FOCUSABLE = "button, a[href], input, select, textarea, [tabindex]:not([tabindex='-1'])";

function focusables(node: HTMLElement): HTMLElement[] {
  return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (item) => !item.matches(":disabled") && item.closest("[inert]") === null,
  );
}

export function modalKeys(onescape: () => void, home: () => HTMLElement | undefined) {
  return (node: HTMLElement) => {
    const onkeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onescape();
        return;
      }
      if (event.key !== "Tab") {
        return;
      }
      const items = focusables(node);
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (!first || !last) {
        event.preventDefault();
        home()?.focus();
        return;
      }
      if (!node.contains(active)) {
        event.preventDefault();
        (event.shiftKey ? last : first).focus();
      } else if (event.shiftKey && (active === first || active === node || active === home())) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onkeydown);
    return () => document.removeEventListener("keydown", onkeydown);
  };
}

export function focusOnLeave(target: Element | null): () => void {
  return () => {
    requestAnimationFrame(() => {
      const lost = document.activeElement === null || document.activeElement === document.body;
      if (lost && target instanceof HTMLElement && target.isConnected && target.closest("[inert]") === null) {
        target.focus({ preventScroll: true });
      }
    });
  };
}
