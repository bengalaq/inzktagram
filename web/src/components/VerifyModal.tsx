import { VerifyResponse } from "../api";
import { useI18n } from "../i18n";
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
  const { t, algorithmName, localeTag } = useI18n();
  if (!open) return null;
  const j = result?.server.journal;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal-head">
          <h2>{t("verifyTitle")}</h2>
          <button className="icon-btn" onClick={onClose} aria-label={t("close")}>
            <XIcon />
          </button>
        </header>

        {loading && (
          <div className="verify-loading">
            <div className="spinner" />
            <p>{loadingLabel}</p>
            <p className="muted">{t("verifyHint")}</p>
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
                  <CheckIcon /> {t("verifyOk")}
                </>
              ) : (
                <>
                  <XIcon /> {t("verifyFail")}
                </>
              )}
            </div>

            <ul className="check-list">
              <CheckRow ok={result.server.checks.proof_valid}>
                {t("verifyStark")} <Hash value={result.server.image_id} />{" "}
                {t("verifyStarkMs", { ms: result.server.verify_ms })}
              </CheckRow>
              <CheckRow ok={result.algorithmMatches}>
                {t("verifyAlgProved")}{" "}
                <strong>
                  {j
                    ? `${j.algorithm_id} · ${algorithmName(j.algorithm_id)}`
                    : "—"}
                </strong>{" "}
                {t("verifyAlgMatches")}{" "}
                <strong>
                  {result.algorithmExpected} · {algorithmName(result.algorithmExpected)}
                </strong>
              </CheckRow>
              <CheckRow ok={result.localFeedMatches}>
                {t("verifyFeedHashBefore")}
                <Hash value={result.localFeedHash} />
                {t("verifyFeedHashMid")} {j && <Hash value={j.feed_hash} />}
              </CheckRow>
            </ul>

            {!result.verdict &&
              result.server.algorithm_served !== result.server.algorithm_claimed && (
                <div className="demo-reveal">
                  <AlertIcon size={18} />
                  <span>
                    {t("verifyDemoReveal", {
                      served: algorithmName(result.server.algorithm_served),
                      claimed: algorithmName(result.server.algorithm_claimed),
                    })}
                  </span>
                </div>
              )}

            {result.server.dev_mode && (
              <div className="dev-warning">
                <AlertIcon size={18} />
                <span>{t("verifyDevMode")}</span>
              </div>
            )}

            {j && (
              <details className="journal-details">
                <summary>{t("journalTitle")}</summary>
                <dl>
                  <dt>{t("journalConfig")}</dt>
                  <dd><Hash value={j.config_hash} /></dd>
                  <dt>{t("journalParams")}</dt>
                  <dd><Hash value={j.params_hash} /></dd>
                  <dt>{t("journalCandidates")}</dt>
                  <dd><Hash value={j.candidates_hash} /></dd>
                  <dt>{t("journalFeed")}</dt>
                  <dd><Hash value={j.feed_hash} /></dd>
                  <dt>{t("journalTime")}</dt>
                  <dd>{new Date(j.timestamp * 1000).toLocaleString(localeTag)}</dd>
                </dl>
              </details>
            )}

            <div className="verify-trustless">
              <h3>{t("trustlessTitle")}</h3>
              <p>
                {t("trustlessBefore")}
                <code>download_receipts</code>
                {t("trustlessMid")}
                <code>inzktagram/</code>
                {t("trustlessAfter")}
              </p>
              {receiptUrl && (
                <a
                  className="btn btn-ghost"
                  href={receiptUrl}
                  download={`inzktagram_view_${viewId}.receipt`}
                >
                  <DownloadIcon size={16} /> {t("downloadReceipt")}
                </a>
              )}
              <pre className="cli-sample">{`.\\verify.cmd inzktagram_view_${viewId}.receipt --expect-algorithm ${result.algorithmExpected} --expect-feed-hash ${result.localFeedHash}`}</pre>
              <p className="muted">{t("trustlessAudit")}</p>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
