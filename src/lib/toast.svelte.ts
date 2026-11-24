export type ToastTone = "success" | "error";

export interface ToastAction {
  label: string;
  run: () => void;
}

export interface Toast {
  id: number;
  tone: ToastTone;
  title: string;
  body: string;
  action: ToastAction | null;
}

export interface ToastRequest {
  tone?: ToastTone;
  title: string;
  body: string;
  action?: ToastAction;
}

export class ToastStore {
  #current = $state.raw<Toast | null>(null);
  #next = 1;

  get current(): Toast | null {
    return this.#current;
  }

  show(request: ToastRequest): Toast {
    const toast: Toast = {
      id: this.#next,
      tone: request.tone ?? "success",
      title: request.title,
      body: request.body,
      action: request.action ?? null,
    };
    this.#next += 1;
    this.#current = toast;
    return toast;
  }

  dismiss(id: number): void {
    if (this.#current?.id === id) {
      this.#current = null;
    }
  }
}

export const toasts = new ToastStore();
