import { User } from "../api";
import { GearIcon, HomeIcon, PlusIcon, ShieldIcon } from "./Icons";
import { Page } from "./SideNav";

interface Props {
  user: User;
  page: Page;
  proofBadge: "ok" | "fail" | null;
  onGo: (page: Page) => void;
  onNewPost: () => void;
  onVerify: () => void;
}

export default function BottomNav({
  user,
  page,
  proofBadge,
  onGo,
  onNewPost,
  onVerify,
}: Props) {
  return (
    <nav className="bottom-nav" aria-label="Navegación">
      <button
        className={`bottom-nav-item ${page === "feed" ? "active" : ""}`}
        onClick={() => onGo("feed")}
        aria-label="Inicio"
      >
        <HomeIcon />
      </button>
      <button className="bottom-nav-item" onClick={onNewPost} aria-label="Crear">
        <PlusIcon />
      </button>
      <button className="bottom-nav-item" onClick={onVerify} aria-label="Verificar">
        <span className="nav-icon-badge">
          <ShieldIcon />
          {proofBadge && <span className={`badge-dot ${proofBadge}`} />}
        </span>
      </button>
      <button
        className={`bottom-nav-item ${page === "settings" ? "active" : ""}`}
        onClick={() => onGo("settings")}
        aria-label="Ajustes"
      >
        <GearIcon />
      </button>
      <button
        className="bottom-nav-item"
        onClick={() => onGo("settings")}
        aria-label={user.display_name}
      >
        <span className="avatar avatar-xs" style={{ background: user.avatar_color }}>
          {user.display_name[0]}
        </span>
      </button>
    </nav>
  );
}
