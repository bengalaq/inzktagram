import { User } from "../api";
import { GearIcon, HomeIcon, PlusIcon, ShieldIcon } from "./Icons";

export type Page = "feed" | "settings";

interface Props {
  user: User;
  page: Page;
  proofBadge: "ok" | "fail" | null;
  onGo: (page: Page) => void;
  onNewPost: () => void;
  onVerify: () => void;
  onSwitchUser: () => void;
}

export default function SideNav({
  user,
  page,
  proofBadge,
  onGo,
  onNewPost,
  onVerify,
  onSwitchUser,
}: Props) {
  return (
    <nav className="sidenav">
      <h1 className="logo" onClick={() => onGo("feed")}>
        inZKtagram
      </h1>
      <button
        className={`nav-item ${page === "feed" ? "active" : ""}`}
        onClick={() => onGo("feed")}
      >
        <HomeIcon /> <span>Inicio</span>
      </button>
      <button className="nav-item" onClick={onNewPost}>
        <PlusIcon /> <span>Crear</span>
      </button>
      <button className="nav-item" onClick={onVerify}>
        <span className="nav-icon-badge">
          <ShieldIcon />
          {proofBadge && (
            <span className={`badge-dot ${proofBadge === "ok" ? "ok" : "fail"}`} />
          )}
        </span>
        <span>Verificar</span>
      </button>
      <button
        className={`nav-item ${page === "settings" ? "active" : ""}`}
        onClick={() => onGo("settings")}
      >
        <GearIcon /> <span>Ajustes</span>
      </button>

      <div className="sidenav-spacer" />

      <button className="nav-item nav-profile" onClick={onSwitchUser} title="Cambiar de perfil">
        <span className="avatar avatar-sm" style={{ background: user.avatar_color }}>
          {user.display_name[0]}
        </span>
        <span>{user.display_name}</span>
      </button>
    </nav>
  );
}
