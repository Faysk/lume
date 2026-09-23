import { useEffect, useRef, useState } from "react";

import {
  getMediaItem,
  mediaUrl,
  openMediaExternal,
  thumbnailUrl,
} from "../lib/api";
import type { MediaItem } from "../lib/types";

interface ViewerProps {
  item: MediaItem;
  canPrevious: boolean;
  canNext: boolean;
  onPrevious: () => void;
  onNext: () => void;
  onClose: () => void;
}

interface Point {
  x: number;
  y: number;
}

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 ** 2) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 ** 3) return `${(value / 1024 ** 2).toFixed(1)} MB`;
  return `${(value / 1024 ** 3).toFixed(1)} GB`;
}

function formatDate(value: number | null): string {
  if (!value) return "—";
  return new Intl.DateTimeFormat("pt-PT", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value * 1000));
}

export function Viewer({
  item,
  canPrevious,
  canNext,
  onPrevious,
  onNext,
  onClose,
}: ViewerProps) {
  const [url, setUrl] = useState<string>();
  const [error, setError] = useState<string>();
  const [details, setDetails] = useState(item);
  const [showInfo, setShowInfo] = useState(false);
  const [zoom, setZoom] = useState(1);
  const [actualSize, setActualSize] = useState(false);
  const [pan, setPan] = useState<Point>({ x: 0, y: 0 });
  const dialogRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<
    { pointerId: number; start: Point; origin: Point } | undefined
  >(undefined);

  useEffect(() => {
    let active = true;
    setUrl(undefined);
    setError(undefined);
    setDetails(item);
    setZoom(1);
    setActualSize(false);
    setPan({ x: 0, y: 0 });

    mediaUrl(item.id)
      .then((value) => {
        if (active) setUrl(value);
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      });

    const refreshDetails = async () => {
      if (item.mediaType === "image") {
        await thumbnailUrl(item.id).catch(() => undefined);
      }
      const fresh = await getMediaItem(item.id);
      if (active) setDetails(fresh);
    };

    void refreshDetails().catch(() => undefined);

    return () => {
      active = false;
    };
  }, [item]);

  useEffect(() => {
    dialogRef.current?.focus();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      } else if (event.target instanceof HTMLVideoElement) {
        return;
      } else if (event.key === "ArrowLeft" && canPrevious) {
        event.preventDefault();
        onPrevious();
      } else if (event.key === "ArrowRight" && canNext) {
        event.preventDefault();
        onNext();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [canNext, canPrevious, onClose, onNext, onPrevious]);

  const setFit = () => {
    setActualSize(false);
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const setActual = () => {
    setActualSize(true);
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const changeZoom = (next: number) => {
    setZoom(Math.min(8, Math.max(0.25, next)));
    if (next <= 1 && !actualSize) setPan({ x: 0, y: 0 });
  };

  const imageCanPan = item.mediaType === "image" && (actualSize || zoom > 1);

  return (
    <div
      ref={dialogRef}
      className="viewer-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={item.fileName}
      tabIndex={-1}
    >
      <header className="viewer-header">
        <div className="viewer-title">
          <strong>{item.fileName}</strong>
          <span>{item.sourceName}</span>
        </div>

        <div className="viewer-actions">
          {item.mediaType === "image" ? (
            <>
              <button type="button" onClick={setFit}>
                Ajustar
              </button>
              <button type="button" onClick={setActual}>
                100%
              </button>
              <button
                type="button"
                aria-label="Diminuir zoom"
                onClick={() => changeZoom(zoom / 1.2)}
              >
                −
              </button>
              <span className="zoom-label">{Math.round(zoom * 100)}%</span>
              <button
                type="button"
                aria-label="Aumentar zoom"
                onClick={() => changeZoom(zoom * 1.2)}
              >
                ＋
              </button>
            </>
          ) : null}

          <button type="button" onClick={() => setShowInfo((value) => !value)}>
            Info
          </button>
          <button
            type="button"
            onClick={() =>
              void openMediaExternal(item.id).catch((reason: unknown) => {
                setError(reason instanceof Error ? reason.message : String(reason));
              })
            }
          >
            Abrir fora
          </button>
          <button className="icon-button" type="button" onClick={onClose} aria-label="Fechar">
            ×
          </button>
        </div>
      </header>

      <div className={`viewer-body${showInfo ? " viewer-body-with-info" : ""}`}>
        <main
          className={`viewer-stage${imageCanPan ? " viewer-stage-pannable" : ""}`}
          onWheel={(event) => {
            if (item.mediaType !== "image") return;
            event.preventDefault();
            changeZoom(zoom * (event.deltaY < 0 ? 1.12 : 1 / 1.12));
          }}
          onPointerDown={(event) => {
            if (!imageCanPan) return;
            event.currentTarget.setPointerCapture(event.pointerId);
            dragRef.current = {
              pointerId: event.pointerId,
              start: { x: event.clientX, y: event.clientY },
              origin: pan,
            };
          }}
          onPointerMove={(event) => {
            const drag = dragRef.current;
            if (!drag || drag.pointerId !== event.pointerId) return;
            setPan({
              x: drag.origin.x + event.clientX - drag.start.x,
              y: drag.origin.y + event.clientY - drag.start.y,
            });
          }}
          onPointerUp={(event) => {
            if (dragRef.current?.pointerId === event.pointerId) {
              dragRef.current = undefined;
              event.currentTarget.releasePointerCapture(event.pointerId);
            }
          }}
        >
          {!url && !error ? <div className="viewer-loading">Abrindo mídia…</div> : null}

          {error ? (
            <div className="viewer-message">
              <strong>Não deu para abrir este arquivo.</strong>
              <span>{error}</span>
              <button
                className="toolbar-ghost-button"
                type="button"
                onClick={() => void openMediaExternal(item.id)}
              >
                Tentar no aplicativo padrão
              </button>
            </div>
          ) : null}

          {url && item.mediaType === "image" ? (
            <img
              className={`viewer-image${actualSize ? " viewer-image-actual" : ""}`}
              src={url}
              alt={item.fileName}
              draggable={false}
              style={{
                transform: `translate3d(${pan.x}px, ${pan.y}px, 0) scale(${zoom})`,
              }}
            />
          ) : null}

          {url && item.mediaType === "video" ? (
            <video
              key={url}
              className="viewer-video"
              src={url}
              controls
              autoPlay
              onError={() =>
                setError(
                  "O codec deste vídeo não é reproduzível pelo WebView do Windows.",
                )
              }
            />
          ) : null}

          <button
            className="viewer-nav viewer-nav-left"
            type="button"
            onClick={onPrevious}
            disabled={!canPrevious}
            aria-label="Mídia anterior"
          >
            ‹
          </button>
          <button
            className="viewer-nav viewer-nav-right"
            type="button"
            onClick={onNext}
            disabled={!canNext}
            aria-label="Próxima mídia"
          >
            ›
          </button>
        </main>

        {showInfo ? (
          <aside className="viewer-info" aria-label="Informações da mídia">
            <h3>Informações</h3>
            <dl>
              <div>
                <dt>Nome</dt>
                <dd>{details.fileName}</dd>
              </div>
              <div>
                <dt>Tipo</dt>
                <dd>{details.extension.toUpperCase()}</dd>
              </div>
              <div>
                <dt>Tamanho</dt>
                <dd>{formatBytes(details.sizeBytes)}</dd>
              </div>
              <div>
                <dt>Data</dt>
                <dd>{formatDate(details.modifiedAtFs ?? details.createdAtFs)}</dd>
              </div>
              <div>
                <dt>Dimensões</dt>
                <dd>
                  {details.width && details.height
                    ? `${details.width} × ${details.height}`
                    : "—"}
                </dd>
              </div>
              <div>
                <dt>Fonte</dt>
                <dd>{details.sourceName}</dd>
              </div>
              <div>
                <dt>Caminho</dt>
                <dd>{details.relativePath}</dd>
              </div>
            </dl>
          </aside>
        ) : null}
      </div>

      <footer className="viewer-footer">
        <span>{item.relativePath}</span>
        <span>{item.extension.toUpperCase()}</span>
      </footer>
    </div>
  );
}
