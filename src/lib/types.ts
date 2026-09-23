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
  message: string | null;
}
