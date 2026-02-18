export interface ToastMessage {
  id: string;
  text: string;
  type: "success" | "error" | "info";
}

export function createToastStore() {
  let toasts: ToastMessage[] = $state([]);

  function show(
    text: string,
    type: ToastMessage["type"] = "info",
    duration = 3000
  ) {
    const id = crypto.randomUUID();
    toasts = [...toasts, { id, text, type }];
    setTimeout(() => {
      dismiss(id);
    }, duration);
  }

  function dismiss(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  }

  return {
    get toasts() {
      return toasts;
    },
    show,
    dismiss,
    success(text: string) {
      show(text, "success");
    },
    error(text: string) {
      show(text, "error", 5000);
    },
    info(text: string) {
      show(text, "info");
    },
  };
}
