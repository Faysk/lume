import { useEffect, useRef, useState } from "react";

import { mediaUrl } from "../lib/api";
import type { MediaItem } from "../lib/types";

interface ViewerProps {
  item: MediaItem;
  canPrevious: boolean;
  canNext: boolean;
  onPrevious: () => void;
  onNext: () => void;
  onClose: () => void;
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
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let active = true;
    setUrl(undefined);
    setError(undefined);

    mediaUrl(item.id)
      .then((value) => {
        if (active) setUrl(value);
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      });

    return () => {
      active = false;
    };
  }, [item.id]);

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
        <button className="icon-button" type="button" onClick={onClose} aria-label="Fechar">
          ×
        </button>
      </header>

      <main className="viewer-stage">
        {!url && !error ? <div className="viewer-loading">Abrindo mídia…</div> : null}

        {error ? (
          <div className="viewer-message">
            <strong>Não deu para abrir este arquivo.</strong>
            <span>{error}</span>
          </div>
        ) : null}

        {url && item.mediaType === "image" ? (
          <img className="viewer-image" src={url} alt={item.fileName} />
        ) : null}

        {url && item.mediaType === "video" ? (
          <video
            key={url}
            className="viewer-video"
            src={url}
            controls
            autoPlay
            onError={() =>
              setError("O codec deste vídeo não é reproduzível pelo WebView do Windows.")
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

      <footer className="viewer-footer">
        <span>{item.relativePath}</span>
        <span>{item.extension.toUpperCase()}</span>
      </footer>
    </div>
  );
}
