import { invoke } from "@tauri-apps/api/core";
import type { ConflictFile, ConflictDocument } from "./types";

export const listConflicts = () =>
  invoke<ConflictFile[]>("list_conflicts");

export const setRepo = (path: string) =>
  invoke<ConflictFile[]>("set_repo", { path });

export const loadConflict = (path: string) =>
  invoke<ConflictDocument>("load_conflict", { path });

export const saveResolution = (path: string, content: string) =>
  invoke<ConflictFile[]>("save_resolution", { path, content });
