import { ALGORITHMS } from "../api";
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
  return (
    <div className="settings">
      <h2 className="page-title">Tu algoritmo de recomendación</h2>
      <p className="muted settings-intro">
        Acá mandás vos. Elegí cómo se ordena tu feed; cada vez que lo abras, la
        plataforma generará una prueba de conocimiento cero (RISC Zero) de que
        usó exactamente el algoritmo que elegiste.
      </p>

      <div className="alg-cards">
        {ALGORITHMS.map((alg) => {
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
          <ShieldIcon size={18} /> Verificabilidad
        </h3>
        <p className="muted">
          El programa que rankea tu feed dentro de la zkVM está identificado
          públicamente por su image ID. Cualquiera puede recompilarlo desde el
          código fuente y comprobar que coincide:
        </p>
        <code className="hash hash-block" title={imageId}>
          {imageId}
        </code>
      </section>

      <section className="settings-demo">
        <h3>
          <AlertIcon size={18} /> Demo: servidor malicioso
        </h3>
        <p className="muted">
          Activá esto para simular una plataforma deshonesta: servirá el feed
          con el algoritmo <strong>Engagement</strong> aunque afirme usar el que
          elegiste. Después tocá «Verificar» en tu feed y mirá cómo la prueba ZK
          detecta el engaño.
        </p>
        <label className="toggle">
          <input
            type="checkbox"
            checked={malicious}
            onChange={(e) => onToggleMalicious(e.target.checked)}
          />
          <span className="toggle-track" />
          <span>{malicious ? "Servidor malicioso ACTIVO" : "Servidor honesto"}</span>
        </label>
      </section>
    </div>
  );
}
