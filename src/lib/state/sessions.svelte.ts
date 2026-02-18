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
    saveTimer = setTimeout(() => {
      save(meta);
    }, 500);
  }

  /** Persist session metadata */
  function save(_meta: Partial<SessionMeta>) {
    if (!currentSessionId) {
      return;
    }
    // TODO: invoke("save_session_meta", { sessionId: currentSessionId, meta })
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

    loadSessions,
    createSession,
    deleteSession,
    scheduleSave,

    setCurrentSession(id: string | null) {
      currentSessionId = id;
    },
  };
}
