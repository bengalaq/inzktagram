# inZKtagram

Red social con **algoritmo de recomendación verificable** mediante pruebas de
conocimiento cero (RISC Zero zkVM).

El usuario elige en sus ajustes cuál de 3 algoritmos ordena su feed:

1. **Engagement** — maximiza retención: ganchos virales cortos, miles de
   likes y cuentas que no seguís (el loop de dopamina de las redes actuales).
2. **Bienestar** — protege la atención: solo cuentas seguidas, textos largos
   en orden mayormente cronológico, sin likes ni carnada.
3. **Mixto** — combinación ponderada (60/40) de los dos anteriores.

La plataforma acompaña cada feed con un **receipt de RISC Zero** que prueba
que el feed mostrado fue calculado con el algoritmo elegido. El usuario puede
verificarlo en la app o, sin confiar en el servidor, con un verificador de
línea de comandos independiente.

---

## 1. La aplicación

**Problema.** Los algoritmos de recomendación son opacos: la plataforma
*declara* qué hace, nadie puede comprobarlo. Este proyecto convierte esa
declaración en una garantía criptográfica: el usuario elige el algoritmo y
puede verificar que fue respetado.

**Qué se prueba, quién y a quién.** El **servidor** (prover) le prueba al
**usuario o a un auditor** (verifier):

> *"El feed `F` (hash `feed_hash`) es el resultado de ejecutar el algoritmo
> `algorithm_id` con parámetros comprometidos (`params_hash`) sobre el
> conjunto de candidatos comprometido (`candidates_hash`), donde
> `algorithm_id` coincide con la configuración del usuario (`config_hash`)."*

- **Statement (público):** el journal del receipt: `algorithm_id`,
  `config_hash`, `params_hash`, `candidates_hash`, `feed_hash`, `timestamp` —
  más el **image ID** del guest (hash del programa, reproducible compilando el
  código fuente).
- **Witness (privado):** el conjunto completo de candidatos con sus señales
  (likes, timestamps, follows) y la configuración del usuario.
- **Supuestos de confianza:** la prueba garantiza *integridad de cómputo*, no
  honestidad de inputs. El servidor podría manipular el conjunto de candidatos
  antes de rankear; el journal publica `candidates_hash` como mitigación
  parcial (un auditor con acceso a los posts públicos puede recomputarlo). Un
  transparency log de posts que cierre esa brecha queda como extensión futura.
  El cliente web sí cierra la brecha de "mostrar otra cosa": recomputa el hash
  del feed renderizado (WebCrypto) y lo compara con el `feed_hash` probado.

## 2. Sistema de pruebas

- **RISC Zero zkVM** (crates `risc0-zkvm` / `risc0-build` v3.0.6): zk-STARK
  transparente (sin trusted setup), prover local en CPU/GPU, verificación en
  milisegundos, receipts de cientos de KB. El guest es Rust normal (RISC-V).
- El ranking vive en un único crate (`feed-core`) compilado **tanto en el
  servidor como en el guest**: lo que se muestra y lo que se prueba es
  literalmente la misma función. Determinismo garantizado por aritmética
  entera, desempates totales por `post_id` y reloj como input explícito.

**Por qué STARK y no SNARK.** No es que el feed *exija* STARKs: elegimos una
zkVM para probar Rust real (el mismo `feed-core`) en un mes, y RISC Zero
implementa eso con STARKs. ZKML usa Halo2 (SNARK en circuito); un ranking con
`if`, `sort` y tope de autor es incómodo de circuitizar y rompería la
garantía de “una sola función”. El STARK trae setup transparente (el usuario
verifica contra el image ID, sin ceremonia de confianza) y receipts de
cientos de KB, aceptables porque el verificador es una persona, no un
contrato. El wrap a Groth16 (SNARK chico para on-chain) queda como extensión:
no aporta a la tesis de elección verificable.

## 3. Arquitectura

```
web/           React + Vite. Estética estilo Instagram, paleta calma.
               Verifica del lado del cliente: hash del feed renderizado.
feed-core/     Los 3 algoritmos de ranking + hashes canónicos + journal.
methods/       Guest RISC Zero: rank(input) → commit(journal).
server/        Axum + SQLite. API REST + worker asíncrono de pruebas.
  bin/zkbench  Benchmarks reproducibles (CSV).
verifier-cli/  Verificador independiente de receipts.
```

