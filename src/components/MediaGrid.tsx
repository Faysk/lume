import { useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { MediaItem } from "../lib/types";
import { MediaCard } from "./MediaCard";

interface MediaGridProps {
  items: MediaItem[];
  thumbnails: Map<number, string>;
  hasMore: boolean;
  loadingMore: boolean;
  onNeedThumbnail: (item: MediaItem) => void;
  onOpen: (item: MediaItem) => void;
  onEndReached: () => void;
}

const MIN_CARD_WIDTH = 188;
const GAP = 14;

export function MediaGrid({
  items,
  thumbnails,
  hasMore,
  loadingMore,
  onNeedThumbnail,
  onOpen,
  onEndReached,
}: MediaGridProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(900);

  useEffect(() => {
    const element = scrollRef.current;
    if (!element) return;

    const update = () => setWidth(element.clientWidth);
    update();

    const observer = new ResizeObserver(update);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const columns = Math.max(1, Math.floor((width + GAP) / (MIN_CARD_WIDTH + GAP)));
  const cardWidth = Math.max(
    120,
    (width - GAP * Math.max(0, columns - 1) - 24) / columns,
  );
  const estimatedRowHeight = cardWidth + 72;
  const rowCount = Math.ceil(items.length / columns);

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => estimatedRowHeight,
    overscan: 3,
  });

  useEffect(() => {
    virtualizer.measure();
  }, [columns, estimatedRowHeight, virtualizer]);

  const virtualRows = virtualizer.getVirtualItems();
  const lastRow = virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].index : 0;

  useEffect(() => {
    if (hasMore && !loadingMore && rowCount > 0 && lastRow >= rowCount - 3) {
      onEndReached();
    }
  }, [hasMore, lastRow, loadingMore, onEndReached, rowCount]);

  const rows = useMemo(
    () =>
      virtualRows.map((virtualRow) => {
        const start = virtualRow.index * columns;
        return {
          virtualRow,
          items: items.slice(start, start + columns),
        };
      }),
    [columns, items, virtualRows],
  );

  return (
    <div ref={scrollRef} className="media-grid-scroll">
      <div
        className="media-grid-virtual"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {rows.map(({ virtualRow, items: rowItems }) => (
          <div
            key={virtualRow.key}
            className="media-row"
            style={{
              gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
              transform: `translateY(${virtualRow.start}px)`,
            }}
          >
            {rowItems.map((item) => (
              <MediaCard
                key={item.id}
                item={item}
                thumbnail={thumbnails.get(item.id)}
                onNeedThumbnail={onNeedThumbnail}
                onOpen={onOpen}
              />
            ))}
          </div>
        ))}
      </div>

      {loadingMore ? <div className="page-loader">Carregando mais mídias…</div> : null}
    </div>
  );
}
