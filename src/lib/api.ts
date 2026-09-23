import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import type { MediaPage, Source } from "./types";

export async function chooseSourceDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Adicionar fonte ao Lume",
  });

  return typeof selected === "string" ? selected : null;
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

export function queryMedia(offset: number, limit: number): Promise<MediaPage> {
  return invoke<MediaPage>("query_media", { offset, limit });
}

export async function thumbnailUrl(mediaId: number): Promise<string> {
  const path = await invoke<string>("ensure_thumbnail", { mediaId });
  return convertFileSrc(path);
}

export async function mediaUrl(mediaId: number): Promise<string> {
  const path = await invoke<string>("media_asset_path", { mediaId });
  return convertFileSrc(path);
}
