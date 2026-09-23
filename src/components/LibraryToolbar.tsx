import type { MediaSort, Source, ThemePreference } from "../lib/types";

interface LibraryToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  mediaType: "all" | "image" | "video";
  onMediaTypeChange: (value: "all" | "image" | "video") => void;
  extensions: string[];
  selectedExtensions: string[];
  onToggleExtension: (extension: string) => void;
  sources: Source[];
  selectedSourceIds: number[];
  onToggleSource: (sourceId: number) => void;
  dateFrom: string;
  dateTo: string;
  onDateFromChange: (value: string) => void;
  onDateToChange: (value: string) => void;
  minSizeMb: string;
  maxSizeMb: string;
  onMinSizeMbChange: (value: string) => void;
  onMaxSizeMbChange: (value: string) => void;
  sort: MediaSort;
  onSortChange: (value: MediaSort) => void;
  viewMode: "grid" | "list";
  onViewModeChange: (value: "grid" | "list") => void;
  minCardWidth: number;
  onMinCardWidthChange: (value: number) => void;
  theme: ThemePreference;
  onThemeChange: (value: ThemePreference) => void;
  activeFilterCount: number;
  onClearFilters: () => void;
}

export function LibraryToolbar(props: LibraryToolbarProps) {
  return (
    <section className="library-toolbar" aria-label="Controles da biblioteca">
      <label className="search-control">
        <span aria-hidden="true">⌕</span>
        <input
          value={props.search}
          onChange={(event) => props.onSearchChange(event.currentTarget.value)}
          placeholder="Pesquisar por nome…"
          aria-label="Pesquisar por nome"
        />
      </label>

      <select
        className="toolbar-select"
        value={props.mediaType}
        onChange={(event) =>
          props.onMediaTypeChange(
            event.currentTarget.value as "all" | "image" | "video",
          )
        }
        aria-label="Filtrar por tipo"
      >
        <option value="all">Tudo</option>
        <option value="image">Imagens</option>
        <option value="video">Vídeos</option>
      </select>

      <details className="filter-menu">
        <summary>
          Extensão
          {props.selectedExtensions.length
            ? ` · ${props.selectedExtensions.length}`
            : ""}
        </summary>
        <div className="filter-menu-panel">
          {props.extensions.length === 0 ? (
            <span className="filter-empty">Nenhuma extensão</span>
          ) : null}
          {props.extensions.map((extension) => (
            <label key={extension}>
              <input
                type="checkbox"
                checked={props.selectedExtensions.includes(extension)}
                onChange={() => props.onToggleExtension(extension)}
              />
              {extension.toUpperCase()}
            </label>
          ))}
        </div>
      </details>

      <details className="filter-menu">
        <summary>
          Fonte
          {props.selectedSourceIds.length
            ? ` · ${props.selectedSourceIds.length}`
            : ""}
        </summary>
        <div className="filter-menu-panel filter-menu-wide">
          {props.sources.map((source) => (
            <label key={source.id} title={source.rootPath}>
              <input
                type="checkbox"
                checked={props.selectedSourceIds.includes(source.id)}
                onChange={() => props.onToggleSource(source.id)}
              />
              <span className="filter-source-copy">
                <strong>{source.displayName}</strong>
                <small>{source.rootPath}</small>
              </span>
            </label>
          ))}
        </div>
      </details>

      <details className="filter-menu">
        <summary>
          Mais filtros{props.activeFilterCount ? ` · ${props.activeFilterCount}` : ""}
        </summary>
        <div className="filter-menu-panel filter-menu-form">
          <label>
            <span>Data inicial</span>
            <input
              type="date"
              value={props.dateFrom}
              onChange={(event) =>
                props.onDateFromChange(event.currentTarget.value)
              }
            />
          </label>
          <label>
            <span>Data final</span>
            <input
              type="date"
              value={props.dateTo}
              onChange={(event) => props.onDateToChange(event.currentTarget.value)}
            />
          </label>
          <label>
            <span>Mín. MB</span>
            <input
              type="number"
              min="0"
              step="1"
              value={props.minSizeMb}
              onChange={(event) =>
                props.onMinSizeMbChange(event.currentTarget.value)
              }
              placeholder="0"
            />
          </label>
          <label>
            <span>Máx. MB</span>
            <input
              type="number"
              min="0"
              step="1"
              value={props.maxSizeMb}
              onChange={(event) =>
                props.onMaxSizeMbChange(event.currentTarget.value)
              }
              placeholder="∞"
            />
          </label>
        </div>
      </details>

      <select
        className="toolbar-select sort-select"
        value={props.sort}
        onChange={(event) =>
          props.onSortChange(event.currentTarget.value as MediaSort)
        }
        aria-label="Ordenar biblioteca"
      >
        <option value="date_desc">Mais recentes</option>
        <option value="date_asc">Mais antigas</option>
        <option value="name_asc">Nome A–Z</option>
        <option value="name_desc">Nome Z–A</option>
        <option value="size_desc">Maiores</option>
        <option value="size_asc">Menores</option>
      </select>

      {props.activeFilterCount > 0 ? (
        <button
          className="toolbar-ghost-button"
          type="button"
          onClick={props.onClearFilters}
        >
          Limpar
        </button>
      ) : null}

      <span className="toolbar-spacer" />

      {props.viewMode === "grid" ? (
        <label className="density-control" title="Tamanho das miniaturas">
          <span>−</span>
          <input
            type="range"
            min="120"
            max="320"
            step="10"
            value={props.minCardWidth}
            onChange={(event) =>
              props.onMinCardWidthChange(Number(event.currentTarget.value))
            }
          />
          <span>＋</span>
        </label>
      ) : null}

      <select
        className="toolbar-select theme-select"
        value={props.theme}
        onChange={(event) =>
          props.onThemeChange(event.currentTarget.value as ThemePreference)
        }
        aria-label="Tema"
        title="Tema"
      >
        <option value="system">Tema · Sistema</option>
        <option value="dark">Tema · Escuro</option>
        <option value="light">Tema · Claro</option>
      </select>

      <div className="view-toggle" role="group" aria-label="Modo de visualização">
        <button
          type="button"
          className={props.viewMode === "grid" ? "active" : ""}
          onClick={() => props.onViewModeChange("grid")}
          aria-label="Visualização em grade"
        >
          ▦
        </button>
        <button
          type="button"
          className={props.viewMode === "list" ? "active" : ""}
          onClick={() => props.onViewModeChange("list")}
          aria-label="Visualização em lista"
        >
          ☷
        </button>
      </div>
    </section>
  );
}
