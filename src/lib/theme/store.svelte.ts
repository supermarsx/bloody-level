type Mode = "light" | "dark" | "system";

const STORAGE_KEY = "blevel.theme";

function readStored(): Mode {
  if (typeof localStorage === "undefined") return "system";
  const v = localStorage.getItem(STORAGE_KEY);
  return v === "light" || v === "dark" || v === "system" ? v : "system";
}

function systemDark(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function resolve(mode: Mode): "light" | "dark" {
  if (mode === "system") return systemDark() ? "dark" : "light";
  return mode;
}

class ThemeStore {
  mode: Mode = $state<Mode>(readStored());
  resolved: "light" | "dark" = $state(resolve(this.mode));

  constructor() {
    if (typeof window !== "undefined") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", () => {
        if (this.mode === "system") {
          this.resolved = systemDark() ? "dark" : "light";
          this.apply();
        }
      });
    }
  }

  set(mode: Mode) {
    this.mode = mode;
    this.resolved = resolve(mode);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, mode);
    }
    this.apply();
  }

  cycle() {
    const order: Mode[] = ["system", "light", "dark"];
    const next = order[(order.indexOf(this.mode) + 1) % order.length];
    this.set(next);
  }

  apply() {
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", this.resolved);
      document.documentElement.dispatchEvent(
        new CustomEvent("theme-change", { detail: this.resolved }),
      );
    }
  }
}

export const theme = new ThemeStore();
