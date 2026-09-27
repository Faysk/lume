import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

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
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 58,
    overscan: 8,
    scrollMargin: 34,
  });
  const virtualRows = virtualizer.getVirtualItems();

  useEffect(() => {
    for (const row of virtualRows) {
      const item = items[row.index];
      if (!item) continue;
      if (item.mediaType === "image" && !thumbnails.has(item.id)) {
        onNeedThumbnail(item);
      }
    }
  }, [items, onNeedThumbnail, thumbnails, virtualRows]);

  const lastRow =
    virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].index : 0;
  useEffect(() => {
    if (
      hasMore &&
      !loadingMore &&
      items.length > 0 &&
      lastRow >= items.length - 12
    ) {
      onEndReached();
    }
  }, [hasMore, items.length, lastRow, loadingMore, onEndReached]);

  return (
    <div ref={scrollRef} className="media-list-scroll">
      <div className="media-list-header" aria-hidden="true">
        <span>Mídia</span>
        <span>Tipo</span>
        <span>Tamanho</span>
        <span>Data</span>
        <span>Fonte</span>
      </div>

      <div
        className="media-list"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {virtualRows.map((row) => {
          const item = items[row.index];
          if (!item) return null;

          return (
            <button
              key={item.id}
              type="button"
              className={`media-list-row${selectedId === item.id ? " media-list-row-selected" : ""}`}
              onClick={() => onOpen(item)}
              style={{ transform: `translateY(${row.start - 34}px)` }}
            >
              <span className="list-media-cell">
                <span className="list-thumb">
                  {thumbnails.get(item.id) ? (
                    <img
                      src={thumbnails.get(item.id)}
                      alt=""
                      draggable={false}
                    />
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
          );
        })}
      </div>

      {loadingMore ? (
        <div className="page-loader list-page-loader">Carregando mais mídias…</div>
      ) : null}
    </div>
  );
}
