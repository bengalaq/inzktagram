import { algorithmName, VerifyResponse } from "../api";
import { AlertIcon, CheckIcon, DownloadIcon, XIcon } from "./Icons";

export interface VerificationResult {
  server: VerifyResponse;
  localFeedHash: string;
  localFeedMatches: boolean;
  algorithmExpected: number;
  algorithmMatches: boolean;
  verdict: boolean;
}

interface Props {
  open: boolean;
  loading: boolean;
  loadingLabel: string;
  result: VerificationResult | null;
  error: string | null;
  viewId: number | null;
  receiptUrl: string | null;
  onClose: () => void;
}

function Hash({ value }: { value: string }) {
  return (
    <code className="hash" title={value}>
      {value.slice(0, 10)}…{value.slice(-6)}
    </code>
  );
}

function CheckRow({ ok, children }: { ok: boolean; children: React.ReactNode }) {
  return (
    <li className={`check-row ${ok ? "ok" : "fail"}`}>
      <span className="check-icon">{ok ? <CheckIcon size={16} /> : <XIcon size={16} />}</span>
      <span>{children}</span>
    </li>
  );
}

export default function VerifyModal({
  open,
  loading,
  loadingLabel,
  result,
  error,
  viewId,
  receiptUrl,
  onClose,
}: Props) {
  if (!open) return null;
  const j = result?.server.journal;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal-head">
          <h2>Verificación ZK del feed</h2>
          <button className="icon-btn" onClick={onClose} aria-label="Cerrar">
            <XIcon />
          </button>
        </header>

        {loading && (
          <div className="verify-loading">
            <div className="spinner" />
            <p>{loadingLabel}</p>
            <p className="muted">
              La prueba STARK certifica la ejecución del algoritmo dentro de la
              zkVM de RISC Zero.
            </p>
          </div>
        )}

        {error && !loading && (
          <div className="verify-verdict fail">
            <AlertIcon /> {error}
          </div>
        )}

        {result && !loading && (
          <>
            <div className={`verify-verdict ${result.verdict ? "ok" : "fail"}`}>
              {result.verdict ? (
                <>
                  <CheckIcon /> Feed verificado: la plataforma usó el algoritmo
                  que elegiste
                </>
              ) : (
                <>
                  <XIcon /> Verificación FALLIDA: la plataforma no respetó tu
                  elección
                </>
              )}
            </div>

            <ul className="check-list">
              <CheckRow ok={result.server.checks.proof_valid}>
                Prueba STARK válida para el programa{" "}
                <Hash value={result.server.image_id} /> (verificada en{" "}
                {result.server.verify_ms} ms)
              </CheckRow>
              <CheckRow ok={result.algorithmMatches}>
                Algoritmo probado{" "}
                <strong>{j ? `${j.algorithm_id} · ${j.algorithm_name}` : "—"}</strong>{" "}
                coincide con tu elección{" "}
                <strong>
                  {result.algorithmExpected} · {algorithmName(result.algorithmExpected)}
                </strong>
              </CheckRow>
              <CheckRow ok={result.localFeedMatches}>
                El hash del feed que ves (<Hash value={result.localFeedHash} />,
                calculado en tu navegador) coincide con el certificado en la
                prueba {j && <Hash value={j.feed_hash} />}
              </CheckRow>
            </ul>

            {!result.verdict &&
              result.server.algorithm_served !== result.server.algorithm_claimed && (
                <div className="demo-reveal">
                  <AlertIcon size={18} />
                  <span>
                    Demo: el servidor estaba en modo malicioso. Sirvió el
                    algoritmo{" "}
                    <strong>{algorithmName(result.server.algorithm_served)}</strong>{" "}
                    mientras afirmaba usar{" "}
                    <strong>{algorithmName(result.server.algorithm_claimed)}</strong>
                    . La prueba ZK lo delató.
                  </span>
                </div>
              )}

            {result.server.dev_mode && (
              <div className="dev-warning">
                <AlertIcon size={18} />
                <span>
                  El servidor corre con RISC0_DEV_MODE: este receipt es de
                  desarrollo, no una prueba STARK real.
                </span>
              </div>
            )}

            {j && (
              <details className="journal-details">
                <summary>Journal completo (parte pública de la prueba)</summary>
                <dl>
                  <dt>Hash de tu configuración</dt>
                  <dd><Hash value={j.config_hash} /></dd>
                  <dt>Hash de parámetros del algoritmo</dt>
                  <dd><Hash value={j.params_hash} /></dd>
                  <dt>Hash del conjunto de candidatos</dt>
                  <dd><Hash value={j.candidates_hash} /></dd>
                  <dt>Hash del feed resultante</dt>
                  <dd><Hash value={j.feed_hash} /></dd>
                  <dt>Timestamp del cómputo</dt>
                  <dd>{new Date(j.timestamp * 1000).toLocaleString("es")}</dd>
                </dl>
              </details>
            )}

            <div className="verify-trustless">
              <h3>¿No confiás en esta pantalla? Verificalo vos</h3>
              <p>
                Descargá el receipt, ponelo en la carpeta{" "}
                <code>download_receipts</code> del proyecto y corré esto desde{" "}
                <code>inzktagram/</code>:
              </p>
              {receiptUrl && (
                <a
                  className="btn btn-ghost"
                  href={receiptUrl}
                  download={`inzktagram_view_${viewId}.receipt`}
                >
                  <DownloadIcon size={16} /> Descargar receipt
                </a>
              )}
              <pre className="cli-sample">{`.\\verify.cmd inzktagram_view_${viewId}.receipt --expect-algorithm ${result.algorithmExpected} --expect-feed-hash ${result.localFeedHash}`}</pre>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
