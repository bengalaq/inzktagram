import { useCallback, useEffect, useRef, useState } from "react";
import {
  algorithmName,
  api,
  FeedResponse,
  ProofStatus,
  StateResponse,
  User,
} from "./api";
import { computeFeedHash } from "./hash";
import BottomNav from "./components/BottomNav";
import { RefreshIcon, ShieldIcon } from "./components/Icons";
import NewPostModal from "./components/NewPostModal";
import PostCard from "./components/PostCard";
import SettingsPage from "./components/SettingsPage";
import SideNav, { Page } from "./components/SideNav";
import UserPicker from "./components/UserPicker";
import VerifyModal, { VerificationResult } from "./components/VerifyModal";

const USER_KEY = "inzktagram.user_id";

export default function App() {
  const [users, setUsers] = useState<User[]>([]);
  const [user, setUser] = useState<User | null>(null);
  const [page, setPage] = useState<Page>("feed");
  const [state, setState] = useState<StateResponse | null>(null);
  const [feed, setFeed] = useState<FeedResponse | null>(null);
  const [feedAlgorithm, setFeedAlgorithm] = useState<number>(2);
  const [loadingFeed, setLoadingFeed] = useState(false);
  const [proof, setProof] = useState<ProofStatus | null>(null);
  const [saving, setSaving] = useState(false);

  const [showNewPost, setShowNewPost] = useState(false);
  const [showVerify, setShowVerify] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [verifyLabel, setVerifyLabel] = useState("");
  const [verifyResult, setVerifyResult] = useState<VerificationResult | null>(null);
  const [verifyError, setVerifyError] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const verifyRun = useRef(0);

  const showToast = (msg: string) => {
    setToast(msg);
    window.setTimeout(() => setToast(null), 3500);
  };

  useEffect(() => {
    api.users().then((us) => {
      setUsers(us);
      const saved = Number(localStorage.getItem(USER_KEY));
      const found = us.find((u) => u.id === saved);
      if (found) setUser(found);
    });
  }, []);

  const loadFeed = useCallback(async (u: User) => {
    setLoadingFeed(true);
    setProof(null);
    setVerifyResult(null);
    setVerifyError(null);
    try {
      const st = await api.state(u.id);
      setState(st);
      setFeedAlgorithm(st.algorithm_id);
      const f = await api.feed(u.id);
      setFeed(f);
    } catch (e) {
      showToast(`Error cargando el feed: ${(e as Error).message}`);
    } finally {
      setLoadingFeed(false);
    }
  }, []);

  useEffect(() => {
    if (user) loadFeed(user);
  }, [user, loadFeed]);

  // Polling del estado de la prueba mientras está pendiente / generándose.
  useEffect(() => {
    if (!feed) return;
    let cancelled = false;
    const tick = async () => {
      try {
        const p = await api.proofStatus(feed.view_id);
        if (!cancelled) setProof(p);
        return p.status === "proved" || p.status === "failed";
      } catch {
        return false;
      }
    };
    tick();
    const id = window.setInterval(async () => {
      if (await tick()) window.clearInterval(id);
    }, 2500);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [feed]);

  const pickUser = (u: User) => {
    localStorage.setItem(USER_KEY, String(u.id));
    setUser(u);
    setPage("feed");
  };

  const publish = async (content: string) => {
    if (!user) return;
    await api.createPost(user.id, content);
    showToast("Publicado. Tu feed se regeneró (con nueva prueba ZK).");
    await loadFeed(user);
  };

  const selectAlgorithm = async (id: number) => {
    if (!user) return;
    setSaving(true);
    try {
      await api.setAlgorithm(user.id, id);
      showToast(`Algoritmo cambiado a ${algorithmName(id)}. Regenerando feed…`);
      await loadFeed(user);
    } finally {
      setSaving(false);
    }
  };

  const toggleMalicious = async (enabled: boolean) => {
    if (!user) return;
    await api.setMalicious(enabled);
    showToast(
      enabled
        ? "Servidor malicioso activo: andá al feed y verificá la prueba."
        : "Servidor honesto de nuevo."
    );
    await loadFeed(user);
  };

  const verify = async () => {
    if (!feed) return;
    const run = ++verifyRun.current;
    setShowVerify(true);
    setVerifying(true);
    setVerifyResult(null);
    setVerifyError(null);
    try {
      // 1. Esperar a que el worker termine la prueba (si aún no está).
      setVerifyLabel("Generando la prueba STARK en la zkVM…");
      let status = proof ?? (await api.proofStatus(feed.view_id));
      const deadline = Date.now() + 15 * 60 * 1000;
      while (status.status === "pending" || status.status === "proving") {
        if (Date.now() > deadline) throw new Error("timeout esperando la prueba");
        await new Promise((r) => setTimeout(r, 2000));
        if (verifyRun.current !== run) return;
        status = await api.proofStatus(feed.view_id);
        setProof(status);
      }
      if (status.status === "failed") {
        throw new Error(`el prover falló: ${status.error ?? "error desconocido"}`);
      }

      // 2. Verificación criptográfica del receipt.
      setVerifyLabel("Verificando el receipt…");
      const server = await api.verifyProof(feed.view_id);

      // 3. Chequeos del lado del cliente, sin confiar en el servidor:
      //    el hash del feed renderizado y el algoritmo elegido.
      const localFeedHash = await computeFeedHash(feed.posts.map((p) => p.id));
      const j = server.journal ?? null;
      const localFeedMatches = !!j && j.feed_hash === localFeedHash;
      const algorithmMatches = !!j && j.algorithm_id === feedAlgorithm;
      const verdict = server.proof_valid && localFeedMatches && algorithmMatches;

      if (verifyRun.current !== run) return;
      setVerifyResult({
        server,
        localFeedHash,
        localFeedMatches,
        algorithmExpected: feedAlgorithm,
        algorithmMatches,
        verdict,
      });
    } catch (e) {
      if (verifyRun.current === run) setVerifyError((e as Error).message);
    } finally {
      if (verifyRun.current === run) setVerifying(false);
    }
  };

  if (!user) {
    return users.length ? (
      <UserPicker users={users} onPick={pickUser} />
    ) : (
      <div className="picker-screen">
        <div className="spinner" />
      </div>
    );
  }

  const proofChip = (() => {
    if (!proof) return { text: "prueba: …", cls: "" };
    switch (proof.status) {
      case "pending":
        return { text: "prueba: en cola", cls: "chip-wait" };
      case "proving":
        return { text: "prueba: generándose…", cls: "chip-wait" };
      case "proved":
        return { text: `prueba lista (${proof.proving_ms} ms)`, cls: "chip-ok" };
      default:
        return { text: "prueba: falló", cls: "chip-fail" };
    }
  })();

  const proofBadge = verifyResult ? (verifyResult.verdict ? "ok" : "fail") : null;
  const others = users.filter((u) => u.id !== user.id);

  return (
    <div className="app">
      <SideNav
        user={user}
        page={page}
        proofBadge={proofBadge}
        onGo={setPage}
        onNewPost={() => setShowNewPost(true)}
        onVerify={verify}
        onSwitchUser={() => {
          localStorage.removeItem(USER_KEY);
          setUser(null);
          setState(null);
          setFeed(null);
        }}
      />

      <header className="mobile-top">
        <h1 className="logo" onClick={() => setPage("feed")}>
          inZKtagram
        </h1>
        <button className="icon-btn" onClick={verify} aria-label="Verificar feed">
          <span className="nav-icon-badge">
            <ShieldIcon />
            {proofBadge && <span className={`badge-dot ${proofBadge}`} />}
          </span>
        </button>
      </header>

      <main className="main">
        {page === "feed" && (
          <div className="feed-col">
            <div className="stories">
              {others.map((u) => (
                <div key={u.id} className="story">
                  <span className="story-ring">
                    <span className="avatar avatar-md" style={{ background: u.avatar_color }}>
                      {u.display_name[0]}
                    </span>
                  </span>
                  <span className="story-name">{u.username.split(".")[0]}</span>
                </div>
              ))}
            </div>

            <div className="verify-bar">
              <span className="alg-chip">
                Feed: <strong>{algorithmName(feedAlgorithm)}</strong>
              </span>
              <span className={`chip ${proofChip.cls}`}>{proofChip.text}</span>
              {state?.malicious && <span className="chip chip-fail">demo maliciosa</span>}
              <span className="verify-bar-spacer" />
              <button
                className="icon-btn"
                title="Regenerar feed"
                onClick={() => loadFeed(user)}
              >
                <RefreshIcon size={18} />
              </button>
              <button className="btn btn-verify" onClick={verify}>
                <ShieldIcon size={16} /> Verificar
              </button>
            </div>

            {loadingFeed && <div className="feed-loading"><div className="spinner" /></div>}
            {!loadingFeed && feed?.posts.map((p) => <PostCard key={p.id} post={p} />)}
            {!loadingFeed && feed && feed.posts.length === 0 && (
              <p className="muted feed-empty">
                Tu feed está vacío. Con el algoritmo Bienestar solo ves cuentas
                que seguís.
              </p>
            )}
          </div>
        )}

        {page === "settings" && state && (
          <SettingsPage
            algorithmId={state.algorithm_id}
            malicious={state.malicious}
            imageId={state.image_id}
            saving={saving}
            onSelect={selectAlgorithm}
            onToggleMalicious={toggleMalicious}
          />
        )}
      </main>

      <aside className="rightbar">
        <div className="me-card">
          <span className="avatar avatar-md" style={{ background: user.avatar_color }}>
            {user.display_name[0]}
          </span>
          <div>
            <div className="me-name">{user.display_name}</div>
            <div className="muted">@{user.username}</div>
          </div>
        </div>

        <div className="zk-card">
          <h3>
            <ShieldIcon size={16} /> Estado ZK
          </h3>
          <dl>
            <dt>Algoritmo elegido</dt>
            <dd>{algorithmName(state?.algorithm_id ?? feedAlgorithm)}</dd>
            <dt>Vista del feed</dt>
            <dd>#{feed?.view_id ?? "—"}</dd>
            <dt>Prueba</dt>
            <dd>{proofChip.text}</dd>
          </dl>
          {proof?.dev_mode && (
            <p className="dev-note">RISC0_DEV_MODE: receipts de desarrollo</p>
          )}
          {proof?.status === "proved" && feed && (
            <a
              className="btn btn-ghost btn-full"
              href={api.receiptUrl(feed.view_id)}
              download={`inzktagram_view_${feed.view_id}.receipt`}
            >
              Descargar receipt
            </a>
          )}
        </div>

        <div className="suggestions">
          <h4>Personas en inZKtagram</h4>
          {others.slice(0, 5).map((u) => (
            <div key={u.id} className="suggestion">
              <span className="avatar avatar-sm" style={{ background: u.avatar_color }}>
                {u.display_name[0]}
              </span>
              <span>{u.username}</span>
            </div>
          ))}
        </div>
      </aside>

      <BottomNav
        user={user}
        page={page}
        proofBadge={proofBadge}
        onGo={setPage}
        onNewPost={() => setShowNewPost(true)}
        onVerify={verify}
      />

      <NewPostModal
        open={showNewPost}
        onClose={() => setShowNewPost(false)}
        onPublish={publish}
      />
      <VerifyModal
        open={showVerify}
        loading={verifying}
        loadingLabel={verifyLabel}
        result={verifyResult}
        error={verifyError}
        viewId={feed?.view_id ?? null}
        receiptUrl={feed ? api.receiptUrl(feed.view_id) : null}
        onClose={() => {
          verifyRun.current++;
          setShowVerify(false);
          setVerifying(false);
        }}
      />

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
