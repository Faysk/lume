export interface Source {
  id: number;
  rootPath: string;
  displayName: string;
  status: "online" | "scanning" | "offline" | "error" | string;
  lastScanStartedAt: string | null;
  lastScanFinishedAt: string | null;
}

export interface MediaItem {
  id: number;
  sourceId: number;
  relativePath: string;
  fileName: string;
  extension: string;
  mediaType: "image" | "video" | string;
  sizeBytes: number;
  createdAtFs: number | null;
  modifiedAtFs: number | null;
  width: number | null;
  height: number | null;
  thumbnailState: "pending" | "ready" | "failed" | string;
  sourceName: string;
  sourceRoot: string;
}

export interface MediaPage {
  items: MediaItem[];
  total: number;
  offset: number;
  limit: number;
}

export interface ScanProgress {
  sourceId: number;
  discovered: number;
  supported: number;
  errors: number;
  done: boolean;
  cancelled: boolean;
  message: string | null;
}

export type MediaSort =
  | "date_desc"
  | "date_asc"
  | "name_asc"
  | "name_desc"
  | "size_desc"
  | "size_asc";

export interface MediaQuery {
  offset: number;
  limit: number;
  search?: string | null;
  mediaType?: "image" | "video" | null;
  extensions: string[];
  sourceIds: number[];
  modifiedFrom?: number | null;
  modifiedTo?: number | null;
  minSizeBytes?: number | null;
  maxSizeBytes?: number | null;
  sort?: MediaSort | null;
}

export type ThemePreference = "system" | "light" | "dark";

export interface UiPreferences {
  version: number;
  theme: ThemePreference;
  viewMode: "grid" | "list";
  sort: MediaSort;
  thumbnailWidth: number;
}
