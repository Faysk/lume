import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import "./App.css";
import { LibraryToolbar } from "./components/LibraryToolbar";
import { MediaGrid } from "./components/MediaGrid";
import { MediaList } from "./components/MediaList";
import { SourceManager } from "./components/SourceManager";
import { Viewer } from "./components/Viewer";
import {
  addSource,
  cacheSize,
  cancelScan,
  chooseSourceDirectory,
  clearThumbnailCache,
  getUiPreferences,
  listExtensions,
  listSources,
  openLogsFolder,
  queryMedia,
  removeSource,
  saveUiPreferences,
  startScan,
  thumbnailUrl,
} from "./lib/api";
import type {
  MediaItem,
  MediaQuery,
  MediaSort,
  ScanProgress,
  Source,
  ThemePreference,
} from "./lib/types";

const PAGE_SIZE = 240;
const MEGABYTE = 1024 * 1024;

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

function dateBoundary(value: string, endOfDay = false): number | null {
  if (!value) return null;
  const date = new Date(`${value}T${endOfDay ? "23:59:59" : "00:00:00"}`);
  const unix = Math.floor(date.getTime() / 1000);
  return Number.isFinite(unix) ? unix : null;
}

function megabytes(value: string): number | null {
  if (!value.trim()) return null;
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < 0) return null;
  return Math.round(parsed * MEGABYTE);
}

function toggleValue<T>(current: T[], value: T): T[] {
  return current.includes(value)
    ? current.filter((item) => item !== value)
    : [...current, value];
}

