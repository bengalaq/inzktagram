import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type Locale = "es" | "en";

const LANG_KEY = "inzktagram.locale";

const es = {
  navHome: "Inicio",
  navCreate: "Crear",
  navVerify: "Verificar",
  navSettings: "Ajustes",
  navSwitchUser: "Cambiar de perfil",
  navVerifyFeed: "Verificar feed",
  navLabel: "Navegación",

  langEs: "ES",
  langEn: "EN",
  langSwitchToEn: "Cambiar a English",
  langSwitchToEs: "Switch to Español",

  pickerSub: "Una red social donde el algoritmo lo elegís vos, y podés probarlo.",
  pickerHint:
    "Elegí un perfil. Las cuentas en verde publican despacio; las de rojo, el loop de atención. El feed cambia según el algoritmo.",

  feedLabel: "Feed",
  feedEmpty:
    "Tu feed está vacío. Con el algoritmo Bienestar solo ves cuentas que seguís.",
  refreshFeed: "Regenerar feed",
  maliciousChip: "demo maliciosa",

  proofEllipsis: "prueba: …",
  proofQueued: "prueba: en cola",
  proofProving: "prueba: generándose…",
  proofReady: "prueba lista ({ms} ms)",
  proofFailed: "prueba: falló",

  zkState: "Estado ZK",
  zkAlgorithm: "Algoritmo elegido",
  zkView: "Vista del feed",
  zkProof: "Prueba",
  zkDevMode: "RISC0_DEV_MODE: receipts de desarrollo",
  downloadReceipt: "Descargar receipt",
  people: "Personas en inZKtagram",

  followed: "seguís",
  trending: "tendencia",
  moreOptions: "Más opciones",
  like: "Me gusta",
  comment: "Comentar",
  share: "Compartir",
  save: "Guardar",
  likesCount: "{n} Me gusta",
  viewComments: "Ver los {n} comentarios",
  firstComment: "Sé el primero en comentar",
  timeNow: "ahora",
  timeMins: "hace {n} min",
  timeHours: "hace {n} h",
  timeDays: "hace {n} d",

  newPost: "Nueva publicación",
  close: "Cerrar",
  postPlaceholder: "¿Qué querés compartir hoy? Tomate tu tiempo…",
  publish: "Publicar",
  publishing: "Publicando…",

  settingsTitle: "Tu algoritmo de recomendación",
  settingsIntro:
    "Acá mandás vos. Elegí cómo se ordena tu feed; cada vez que lo abras, la plataforma generará una prueba de conocimiento cero (RISC Zero) de que usó exactamente el algoritmo que elegiste.",
  settingsZk: "Verificabilidad",
  settingsZkBody:
    "El programa que rankea tu feed dentro de la zkVM está identificado públicamente por su image ID. Cualquiera puede recompilarlo desde el código fuente y comprobar que coincide:",
  settingsMalicious: "Demo: servidor malicioso",
  settingsMaliciousBody:
    "Activá esto para simular una plataforma deshonesta: servirá el feed con el algoritmo Engagement aunque afirme usar el que elegiste. Después tocá «Verificar» en tu feed y mirá cómo la prueba ZK detecta el engaño.",
  maliciousOn: "Servidor malicioso ACTIVO",
  maliciousOff: "Servidor honesto",

  alg1Name: "Engagement",
  alg1Tag: "Como las redes de siempre",
  alg1Desc:
    "Maximiza tu tiempo en la plataforma: recencia agresiva, ganchos virales cortos y cuentas que no seguís inyectadas como «novedad». El feed se llena de likes, FOMO y hilos. Es el modelo de negocio de la atención.",
  alg2Name: "Bienestar",
  alg2Tag: "Protege tu atención",
  alg2Desc:
    "Solo cuentas que seguís, en orden mayormente cronológico. Prioriza textos largos, entierra los ganchos cortos y no usa likes ni viralidad. El feed se siente como hablar con gente conocida, sin tragamonedas.",
  alg3Name: "Mixto",
  alg3Tag: "Un punto medio",
  alg3Desc:
    "Combinación ponderada (60% bienestar, 40% engagement): descubrís cosas nuevas sin que el feed se convierta en una máquina tragamonedas.",

  toastFeedError: "Error cargando el feed: {msg}",
  toastPublished: "Publicado. Tu feed se regeneró (con nueva prueba ZK).",
  toastAlgChanged: "Algoritmo cambiado a {name}. Regenerando feed…",
  toastMaliciousOn: "Servidor malicioso activo: andá al feed y verificá la prueba.",
  toastMaliciousOff: "Servidor honesto de nuevo.",

  verifyTitle: "Verificación ZK del feed",
  verifyGenerating: "Generando la prueba STARK en la zkVM…",
  verifyChecking: "Verificando el receipt…",
  verifyHint:
    "La prueba STARK certifica la ejecución del algoritmo dentro de la zkVM de RISC Zero.",
  verifyOk: "Feed verificado: la plataforma usó el algoritmo que elegiste",
  verifyFail: "Verificación FALLIDA: la plataforma no respetó tu elección",
  verifyStark: "Prueba STARK válida para el programa",
  verifyStarkMs: "(verificada en {ms} ms)",
  verifyAlgProved: "Algoritmo probado",
  verifyAlgMatches: "coincide con tu elección",
  verifyFeedHashBefore: "El hash del feed que ves (",
  verifyFeedHashMid: ", calculado en tu navegador) coincide con el certificado en la prueba",
  verifyDemoReveal:
    "Demo: el servidor estaba en modo malicioso. Sirvió el algoritmo {served} mientras afirmaba usar {claimed}. La prueba ZK lo delató.",
  verifyDevMode:
    "El servidor corre con RISC0_DEV_MODE: este receipt es de desarrollo, no una prueba STARK real.",
  journalTitle: "Journal completo (parte pública de la prueba)",
  journalConfig: "Hash de tu configuración",
  journalParams: "Hash de parámetros del algoritmo",
  journalCandidates: "Hash del conjunto de candidatos",
  journalFeed: "Hash del feed resultante",
  journalTime: "Timestamp del cómputo",
  trustlessTitle: "¿No confiás en esta pantalla? Verificalo vos",
  trustlessBefore: "Descargá el receipt, ponelo en la carpeta ",
  trustlessMid: " del proyecto y corré esto desde ",
  trustlessAfter: ":",
  trustlessAudit:
    "Opcional (auditoría del conjunto de entrada): GET /api/audit/<user_id> → dump.json, y agregá --candidates dump.json. El dump no es un transparency log; el chequeo es el hash contra el journal.",
  errTimeout: "timeout esperando la prueba",
  errProver: "el prover falló: {msg}",
  errUnknown: "error desconocido",
} as const;

