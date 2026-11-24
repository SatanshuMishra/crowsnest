import type { AppearanceKey, AppearanceValues } from "./appearance";

export type PreviewVia = "hover" | "focus";

export interface ActivePreview {
  key: AppearanceKey;
  value: AppearanceValues[AppearanceKey];
  via: PreviewVia;
}

class PreviewStore {
  current = $state.raw<ActivePreview | null>(null);

  show(next: ActivePreview): void {
    this.current = next;
  }

  end(key: AppearanceKey, via: PreviewVia): void {
    if (this.current?.key === key && this.current.via === via) {
      this.current = null;
    }
  }

  clear(): void {
    this.current = null;
  }
}

export const preview = new PreviewStore();