function App() {
  const [sources, setSources] = useState<Source[]>([]);
  const [extensions, setExtensions] = useState<string[]>([]);
  const [items, setItems] = useState<MediaItem[]>([]);
  const [total, setTotal] = useState(0);
  const [thumbnails, setThumbnails] = useState<Map<number, string>>(new Map());
  const [scanProgress, setScanProgress] = useState<Map<number, ScanProgress>>(
    new Map(),
  );
  const [selectedId, setSelectedId] = useState<number>();
  const [initializing, setInitializing] = useState(true);
  const [addingSource, setAddingSource] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string>();
  const [sourceManagerOpen, setSourceManagerOpen] = useState(false);
  const [cacheBytes, setCacheBytes] = useState(0);
  const [clearingCache, setClearingCache] = useState(false);

  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [mediaType, setMediaType] = useState<"all" | "image" | "video">("all");
  const [selectedExtensions, setSelectedExtensions] = useState<string[]>([]);
  const [selectedSourceIds, setSelectedSourceIds] = useState<number[]>([]);
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [minSizeMb, setMinSizeMb] = useState("");
  const [maxSizeMb, setMaxSizeMb] = useState("");
  const [sort, setSort] = useState<MediaSort>("date_desc");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [minCardWidth, setMinCardWidth] = useState(188);
  const [theme, setTheme] = useState<ThemePreference>("system");
  const [preferencesReady, setPreferencesReady] = useState(false);

  const thumbnailAttempts = useRef(new Set<number>());
  const thumbnailQueue = useRef<MediaItem[]>([]);
  const thumbnailWorkers = useRef(0);
  const refreshTimer = useRef<number | undefined>(undefined);
  const requestVersion = useRef(0);
  const loadingMoreRef = useRef(false);

  useEffect(() => {
    const timer = window.setTimeout(() => setSearch(searchInput.trim()), 260);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "system") {
      root.removeAttribute("data-theme");
    } else {
      root.dataset.theme = theme;
    }
  }, [theme]);

  useEffect(() => {
    if (!preferencesReady) return;

    const timer = window.setTimeout(() => {
      void saveUiPreferences({
        version: 1,
        theme,
        viewMode,
        sort,
        thumbnailWidth: minCardWidth,
      }).catch(() => {
        // Preferências não bloqueiam o uso da biblioteca.
      });
    }, 350);

    return () => window.clearTimeout(timer);
  }, [minCardWidth, preferencesReady, sort, theme, viewMode]);

  useEffect(() => {
    if (!sourceManagerOpen) return;
    void cacheSize()
      .then(setCacheBytes)
      .catch(() => setCacheBytes(0));
  }, [sourceManagerOpen]);


  const baseQuery = useMemo<Omit<MediaQuery, "offset" | "limit">>(
    () => ({
      search: search || null,
      mediaType: mediaType === "all" ? null : mediaType,
      extensions: selectedExtensions,
      sourceIds: selectedSourceIds,
      modifiedFrom: dateBoundary(dateFrom),
      modifiedTo: dateBoundary(dateTo, true),
      minSizeBytes: megabytes(minSizeMb),
      maxSizeBytes: megabytes(maxSizeMb),
      sort,
    }),
    [
      dateFrom,
      dateTo,
      maxSizeMb,
      mediaType,
      minSizeMb,
      search,
      selectedExtensions,
      selectedSourceIds,
      sort,
    ],
  );

  const buildQuery = useCallback(
    (offset: number, limit: number): MediaQuery => ({
      offset,
      limit,
      ...baseQuery,
    }),
    [baseQuery],
  );

  const loadSourcesAndExtensions = useCallback(async () => {
    const [nextSources, nextExtensions] = await Promise.all([
      listSources(),
      listExtensions(),
    ]);
    setSources(nextSources);
    setExtensions(nextExtensions);
  }, []);

  const refreshMedia = useCallback(
    async (requestedLimit?: number) => {
      const version = ++requestVersion.current;
      const limit = Math.max(PAGE_SIZE, requestedLimit ?? PAGE_SIZE);
      const page = await queryMedia(buildQuery(0, limit));
      if (version !== requestVersion.current) return;
      setItems(page.items);
      setTotal(page.total);
    },
    [buildQuery],
  );

  useEffect(() => {
    let active = true;

    Promise.all([loadSourcesAndExtensions(), getUiPreferences()])
      .then(([, preferences]) => {
        if (!active) return;
        setTheme(preferences.theme);
        setViewMode(preferences.viewMode);
        setSort(preferences.sort);
        setMinCardWidth(preferences.thumbnailWidth);
        setPreferencesReady(true);
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      })
      .finally(() => {
        if (active) setInitializing(false);
      });

    return () => {
      active = false;
    };
  }, [loadSourcesAndExtensions]);

  useEffect(() => {
    let active = true;
    const version = ++requestVersion.current;

    queryMedia(buildQuery(0, PAGE_SIZE))
      .then((page) => {
        if (!active || version !== requestVersion.current) return;
        setItems(page.items);
        setTotal(page.total);
        setSelectedId((current) =>
          current !== undefined && page.items.some((item) => item.id === current)
            ? current
            : undefined,
        );
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      });

    return () => {
      active = false;
    };
  }, [buildQuery]);

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
        void refreshMedia(Math.max(PAGE_SIZE, items.length)).catch(
          (reason: unknown) => {
            setError(reason instanceof Error ? reason.message : String(reason));
          },
        );

        if (progress.done) {
          void loadSourcesAndExtensions().catch(() => undefined);
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
  }, [items.length, loadSourcesAndExtensions, refreshMedia]);

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
      if (!started) setError("Essa fonte já está sendo escaneada.");
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setAddingSource(false);
    }
  }, []);

  const handleRescan = useCallback(async (sourceId: number) => {
    setError(undefined);

    try {
      const started = await startScan(sourceId);
      if (!started) {
        setError("Essa fonte já está sendo escaneada.");
        return;
      }

      setSources((current) =>
        current.map((source) =>
          source.id === sourceId ? { ...source, status: "scanning" } : source,
        ),
      );
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, []);

  const handleCancelScan = useCallback(async (sourceId: number) => {
    try {
      const requested = await cancelScan(sourceId);
      if (!requested) {
        setError("Essa fonte não possui uma varredura ativa.");
      }
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }, []);

  const handleRemoveSource = useCallback(
    async (source: Source) => {
      const confirmed = window.confirm(
        `Remover "${source.displayName}" do catálogo do Lume?\n\nOs arquivos físicos não serão apagados.`,
      );
      if (!confirmed) return;

      try {
        await removeSource(source.id);
        setSelectedSourceIds((current) =>
          current.filter((sourceId) => sourceId !== source.id),
        );
        setScanProgress((current) => {
          const next = new Map(current);
          next.delete(source.id);
          return next;
        });
        await loadSourcesAndExtensions();
        await refreshMedia();
      } catch (reason: unknown) {
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    },
    [loadSourcesAndExtensions, refreshMedia],
  );

  const handleClearCache = useCallback(async () => {
    if (!window.confirm("Limpar os thumbnails gerados pelo Lume? Os arquivos originais não serão tocados.")) {
      return;
    }

    setClearingCache(true);
    try {
      await clearThumbnailCache();
      setThumbnails(new Map());
      thumbnailAttempts.current.clear();
      thumbnailQueue.current = [];
      setCacheBytes(0);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setClearingCache(false);
    }
  }, []);

  const pumpThumbnailQueue = useCallback(() => {
    const runNext = () => {
      const nextItem = thumbnailQueue.current.shift();

      if (!nextItem) {
        thumbnailWorkers.current = Math.max(0, thumbnailWorkers.current - 1);
        return;
      }

      void thumbnailUrl(nextItem.id)
        .then((url) => {
          setThumbnails((current) => {
            const next = new Map(current);
            next.set(nextItem.id, url);
            return next;
          });
        })
        .catch(() => {
          // O placeholder local continua visível se a mídia não decodificar.
        })
        .finally(runNext);
    };

    while (
      thumbnailWorkers.current < 4 &&
      thumbnailQueue.current.length > 0
    ) {
      thumbnailWorkers.current += 1;
      runNext();
    }
  }, []);

  const handleNeedThumbnail = useCallback(
    (item: MediaItem) => {
      if (thumbnailAttempts.current.has(item.id)) return;
      thumbnailAttempts.current.add(item.id);

      // O item mais recentemente visível ganha prioridade. A fila pendente fica
      // curta para que um scroll rápido não obrigue o Lume a processar tudo que passou.
      thumbnailQueue.current.unshift(item);

      while (thumbnailQueue.current.length > 32) {
        const dropped = thumbnailQueue.current.pop();
        if (dropped) thumbnailAttempts.current.delete(dropped.id);
      }

      pumpThumbnailQueue();
    },
    [pumpThumbnailQueue],
  );

  const loadMorePage = useCallback(async (): Promise<MediaItem[]> => {
    if (loadingMoreRef.current || items.length >= total) return [];
    loadingMoreRef.current = true;
    setLoadingMore(true);

    try {
      const page = await queryMedia(buildQuery(items.length, PAGE_SIZE));
      const known = new Set(items.map((item) => item.id));
      const appended = page.items.filter((item) => !known.has(item.id));
      setItems((current) => [...current, ...appended]);
      setTotal(page.total);
      return appended;
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
      return [];
    } finally {
      loadingMoreRef.current = false;
      setLoadingMore(false);
    }
  }, [buildQuery, items, total]);

  const selectedIndex = useMemo(
    () =>
      selectedId === undefined
        ? -1
        : items.findIndex((item) => item.id === selectedId),
    [items, selectedId],
  );
  const selected = selectedIndex >= 0 ? items[selectedIndex] : undefined;

  const openPrevious = useCallback(() => {
    if (selectedIndex > 0) setSelectedId(items[selectedIndex - 1].id);
  }, [items, selectedIndex]);

  const openNext = useCallback(() => {
    if (selectedIndex < 0) return;

    if (selectedIndex < items.length - 1) {
      setSelectedId(items[selectedIndex + 1].id);
      return;
    }

    if (items.length < total) {
      void loadMorePage().then((appended) => {
        if (appended[0]) setSelectedId(appended[0].id);
      });
    }
  }, [items, loadMorePage, selectedIndex, total]);

  const activeFilterCount =
    selectedExtensions.length +
    selectedSourceIds.length +
    (mediaType === "all" ? 0 : 1) +
    (dateFrom ? 1 : 0) +
    (dateTo ? 1 : 0) +
    (minSizeMb ? 1 : 0) +
    (maxSizeMb ? 1 : 0);

  const clearFilters = () => {
    setMediaType("all");
    setSelectedExtensions([]);
    setSelectedSourceIds([]);
    setDateFrom("");
    setDateTo("");
    setMinSizeMb("");
    setMaxSizeMb("");
  };

  const activeScan = [...scanProgress.values()].find(
    (progress) => !progress.done,
  );
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
          {hasSources ? (
            <button
              className="toolbar-ghost-button header-source-button"
              type="button"
              onClick={() => setSourceManagerOpen(true)}
            >
              Fontes · {sources.length}
            </button>
          ) : null}
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
            const status =
              progress && !progress.done ? "scanning" : source.status;

            return (
              <div className="source-pill" key={source.id} title={source.rootPath}>
                <span className={`status-dot status-${status}`} />
                <span className="source-copy">
                  <strong>{source.displayName}</strong>
                  <span className="source-path">{source.rootPath}</span>
                </span>
                <span className="source-state">
                  {progress && !progress.done
                    ? `${progress.supported.toLocaleString("pt-PT")} encontradas`
                    : sourceStatusLabel({ ...source, status })}
                </span>
              </div>
            );
          })}
        </div>
      ) : null}

      {hasSources ? (
        <LibraryToolbar
          search={searchInput}
          onSearchChange={setSearchInput}
          mediaType={mediaType}
          onMediaTypeChange={setMediaType}
          extensions={extensions}
          selectedExtensions={selectedExtensions}
          onToggleExtension={(extension) =>
            setSelectedExtensions((current) =>
              toggleValue(current, extension),
            )
          }
          sources={sources}
          selectedSourceIds={selectedSourceIds}
          onToggleSource={(sourceId) =>
            setSelectedSourceIds((current) => toggleValue(current, sourceId))
          }
          dateFrom={dateFrom}
          dateTo={dateTo}
          onDateFromChange={setDateFrom}
          onDateToChange={setDateTo}
          minSizeMb={minSizeMb}
          maxSizeMb={maxSizeMb}
          onMinSizeMbChange={setMinSizeMb}
          onMaxSizeMbChange={setMaxSizeMb}
          sort={sort}
          onSortChange={setSort}
          viewMode={viewMode}
          onViewModeChange={setViewMode}
          minCardWidth={minCardWidth}
          onMinCardWidthChange={setMinCardWidth}
          theme={theme}
          onThemeChange={setTheme}
          activeFilterCount={activeFilterCount}
          onClearFilters={clearFilters}
        />
      ) : null}

      {error ? (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button type="button" onClick={() => setError(undefined)}>
            Fechar
          </button>
        </div>
      ) : null}

      {activeScan ? (
        <div className="scan-bar">
          <span className="scan-pulse" />
          <span>
            Lendo a fonte…{" "}
            {activeScan.supported.toLocaleString("pt-PT")} mídias encontradas
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
            <h1>
              Escolha uma pasta.
              <br />
              O Lume cuida do resto.
            </h1>
            <p>
              Fotos, vídeos e GIFs aparecem numa única galeria. Os arquivos
              originais continuam exatamente onde estão.
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
            <span className="readonly-note">
              Somente leitura · nada será movido ou apagado
            </span>
          </section>
        ) : !hasItems && activeScan ? (
          <section className="empty-state compact">
            <div className="loading-ring" aria-hidden="true" />
            <h2>Procurando suas mídias…</h2>
            <p>
              As primeiras vão aparecer aqui sem esperar a varredura terminar.
            </p>
          </section>
        ) : !hasItems ? (
          <section className="empty-state compact">
            <h2>Nenhuma mídia corresponde ao que você pediu.</h2>
            <p>Limpe a busca ou os filtros, ou adicione outra pasta.</p>
            {search || activeFilterCount > 0 ? (
              <button
                className="toolbar-ghost-button empty-clear-button"
                type="button"
                onClick={() => {
                  setSearchInput("");
                  setSearch("");
                  clearFilters();
                }}
              >
                Limpar busca e filtros
              </button>
            ) : null}
          </section>
        ) : viewMode === "grid" ? (
          <MediaGrid
            items={items}
            thumbnails={thumbnails}
            selectedId={selectedId}
            minCardWidth={minCardWidth}
            hasMore={items.length < total}
            loadingMore={loadingMore}
            onNeedThumbnail={handleNeedThumbnail}
            onOpen={(item) => setSelectedId(item.id)}
            onEndReached={() => void loadMorePage()}
          />
        ) : (
          <MediaList
            items={items}
            thumbnails={thumbnails}
            selectedId={selectedId}
            hasMore={items.length < total}
            loadingMore={loadingMore}
            onNeedThumbnail={handleNeedThumbnail}
            onOpen={(item) => setSelectedId(item.id)}
            onEndReached={() => void loadMorePage()}
          />
        )}
      </main>

      {selected ? (
        <Viewer
          item={selected}
          canPrevious={selectedIndex > 0}
          canNext={selectedIndex < items.length - 1 || items.length < total}
          onPrevious={openPrevious}
          onNext={openNext}
          onClose={() => setSelectedId(undefined)}
        />
      ) : null}

      {sourceManagerOpen ? (
        <SourceManager
          sources={sources}
          scanProgress={scanProgress}
          onClose={() => setSourceManagerOpen(false)}
          onAdd={() => void handleAddSource()}
          onRescan={(sourceId) => void handleRescan(sourceId)}
          onCancel={(sourceId) => void handleCancelScan(sourceId)}
          onRemove={(source) => void handleRemoveSource(source)}
          cacheBytes={cacheBytes}
          clearingCache={clearingCache}
          onClearCache={() => void handleClearCache()}
          onOpenLogs={() =>
            void openLogsFolder().catch((reason: unknown) => {
              setError(reason instanceof Error ? reason.message : String(reason));
            })
          }
        />
      ) : null}
    </div>
  );
}

export default App;
