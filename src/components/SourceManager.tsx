import type { ScanProgress, Source } from "../lib/types";

interface SourceManagerProps {
  sources: Source[];
  scanProgress: Map<number, ScanProgress>;
  onClose: () => void;
  onAdd: () => void;
  onRescan: (sourceId: number) => void;
  onCancel: (sourceId: number) => void;
  onRemove: (source: Source) => void;
}

function statusLabel(status: string): string {
  switch (status) {
    case "scanning":
      return "Escaneando";
    case "offline":
      return "Offline";
    case "error":
      return "Atenção";
    default:
      return "Online";
  }
}

export function SourceManager({
  sources,
  scanProgress,
  onClose,
  onAdd,
  onRescan,
  onCancel,
  onRemove,
}: SourceManagerProps) {
  return (
    <div className="source-manager-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="source-manager"
        role="dialog"
        aria-modal="true"
        aria-label="Gerenciar fontes"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="source-manager-header">
          <div>
            <span className="eyebrow">Biblioteca local</span>
            <h2>Fontes</h2>
          </div>
          <button className="icon-button" type="button" onClick={onClose} aria-label="Fechar">
            ×
          </button>
        </header>

        <div className="source-manager-list">
          {sources.map((source) => {
            const progress = scanProgress.get(source.id);
            const scanning = Boolean(progress && !progress.done) || source.status === "scanning";
            const status = scanning ? "scanning" : source.status;

            return (
              <article className="source-manager-row" key={source.id}>
                <div className="source-manager-main">
                  <span className={`status-dot status-${status}`} />
                  <div>
                    <strong>{source.displayName}</strong>
                    <span>{source.rootPath}</span>
                  </div>
                </div>

                <div className="source-manager-status">
                  <strong>{statusLabel(status)}</strong>
                  {progress ? (
                    <span>
                      {progress.supported.toLocaleString("pt-PT")} mídias
                      {progress.errors ? ` · ${progress.errors} erros` : ""}
                    </span>
                  ) : source.lastScanFinishedAt ? (
                    <span>Último scan: {source.lastScanFinishedAt}</span>
                  ) : (
                    <span>Ainda sem scan concluído</span>
                  )}
                  {progress?.message ? <span>{progress.message}</span> : null}
                </div>

                <div className="source-manager-actions">
                  {scanning ? (
                    <button type="button" onClick={() => onCancel(source.id)}>
                      Cancelar
                    </button>
                  ) : (
                    <button type="button" onClick={() => onRescan(source.id)}>
                      Re-scan
                    </button>
                  )}
                  <button
                    className="danger-button"
                    type="button"
                    disabled={scanning}
                    onClick={() => onRemove(source)}
                  >
                    Remover
                  </button>
                </div>
              </article>
            );
          })}
        </div>

        <footer className="source-manager-footer">
          <span>Remover uma fonte só limpa o catálogo do Lume. Os arquivos físicos não são tocados.</span>
          <button className="primary-button" type="button" onClick={onAdd}>
            ＋ Adicionar pasta
          </button>
        </footer>
      </section>
    </div>
  );
}
