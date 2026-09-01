import { User } from "../api";
import { LeafIcon } from "./Icons";

interface Props {
  users: User[];
  onPick: (user: User) => void;
}

export default function UserPicker({ users, onPick }: Props) {
  return (
    <div className="picker-screen">
      <div className="picker-card">
        <h1 className="logo logo-big">inZKtagram</h1>
        <p className="picker-sub">
          <LeafIcon size={16} /> Una red social donde el algoritmo lo elegís
          vos, y podés probarlo.
        </p>
        <p className="picker-hint">
          Elegí un perfil. Las cuentas en verde publican despacio; las de
          rojo, el loop de atención. El feed cambia según el algoritmo.
        </p>
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
