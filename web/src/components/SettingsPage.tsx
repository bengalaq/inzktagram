import { useI18n } from "../i18n";
import { AlertIcon, CheckIcon, ShieldIcon } from "./Icons";

interface Props {
  algorithmId: number;
  malicious: boolean;
  imageId: string;
  saving: boolean;
  onSelect: (id: number) => void;
  onToggleMalicious: (enabled: boolean) => void;
}

export default function SettingsPage({
  algorithmId,
  malicious,
  imageId,
  saving,
  onSelect,
  onToggleMalicious,
}: Props) {
  const { t, algorithms } = useI18n();
  return (
    <div className="settings">
      <h2 className="page-title">{t("settingsTitle")}</h2>
      <p className="muted settings-intro">{t("settingsIntro")}</p>

      <div className="alg-cards">
        {algorithms.map((alg) => {
          const active = alg.id === algorithmId;
          return (
            <button
              key={alg.id}
              className={`alg-card ${active ? "active" : ""}`}
              onClick={() => !saving && onSelect(alg.id)}
              disabled={saving}
            >
              <div className="alg-card-head">
                <span className="alg-num">{alg.id}</span>
                <div>
                  <h3>{alg.name}</h3>
                  <span className="alg-tagline">{alg.tagline}</span>
                </div>
                {active && (
                  <span className="alg-check">
                    <CheckIcon size={18} />
                  </span>
                )}
              </div>
              <p>{alg.description}</p>
            </button>
          );
        })}
      </div>

      <section className="settings-zk">
        <h3>
          <ShieldIcon size={18} /> {t("settingsZk")}
        </h3>
        <p className="muted">{t("settingsZkBody")}</p>
        <code className="hash hash-block" title={imageId}>
          {imageId}
        </code>
      </section>

      <section className="settings-demo">
        <h3>
          <AlertIcon size={18} /> {t("settingsMalicious")}
        </h3>
        <p className="muted">{t("settingsMaliciousBody")}</p>
        <label className="toggle">
          <input
            type="checkbox"
            checked={malicious}
            onChange={(e) => onToggleMalicious(e.target.checked)}
          />
          <span className="toggle-track" />
          <span>{malicious ? t("maliciousOn") : t("maliciousOff")}</span>
        </label>
      </section>
    </div>
  );
}
