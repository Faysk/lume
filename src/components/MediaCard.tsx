import { useEffect } from "react";

import type { MediaItem } from "../lib/types";

interface MediaCardProps {
  item: MediaItem;
  thumbnail?: string;
  onNeedThumbnail: (item: MediaItem) => void;
  onOpen: (item: MediaItem) => void;
  selected?: boolean;
}

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 ** 2) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 ** 3) return `${(value / 1024 ** 2).toFixed(1)} MB`;
  return `${(value / 1024 ** 3).toFixed(1)} GB`;
}

export function MediaCard({
  item,
  thumbnail,
  onNeedThumbnail,
  onOpen,
  selected = false,
}: MediaCardProps) {
  useEffect(() => {
    if (item.mediaType === "image" && !thumbnail) {
      onNeedThumbnail(item);
    }
  }, [item, onNeedThumbnail, thumbnail]);

  return (
    <button
      className={`media-card${selected ? " media-card-selected" : ""}`}
      type="button"
      onClick={() => onOpen(item)}
      title={item.fileName}
    >
      <span className="media-preview">
        {thumbnail ? (
          <img src={thumbnail} alt="" draggable={false} />
        ) : item.mediaType === "video" ? (
          <span className="video-placeholder" aria-hidden="true">
            <span className="play-glyph">▶</span>
          </span>
        ) : (
          <span className="thumb-placeholder" aria-hidden="true">
            <span />
          </span>
        )}
        <span className="media-badge">{item.extension.toUpperCase()}</span>
      </span>

      <span className="media-caption">
        <span className="media-name">{item.fileName}</span>
        <span className="media-meta">
          {item.sourceName} · {formatBytes(item.sizeBytes)}
        </span>
      </span>
    </button>
  );
}