Flujo: el usuario abre su feed → el servidor lo computa al instante con
`feed-core` y encola el job → el worker ejecuta el mismo input en la zkVM y
guarda el receipt → el usuario toca **Verificar**: prueba STARK válida ∧
image ID correcto ∧ `algorithm_id` = su elección ∧ `feed_hash` = hash del feed
que su navegador renderizó.

**Demo de soundness:** en Ajustes se puede activar el "servidor malicioso",
que sirve Engagement mientras afirma usar el algoritmo elegido. La
verificación falla visiblemente (el `feed_hash` probado no coincide con lo
mostrado). Todas las estrategias de engaño fallan en algún chequeo: la zkVM no
puede ser forzada a producir un receipt válido de un cómputo que no ocurrió.

## 4. Cómo correrlo

**Un comando** (Docker: mismo flujo en Windows, macOS y Linux). Requisito:
[Docker Desktop](https://www.docker.com/products/docker-desktop/) con el
motor en marcha.

```bash
./run.sh          # Linux / macOS / WSL
```

```powershell
.\run.cmd         # Windows (cmd o PowerShell)
```

Abrir `http://localhost:8080`. La primera vez compila RISC Zero y puede
tardar varios minutos; después Docker reusa la imagen. Ctrl+C detiene el
contenedor.

Por defecto genera **pruebas STARK reales** (`RISC0_DEV_MODE=0`). El primer
receipt tarda varios minutos en CPU; los siguientes también, uno por cada
vista de feed. Para iterar la UI sin esperar al prover:

```bash
RISC0_DEV_MODE=1 docker compose up --build
```

Verificar un receipt descargado, sin instalar Rust en el host. Guardalo en
`download_receipts/` (dentro de este proyecto) y, **desde `inzktagram/`**:

```powershell
.\verify.cmd inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hash>
```

```bash
./verify.sh inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hash>
```

Equivalente a mano (el contenedor ya monta `download_receipts` en `/receipts`):

```powershell
docker compose exec inzktagram verifier-cli /receipts/inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hash>
```

El image ID del guest va embebido en la imagen: si el receipt lo generó
otro binario, la verificación falla.

### Alternativa sin Docker (WSL Ubuntu 24.04)

RISC Zero 3.0.6 necesita glibc ≥ 2.34. `.\scripts\dev.ps1 run` o
`bash scripts/wsl-dev.sh run`. Frontend en hot reload: `cd web && npm run dev`.

## 5. Tests (completeness y soundness)

```bash
cargo test -p feed-core                       # unitarios de los 3 algoritmos
RISC0_DEV_MODE=1 cargo test -p inzktagram-server --test completeness_dev
cargo test --release -p inzktagram-server --test real_proofs -- --ignored  # STARK reales
```

- **Completeness:** un cómputo honesto produce un receipt que el verificador
  acepta, y el journal coincide con la ejecución nativa.
- **Soundness:** un journal adulterado (p. ej. afirmar otro algoritmo) y un
  image ID incorrecto son rechazados.

CI: `.github/workflows/ci.yml` corre los unitarios y la completeness en cada
push; las pruebas reales se disparan manualmente (`workflow_dispatch`).

## 6. Benchmarks

```bash
cargo run --release -p inzktagram-server --bin zkbench   # sin RISC0_DEV_MODE
```

Genera `benchmarks/results.csv` con: ciclos de usuario en la zkVM, tiempo de
proving, tiempo de verificación, tamaño del receipt y el baseline nativo (el
mismo ranking fuera de la zkVM), para N ∈ {25, 50, 100, 200} candidatos × 3
algoritmos. Completar la tabla final con los números del hardware de entrega.

| N   | Algoritmo | Ciclos | Proving (ms) | Verif. (ms) | Receipt (KB) | Nativo (µs) |
|-----|-----------|--------|--------------|-------------|--------------|-------------|
| _(completar con `benchmarks/results.csv` y especificar hardware)_ |||||||

## 7. Alcance

Solo posteo de texto, likes y elección/verificación de algoritmo. Sin auth
real, sin moderación, sin media. Fuera de alcance (declarado): transparency
log de posts, verificación on-chain (wrap Groth16), ranking con ML.

## 8. Créditos

Proyecto final — curso *Building Cryptographic Proofs: ZKPs & SNARKs*,
ECI 2026 (UBA). Paper base: **ZKML: An Optimizing System for ML Inference in
Zero-Knowledge Proofs** (EuroSys 2024) — trasladamos la idea de inferencia
verificable al dominio de los sistemas de recomendación con elección del
usuario, usando una zkVM (RISC Zero) en lugar de circuitos Halo2.

**Quién hizo qué:** _(completar por el grupo)_.