const en: { [K in keyof typeof es]: string } = {
  navHome: "Home",
  navCreate: "Create",
  navVerify: "Verify",
  navSettings: "Settings",
  navSwitchUser: "Switch profile",
  navVerifyFeed: "Verify feed",
  navLabel: "Navigation",

  langEs: "ES",
  langEn: "EN",
  langSwitchToEn: "Switch to English",
  langSwitchToEs: "Cambiar a Español",

  pickerSub: "A social network where you choose the algorithm — and you can prove it.",
  pickerHint:
    "Pick a profile. Green accounts post slowly; red ones are the attention loop. The feed changes with the algorithm.",

  feedLabel: "Feed",
  feedEmpty:
    "Your feed is empty. With the Wellbeing algorithm you only see accounts you follow.",
  refreshFeed: "Refresh feed",
  maliciousChip: "malicious demo",

  proofEllipsis: "proof: …",
  proofQueued: "proof: queued",
  proofProving: "proof: generating…",
  proofReady: "proof ready ({ms} ms)",
  proofFailed: "proof: failed",

  zkState: "ZK status",
  zkAlgorithm: "Chosen algorithm",
  zkView: "Feed view",
  zkProof: "Proof",
  zkDevMode: "RISC0_DEV_MODE: development receipts",
  downloadReceipt: "Download receipt",
  people: "People on inZKtagram",

  followed: "following",
  trending: "trending",
  moreOptions: "More options",
  like: "Like",
  comment: "Comment",
  share: "Share",
  save: "Save",
  likesCount: "{n} likes",
  viewComments: "View {n} comments",
  firstComment: "Be the first to comment",
  timeNow: "now",
  timeMins: "{n} min ago",
  timeHours: "{n} h ago",
  timeDays: "{n} d ago",

  newPost: "New post",
  close: "Close",
  postPlaceholder: "What do you want to share today? Take your time…",
  publish: "Post",
  publishing: "Posting…",

  settingsTitle: "Your ranking algorithm",
  settingsIntro:
    "You're in charge. Choose how your feed is ordered; every time you open it, the platform will generate a zero-knowledge proof (RISC Zero) that it used exactly the algorithm you picked.",
  settingsZk: "Verifiability",
  settingsZkBody:
    "The program that ranks your feed inside the zkVM is publicly identified by its image ID. Anyone can recompile it from source and check that it matches:",
  settingsMalicious: "Demo: malicious server",
  settingsMaliciousBody:
    "Turn this on to simulate a dishonest platform: it will serve the Engagement feed while claiming to use the one you chose. Then tap “Verify” on your feed and watch the ZK proof catch the lie.",
  maliciousOn: "Malicious server ON",
  maliciousOff: "Honest server",

  alg1Name: "Engagement",
  alg1Tag: "Like the usual networks",
  alg1Desc:
    "Maximizes time on the platform: aggressive recency, short viral hooks, and accounts you don’t follow injected as “novelty”. The feed fills with likes, FOMO and threads. It’s the attention business model.",
  alg2Name: "Wellbeing",
  alg2Tag: "Protect your attention",
  alg2Desc:
    "Only accounts you follow, mostly chronological. It favors long posts, buries short hooks, and ignores likes and virality. The feed feels like talking to people you know — no slot machine.",
  alg3Name: "Mixed",
  alg3Tag: "A middle ground",
  alg3Desc:
    "A weighted mix (60% wellbeing, 40% engagement): you still discover new things without turning the feed into a slot machine.",

  toastFeedError: "Error loading the feed: {msg}",
  toastPublished: "Posted. Your feed was rebuilt (with a new ZK proof).",
  toastAlgChanged: "Algorithm switched to {name}. Rebuilding feed…",
  toastMaliciousOn: "Malicious server on: go to the feed and verify the proof.",
  toastMaliciousOff: "Honest server again.",

  verifyTitle: "ZK feed verification",
  verifyGenerating: "Generating the STARK proof in the zkVM…",
  verifyChecking: "Verifying the receipt…",
  verifyHint:
    "The STARK proof certifies that the algorithm ran inside the RISC Zero zkVM.",
  verifyOk: "Feed verified: the platform used the algorithm you chose",
  verifyFail: "VERIFICATION FAILED: the platform did not honor your choice",
  verifyStark: "Valid STARK proof for program",
  verifyStarkMs: "(verified in {ms} ms)",
  verifyAlgProved: "Proved algorithm",
  verifyAlgMatches: "matches your choice",
  verifyFeedHashBefore: "The hash of the feed you see (",
  verifyFeedHashMid: ", computed in your browser) matches the one committed in the proof",
  verifyDemoReveal:
    "Demo: the server was in malicious mode. It served {served} while claiming {claimed}. The ZK proof caught it.",
  verifyDevMode:
    "The server is running with RISC0_DEV_MODE: this receipt is a development stub, not a real STARK proof.",
  journalTitle: "Full journal (public part of the proof)",
  journalConfig: "Your config hash",
  journalParams: "Algorithm parameters hash",
  journalCandidates: "Candidate set hash",
  journalFeed: "Resulting feed hash",
  journalTime: "Computation timestamp",
  trustlessTitle: "Don’t trust this screen? Verify it yourself",
  trustlessBefore: "Download the receipt, put it in the project’s ",
  trustlessMid: " folder, and run this from ",
  trustlessAfter: ":",
  trustlessAudit:
    "Optional (input-set audit): GET /api/audit/<user_id> → dump.json, then add --candidates dump.json. The dump is not a transparency log; the check is the hash against the journal.",
  errTimeout: "timed out waiting for the proof",
  errProver: "the prover failed: {msg}",
  errUnknown: "unknown error",
};

