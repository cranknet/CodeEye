import { invoke } from "@tauri-apps/api/core";
import type { Annotation } from "./annotations.svelte";

export interface SessionMeta {
  annotations: Annotation[];
  createdAt: number;
  generalNotes: string;
  gitContext?: {
    project: string;
    branch: string;
    working_directory: string;
    suggested_file: string | null;
    recent_diff: string | null;
  };
  id: string;
  pageName: string;
  status: "open" | "resolved";
  updatedAt: number;
  viewport?: { width: number; height: number };
}

export function createSessionStore() {
  let currentSessionId: string | null = $state(null);
  let sessions: SessionMeta[] = $state([]);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingSaveMeta: Partial<SessionMeta> | null = null;
  let autoSaveEnabled = true;

  /** Load session list from Rust backend */
  async function loadSessions() {
    try {
      const list =
        await invoke<
          {
            id: string;
            page_name: string;
            project: string;
            status: string;
            annotation_count: number;
            created_at: number;
            updated_at: number;
          }[]
        >("list_all_sessions");

      sessions = list.map((s) => ({
        id: s.id,
        pageName: s.page_name,
        generalNotes: "",
        annotations: [],
        status: s.status as "open" | "resolved",
        createdAt: s.created_at,
        updatedAt: s.updated_at,
      }));
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  }

  /** Create a new session via Rust backend */
  async function createSession(pageName: string, project: string) {
    try {
      const id = await invoke<string>("create_new_session", {
        pageName,
        project,
      });
      currentSessionId = id;
      return id;
    } catch (err) {
      console.error("Failed to create session:", err);
      return null;
    }
  }

  /** Schedule a debounced save (500ms) */
  function scheduleSave(meta: Partial<SessionMeta>) {
    if (saveTimer) {
      clearTimeout(saveTimer);
    }
    pendingSaveMeta = meta;
    saveTimer = setTimeout(() => {
      save(meta);
    }, 500);
  }

  /** Flush any pending debounced save immediately. Returns when save completes. */
  async function flushSave() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (pendingSaveMeta) {
      const meta = pendingSaveMeta;
      pendingSaveMeta = null;
      await save(meta);
    }
  }

  /** Persist session metadata to Rust backend */
  async function save(meta: Partial<SessionMeta>) {
    if (!currentSessionId) {
      return;
    }
    try {
      const existing = await invoke<Record<string, unknown>>(
        "load_session_meta",
        { sessionId: currentSessionId }
      );

      if (meta.annotations !== undefined) {
        existing.annotations = meta.annotations;
        existing.annotation_count = meta.annotations.length;
      }
      if (meta.generalNotes !== undefined) {
        existing.general_notes = meta.generalNotes;
      }
      if (meta.pageName !== undefined) {
        existing.page_name = meta.pageName;
      }
      if (meta.status !== undefined) {
        existing.status = meta.status;
      }
      if (meta.viewport !== undefined) {
        existing.viewport = meta.viewport;
      }
      existing.updated_at = Date.now();

      await invoke("save_session_meta", {
        sessionId: currentSessionId,
        meta: existing,
      });
    } catch (err) {
      console.error("Failed to save session:", err);
    }
  }

  /** Load full session data from Rust backend */
  async function loadSession(sessionId: string): Promise<SessionMeta | null> {
    try {
      const meta = await invoke<Record<string, unknown>>("load_session_meta", {
        sessionId,
      });
      return {
        id: sessionId,
        pageName: (meta.page_name as string) ?? "",
        generalNotes: (meta.general_notes as string) ?? "",
        annotations: (meta.annotations as Annotation[]) ?? [],
        status: (meta.status as "open" | "resolved") ?? "open",
        createdAt: (meta.created_at as number) ?? 0,
        updatedAt: (meta.updated_at as number) ?? 0,
        gitContext: meta.git as SessionMeta["gitContext"],
        viewport: meta.viewport as SessionMeta["viewport"],
      };
    } catch (err) {
      console.error("Failed to load session:", err);
      return null;
    }
  }

  /** Delete a session */
  async function deleteSession(id: string) {
    try {
      await invoke("delete_session_by_id", { sessionId: id });
      sessions = sessions.filter((s) => s.id !== id);
      if (currentSessionId === id) {
        currentSessionId = null;
      }
    } catch (err) {
      console.error("Failed to delete session:", err);
    }
  }

  return {
    get currentSessionId() {
      return currentSessionId;
    },
    get sessions() {
      return sessions;
    },

    get autoSaveEnabled() {
      return autoSaveEnabled;
    },
    set autoSaveEnabled(v: boolean) {
      autoSaveEnabled = v;
    },

    loadSessions,
    loadSession,
    createSession,
    deleteSession,
    scheduleSave,
    flushSave,

    setCurrentSession(id: string | null) {
      currentSessionId = id;
    },
  };
}
