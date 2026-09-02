import { useI18n } from "../i18n";

export default function LangToggle({ className = "" }: { className?: string }) {
  const { locale, setLocale, t } = useI18n();

  return (
    <div className={`lang-toggle ${className}`.trim()} role="group" aria-label="Language">
      <button
        type="button"
        className={locale === "es" ? "active" : ""}
        aria-pressed={locale === "es"}
        title={t("langSwitchToEs")}
        onClick={() => setLocale("es")}
      >
        {t("langEs")}
      </button>
      <button
        type="button"
        className={locale === "en" ? "active" : ""}
        aria-pressed={locale === "en"}
        title={t("langSwitchToEn")}
        onClick={() => setLocale("en")}
      >
        {t("langEn")}
      </button>
    </div>
  );
}
