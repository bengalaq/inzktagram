import { User } from "../api";
import { useI18n } from "../i18n";
import { LeafIcon } from "./Icons";
import LangToggle from "./LangToggle";

interface Props {
  users: User[];
  onPick: (user: User) => void;
}

export default function UserPicker({ users, onPick }: Props) {
  const { t } = useI18n();
  return (
    <div className="picker-screen">
      <LangToggle className="lang-toggle-float" />
      <div className="picker-card">
        <h1 className="logo logo-big">inZKtagram</h1>
        <p className="picker-sub">
          <LeafIcon size={16} /> {t("pickerSub")}
        </p>
        <p className="picker-hint">{t("pickerHint")}</p>
        <div className="picker-grid">
          {users.map((u) => (
            <button key={u.id} className="picker-user" onClick={() => onPick(u)}>
              <span className="avatar avatar-lg" style={{ background: u.avatar_color }}>
                {u.display_name[0]}
              </span>
              <span className="picker-name">{u.display_name}</span>
              <span className="picker-username">@{u.username}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
