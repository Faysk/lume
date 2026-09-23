import { useEffect } from "react";

import type { MediaItem } from "../lib/types";

interface MediaListProps {
  items: MediaItem[];
  thumbnails: Map<number, string>;
  selectedId?: number;
  hasMore: boolean;
  loadingMore: boolean;
  onNeedThumbnail: (item: MediaItem) => void;
  onOpen: (item: MediaItem) => void;
  onEndReached: () => void;
}

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 ** 2) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 ** 3) return `${(value / 1024 ** 2).toFixed(1)} MB`;
  return `${(value / 1024 ** 3).toFixed(1)} GB`;
}

function formatDate(value: number | null): string {
  if (!value) return "—";
  return new Intl.DateTimeFormat("pt-PT", { dateStyle: "medium" }).format(
    new Date(value * 1000),
  );
}

export function MediaList({
  items,
  thumbnails,
  selectedId,
  hasMore,
  loadingMore,
  onNeedThumbnail,
  onOpen,
  onEndReached,
}: MediaListProps) {
  useEffect(() => {
    for (const item of items.slice(0, 80)) {
      if (item.mediaType === "image" && !thumbnails.has(item.id)) {
        onNeedThumbnail(item);
      }
    }
  }, [items, onNeedThumbnail, thumbnails]);

  return (
    <div
      className="media-list-scroll"
      onScroll={(event) => {
        const element = event.currentTarget;
        const remaining =
          element.scrollHeight - element.scrollTop - element.clientHeight;
        if (hasMore && !loadingMore && remaining < 500) onEndReached();
      }}
    >
      <div className="media-list-header" aria-hidden="true">
        <span>Mídia</span>
        <span>Tipo</span>
        <span>Tamanho</span>
        <span>Data</span>
        <span>Fonte</span>
      </div>

      <div className="media-list">
        {items.map((item) => (
          <button
            key={item.id}
            type="button"
            className={`media-list-row${selectedId === item.id ? " media-list-row-selected" : ""}`}
            onClick={() => onOpen(item)}
          >
            <span className="list-media-cell">
              <span className="list-thumb">
                {thumbnails.get(item.id) ? (
                  <img src={thumbnails.get(item.id)} alt="" draggable={false} />
                ) : (
                  <span aria-hidden="true">
                    {item.mediaType === "video" ? "▶" : "·"}
                  </span>
                )}
              </span>
              <span className="list-name-wrap">
                <strong>{item.fileName}</strong>
                <small>{item.relativePath}</small>
              </span>
            </span>
            <span>{item.extension.toUpperCase()}</span>
            <span>{formatBytes(item.sizeBytes)}</span>
            <span>{formatDate(item.modifiedAtFs ?? item.createdAtFs)}</span>
            <span title={item.sourceRoot}>{item.sourceName}</span>
          </button>
        ))}
      </div>

      {loadingMore ? (
        <div className="page-loader list-page-loader">Carregando mais mídias…</div>
      ) : null}
    </div>
  );
}
