import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import { MediaGrid } from "./components/MediaGrid";
import { Viewer } from "./components/Viewer";
import {
  addSource,
  chooseSourceDirectory,
  listSources,
  queryMedia,
  startScan,
  thumbnailUrl,
} from "./lib/api";
import type { MediaItem, ScanProgress, Source } from "./lib/types";

const PAGE_SIZE = 240;

function sourceStatusLabel(source: Source): string {
  switch (source.status) {
    case "scanning":
      return "Escaneando";
    case "error":
      return "Erro";
    case "offline":
      return "Offline";
    default:
      return "Online";
  }
}

function App() {
  const [sources, setSources] = useState<Source[]>([]);
  const [items, setItems] = useState<MediaItem[]>([]);
  const [total, setTotal] = useState(0);
  const [thumbnails, setThumbnails] = useState<Map<number, string>>(new Map());
  const [scanProgress, setScanProgress] = useState<Map<number, ScanProgress>>(new Map());
  const [selectedId, setSelectedId] = useState<number>();
  const [initializing, setInitializing] = useState(true);
  const [addingSource, setAddingSource] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string>();
  const thumbnailAttempts = useRef(new Set<number>());
  const refreshTimer = useRef<number>();

  const loadSources = useCallback(async () => {
    const next = await listSources();
    setSources(next);
  }, []);

  const refreshMedia = useCallback(async (requestedLimit?: number) => {
    const limit = Math.max(PAGE_SIZE, requestedLimit ?? PAGE_SIZE);
    const page = await queryMedia(0, limit);
    setItems(page.items);
    setTotal(page.total);
  }, []);

  useEffect(() => {
    let active = true;

    Promise.all([listSources(), queryMedia(0, PAGE_SIZE)])
      .then(([nextSources, page]) => {
        if (!active) return;
        setSources(nextSources);
        setItems(page.items);
        setTotal(page.total);
      })
      .catch((reason: unknown) => {
        if (!active) return;
        setError(reason instanceof Error ? reason.message : String(reason));
      })
      .finally(() => {
        if (active) setInitializing(false);
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    listen<ScanProgress>("scan-progress", (event) => {
      if (!active) return;
      const progress = event.payload;

      setScanProgress((current) => {
        const next = new Map(current);
        next.set(progress.sourceId, progress);
        return next;
      });

      window.clearTimeout(refreshTimer.current);
      refreshTimer.current = window.setTimeout(() => {
        void refreshMedia(Math.max(PAGE_SIZE, items.length)).catch((reason: unknown) => {
          setError(reason instanceof Error ? reason.message : String(reason));
        });

        if (progress.done) {
          void loadSources().catch(() => undefined);
        }
      }, progress.done ? 0 : 120);
    })
      .then((stop) => {
        if (active) unlisten = stop;
        else stop();
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      });

    return () => {
      active = false;
      window.clearTimeout(refreshTimer.current);
      unlisten?.();
    };
  }, [items.length, loadSources, refreshMedia]);

  const handleAddSource = useCallback(async () => {
    setError(undefined);
    setAddingSource(true);

    try {
      const path = await chooseSourceDirectory();
      if (!path) return;

      const source = await addSource(path);
      setSources((current) => {
        const withoutDuplicate = current.filter((item) => item.id !== source.id);
        return [...withoutDuplicate, { ...source, status: "scanning" }];
      });

      const started = await startScan(source.id);
      if (!started) {
        setError("Essa fonte já está sendo escaneada.");
      }
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setAddingSource(false);
    }
  }, []);

  const handleNeedThumbnail = useCallback(async (item: MediaItem) => {
    if (thumbnailAttempts.current.has(item.id)) return;
    thumbnailAttempts.current.add(item.id);

    try {
      const url = await thumbnailUrl(item.id);
      setThumbnails((current) => {
        const next = new Map(current);
        next.set(item.id, url);
        return next;
      });
    } catch {
      // A failed thumbnail remains a local placeholder in the 0.1 UI.
    }
  }, []);

  const handleLoadMore = useCallback(async () => {
    if (loadingMore || items.length >= total) return;
    setLoadingMore(true);

    try {
      const page = await queryMedia(items.length, PAGE_SIZE);
      setItems((current) => {
        const known = new Set(current.map((item) => item.id));
        return [...current, ...page.items.filter((item) => !known.has(item.id))];
      });
      setTotal(page.total);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setLoadingMore(false);
    }
  }, [items.length, loadingMore, total]);

  const selectedIndex = useMemo(
    () => (selectedId === undefined ? -1 : items.findIndex((item) => item.id === selectedId)),
    [items, selectedId],
  );
  const selected = selectedIndex >= 0 ? items[selectedIndex] : undefined;

  const openPrevious = useCallback(() => {
    if (selectedIndex > 0) setSelectedId(items[selectedIndex - 1].id);
  }, [items, selectedIndex]);

  const openNext = useCallback(() => {
    if (selectedIndex >= 0 && selectedIndex < items.length - 1) {
      setSelectedId(items[selectedIndex + 1].id);
    }
  }, [items, selectedIndex]);

  const activeScan = [...scanProgress.values()].find((progress) => !progress.done);
  const hasSources = sources.length > 0;
  const hasItems = items.length > 0;

  if (initializing) {
    return (
      <main className="boot-screen">
        <div className="lume-mark" aria-hidden="true">L</div>
        <strong>Lume</strong>
        <span>Abrindo sua biblioteca…</span>
      </main>
    );
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">L</span>
          <div>
            <strong>Lume</strong>
            <span>Toda a sua mídia, sem tirar nada do lugar.</span>
          </div>
        </div>

        <div className="header-actions">
          {total > 0 ? (
            <span className="library-count">
              {total.toLocaleString("pt-PT")} {total === 1 ? "mídia" : "mídias"}
            </span>
          ) : null}
          <button
            className="primary-button"
            type="button"
            onClick={() => void handleAddSource()}
            disabled={addingSource}
          >
            <span aria-hidden="true">＋</span>
            {addingSource ? "Abrindo…" : "Adicionar pasta"}
          </button>
        </div>
      </header>

      {sources.length > 0 ? (
        <div className="source-strip" aria-label="Fontes">
          {sources.map((source) => {
            const progress = scanProgress.get(source.id);
            const status = progress && !progress.done ? "scanning" : source.status;

            return (
              <div className="source-pill" key={source.id} title={source.rootPath}>
                <span className={`status-dot status-${status}`} />
                <strong>{source.displayName}</strong>
                <span>
                  {progress && !progress.done
                    ? `${progress.supported.toLocaleString("pt-PT")} encontradas`
                    : sourceStatusLabel({ ...source, status })}
                </span>
              </div>
            );
          })}
        </div>
      ) : null}

      {error ? (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button type="button" onClick={() => setError(undefined)}>Fechar</button>
        </div>
      ) : null}

      {activeScan ? (
        <div className="scan-bar">
          <span className="scan-pulse" />
          <span>
            Lendo a fonte… {activeScan.supported.toLocaleString("pt-PT")} mídias encontradas
          </span>
        </div>
      ) : null}

      <main className="library-main">
        {!hasSources ? (
          <section className="empty-state">
            <div className="empty-orbit" aria-hidden="true">
              <span />
            </div>
            <p className="eyebrow">Sua biblioteca começa aqui</p>
            <h1>Escolha uma pasta.<br />O Lume cuida do resto.</h1>
            <p>
              Fotos, vídeos e GIFs aparecem numa única galeria. Os arquivos originais
              continuam exatamente onde estão.
            </p>
            <button
              className="primary-button primary-button-large"
              type="button"
              onClick={() => void handleAddSource()}
              disabled={addingSource}
            >
              <span aria-hidden="true">＋</span>
              Adicionar primeira pasta
            </button>
            <span className="readonly-note">Somente leitura · nada será movido ou apagado</span>
          </section>
        ) : !hasItems && activeScan ? (
          <section className="empty-state compact">
            <div className="loading-ring" aria-hidden="true" />
            <h2>Procurando suas mídias…</h2>
            <p>As primeiras vão aparecer aqui sem esperar a varredura terminar.</p>
          </section>
        ) : !hasItems ? (
          <section className="empty-state compact">
            <h2>Nenhuma mídia suportada por enquanto.</h2>
            <p>Você pode adicionar outra pasta ou tentar novamente mais tarde.</p>
          </section>
        ) : (
          <MediaGrid
            items={items}
            thumbnails={thumbnails}
            hasMore={items.length < total}
            loadingMore={loadingMore}
            onNeedThumbnail={handleNeedThumbnail}
            onOpen={(item) => setSelectedId(item.id)}
            onEndReached={() => void handleLoadMore()}
          />
        )}
      </main>

      {selected ? (
        <Viewer
          item={selected}
          canPrevious={selectedIndex > 0}
          canNext={selectedIndex < items.length - 1}
          onPrevious={openPrevious}
          onNext={openNext}
          onClose={() => setSelectedId(undefined)}
        />
      ) : null}
    </div>
  );
}

export default App;
