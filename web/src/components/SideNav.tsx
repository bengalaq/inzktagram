import { User } from "../api";
import { useI18n } from "../i18n";
import { GearIcon, HomeIcon, PlusIcon, ShieldIcon } from "./Icons";
import LangToggle from "./LangToggle";

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
  const { t } = useI18n();
  return (
    <nav className="sidenav">
      <h1 className="logo" onClick={() => onGo("feed")}>
        inZKtagram
      </h1>
      <button
        className={`nav-item ${page === "feed" ? "active" : ""}`}
        onClick={() => onGo("feed")}
      >
        <HomeIcon /> <span>{t("navHome")}</span>
      </button>
      <button className="nav-item" onClick={onNewPost}>
        <PlusIcon /> <span>{t("navCreate")}</span>
      </button>
      <button className="nav-item" onClick={onVerify}>
        <span className="nav-icon-badge">
          <ShieldIcon />
          {proofBadge && (
            <span className={`badge-dot ${proofBadge === "ok" ? "ok" : "fail"}`} />
          )}
        </span>
        <span>{t("navVerify")}</span>
      </button>
      <button
        className={`nav-item ${page === "settings" ? "active" : ""}`}
        onClick={() => onGo("settings")}
      >
        <GearIcon /> <span>{t("navSettings")}</span>
      </button>

      <div className="sidenav-spacer" />

      <LangToggle className="lang-toggle-nav" />

      <button className="nav-item nav-profile" onClick={onSwitchUser} title={t("navSwitchUser")}>
        <span className="avatar avatar-sm" style={{ background: user.avatar_color }}>
          {user.display_name[0]}
        </span>
        <span>{user.display_name}</span>
      </button>
    </nav>
  );
}
