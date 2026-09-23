import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import type { MediaPage, MediaQuery, Source, UiPreferences } from "./types";

export async function chooseSourceDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Adicionar fonte ao Lume",
  });

  return typeof selected === "string" ? selected : null;
}

export function getUiPreferences(): Promise<UiPreferences> {
  return invoke<UiPreferences>("get_ui_preferences");
}

export function saveUiPreferences(
  preferences: UiPreferences,
): Promise<UiPreferences> {
  return invoke<UiPreferences>("save_ui_preferences", { preferences });
}

export function listSources(): Promise<Source[]> {
  return invoke<Source[]>("list_sources");
}

export function addSource(path: string): Promise<Source> {
  return invoke<Source>("add_source", { path });
}

export function startScan(sourceId: number): Promise<boolean> {
  return invoke<boolean>("start_scan", { sourceId });
}

export function cancelScan(sourceId: number): Promise<boolean> {
  return invoke<boolean>("cancel_scan", { sourceId });
}

export function removeSource(sourceId: number): Promise<boolean> {
  return invoke<boolean>("remove_source", { sourceId });
}

export function queryMedia(query: MediaQuery): Promise<MediaPage> {
  return invoke<MediaPage>("query_media", { query });
}

export function listExtensions(): Promise<string[]> {
  return invoke<string[]>("list_extensions");
}

export function getMediaItem(mediaId: number): Promise<import("./types").MediaItem> {
  return invoke<import("./types").MediaItem>("get_media_item", { mediaId });
}

export function openMediaExternal(mediaId: number): Promise<void> {
  return invoke<void>("open_media_external", { mediaId });
}

export function revealMediaInFolder(mediaId: number): Promise<void> {
  return invoke<void>("reveal_media_in_folder", { mediaId });
}

export function copyMediaName(mediaId: number): Promise<void> {
  return invoke<void>("copy_media_name", { mediaId });
}

export function copyMediaPath(mediaId: number): Promise<void> {
  return invoke<void>("copy_media_path", { mediaId });
}

export function cacheSize(): Promise<number> {
  return invoke<number>("cache_size");
}

export function clearThumbnailCache(): Promise<number> {
  return invoke<number>("clear_thumbnail_cache");
}

export async function thumbnailUrl(mediaId: number): Promise<string> {
  const path = await invoke<string>("ensure_thumbnail", { mediaId });
  return convertFileSrc(path);
}

export async function mediaUrl(mediaId: number): Promise<string> {
  const path = await invoke<string>("media_asset_path", { mediaId });
  return convertFileSrc(path);
}