const dict = { es, en };

export type MessageKey = keyof typeof es;

function fill(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? ""));
}

function readSaved(): Locale {
  try {
    const v = localStorage.getItem(LANG_KEY);
    if (v === "en" || v === "es") return v;
  } catch {
    /* ignore */
  }
  return "es";
}

type I18nValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, vars?: Record<string, string | number>) => string;
  algorithmName: (id: number) => string;
  algorithms: { id: number; name: string; tagline: string; description: string }[];
  timeAgo: (epochSecs: number) => string;
  localeTag: string;
};

const I18nContext = createContext<I18nValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(readSaved);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    try {
      localStorage.setItem(LANG_KEY, next);
    } catch {
      /* ignore */
    }
  }, []);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = useMemo<I18nValue>(() => {
    const t = (key: MessageKey, vars?: Record<string, string | number>) =>
      fill(dict[locale][key], vars);

    const algorithmName = (id: number) => {
      if (id === 1) return t("alg1Name");
      if (id === 2) return t("alg2Name");
      if (id === 3) return t("alg3Name");
      return `#${id}`;
    };

    const algorithms = [
      { id: 1, name: t("alg1Name"), tagline: t("alg1Tag"), description: t("alg1Desc") },
      { id: 2, name: t("alg2Name"), tagline: t("alg2Tag"), description: t("alg2Desc") },
      { id: 3, name: t("alg3Name"), tagline: t("alg3Tag"), description: t("alg3Desc") },
    ];

    const timeAgo = (epochSecs: number) => {
      const mins = Math.max(0, Math.floor(Date.now() / 1000 - epochSecs) / 60);
      if (mins < 1) return t("timeNow");
      if (mins < 60) return t("timeMins", { n: Math.floor(mins) });
      const hours = mins / 60;
      if (hours < 24) return t("timeHours", { n: Math.floor(hours) });
      return t("timeDays", { n: Math.floor(hours / 24) });
    };

    return {
      locale,
      setLocale,
      t,
      algorithmName,
      algorithms,
      timeAgo,
      localeTag: locale === "es" ? "es" : "en",
    };
  }, [locale, setLocale]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n requires I18nProvider");
  return ctx;
}
