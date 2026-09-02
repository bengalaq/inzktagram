# inZKtagram

A social feed whose ranking is **user-chosen** and **cryptographically checked**. The user picks one of three algorithms; the server produces a RISC Zero receipt proving that the feed was computed with that choice. Anyone can verify the receipt without trusting the platform.

1. **Engagement** — retention: recency, virality, short bait, novelty slots from accounts you do not follow.
2. **Wellbeing** — attention: followed accounts only, long-form, mostly chronological, no like-based ranking.
3. **Mixed** — 60/40 blend of the two, with an author-repetition cap.

This is the final project for *Building Cryptographic Proofs: ZKPs & SNARKs*, ECI 2026 (UBA). Base paper: **ZKML** (Kang et al., EuroSys 2024). ZKML proves ML inference against a committed model (Halo2 SNARK). We transplant the idea to **verifiable recommendation with user choice**, proving “this feed came from algorithm *A*” instead of “this output came from model *M*”, using a zkVM rather than a hand-written circuit.

---

## 1. Application

**Problem.** Ranking algorithms are opaque. Platforms *declare* a policy; nobody can check it. DSA-style “choose a non-profiled feed” is still a promise. A proof turns the promise into a checkable statement.

**Who proves what to whom.** The **server** (prover) proves to the **user or an auditor** (verifier):

> Feed `F` (hash `feed_hash`) is `rank(algorithm_id, params, candidates, now)`, `algorithm_id` matches the user’s committed config (`config_hash`), and `params` / `candidates` match `params_hash` / `candidates_hash`.

- **Public statement (journal):** `algorithm_id`, `config_hash`, `params_hash`, `candidates_hash`, `feed_hash`, `timestamp`, plus the guest **image ID**.
- **Witness (private):** full candidate vectors (likes, follows, timestamps, lengths) and the config nonce. They never appear in the journal.
- **Trust.** The STARK proves **integrity of that execution**, not honesty of the input set. A server can drop posts *before* ranking and still get a valid receipt. Mitigations in scope: (i) `candidates_hash` is public, and `verifier-cli --candidates dump.json` recomputes it from a public dump (`GET /api/audit/{user_id}`); (ii) the guest **rejects** unsorted IDs, duplicates, and the viewer’s own posts (those rules are in the image ID); (iii) the browser recomputes `feed_hash` over the posts it rendered. A real transparency log of posts is **out of scope**. The UI’s STARK tick is a **courtesy** call to the server; the trustless path is the CLI (the browser does *not* verify the STARK itself).

## 2. Proof system

**RISC Zero 3.0.6 zkVM**, transparent STARKs, no trusted setup. The guest is ordinary Rust. Ranking lives in one crate (`feed-core`) compiled on the **host and in the guest**, so the UI and the proof run the same function (integer arithmetic, total `post_id` tie-break, `now` as an explicit input).

**Why a STARK / zkVM, not a Halo2 SNARK.** The ranking is control-flow heavy (`if`, sort, author cap, novelty injection). Circuitizing it would destroy the “one function” guarantee and blow the one-month budget. RISC Zero lets us prove the real Rust. Cost: receipts of hundreds of KB (fine for a human verifier, not for L1). A Groth16 wrap for on-chain verification is a possible extension; it does not help the user-choice thesis.

This is an **argument of correct execution**. The journal is public. We do **not** claim cryptographic zero-knowledge of the execution trace beyond “the witness is not in the journal.”

## 3. Architecture

```
web/           React. Feed, settings, Verify. Client-side feed_hash (WebCrypto).
feed-core/     Three rankers + canonical hashes + candidacy predicate + journal.
methods/       Guest: check candidacy, rank, commit journal.
server/        Axum + SQLite + async prover worker. `zkbench` binary.
verifier-cli/  Independent STARK + claim checker; optional candidates audit.
```

Flow: open feed → host `feed-core` returns instantly → worker proves the same `FeedInput` → **Verify** checks (courtesy STARK) ∧ `algorithm_id` ∧ client `feed_hash`.

**Soundness demo:** Settings → “malicious server” serves Engagement while claiming the user’s choice. The receipt is valid for the *claimed* algorithm; `feed_hash` does not match what is on screen, so verification fails.

## 4. Running

Docker Desktop. From `inzktagram/`:

```powershell
.\run.cmd          # http://localhost:8080  (real STARKs, RISC0_DEV_MODE=0)
```

```bash
./run.sh
```

First boot compiles the toolchain. The first receipt takes minutes on CPU; later views similar. UI-only iteration: `RISC0_DEV_MODE=1 docker compose up --build`.

Trustless check (receipt in `download_receipts/`):

```powershell
.\verify.cmd inzktagram_view_15.receipt --expect-algorithm 2 --expect-feed-hash <hex>
.\verify.cmd inzktagram_view_15.receipt --candidates dump.json
```

`dump.json` is `GET /api/audit/{user_id}` (convenience; not a transparency log).

## 5. Tests (completeness and soundness)

```bash
cargo test -p feed-core
RISC0_DEV_MODE=1 cargo test -p inzktagram-server --test completeness_dev
RISC0_DEV_MODE=1 cargo test -p inzktagram-server --test application_checks
cargo test --release -p inzktagram-server --test real_proofs -- --ignored
```

| Test | What it shows |
|------|----------------|
| `feed-core` | Determinism, algorithm properties, candidacy, `claims_match`. |
| `completeness_dev` | Guest journal = native `rank` (CI, every push). |
| `application_checks` | Guest aborts on illegal candidates; wrong algorithm / `feed_hash` claims fail (CI, every push). |
| `real_proofs` | Real STARK accepts an honest run; tampered journal and wrong image ID are rejected (`workflow_dispatch` / weekly CI). Log: `benchmarks/real_proofs.log` (49 s on this host). |

## 6. Benchmarks

```bash
./scripts/bench.sh          # Linux / macOS / WSL
.\scripts\bench.cmd         # Windows
```

Must run with `RISC0_DEV_MODE=0`. Output: `benchmarks/results.csv`. Hardware: **AMD Ryzen 7 2700X (8c/16t), Linux in Docker Desktop on Windows**.

| N | Algorithm | Cycles | Proving (ms) | Verify (ms) | Receipt (KB) | Native (µs) |
|---|-----------|--------|--------------|-------------|--------------|-------------|
| 25 | Engagement | 205 114 | 47 510 | 19.5 | 251 | 19 |
| 25 | Wellbeing | 172 018 | 58 193 | 21.0 | 251 | 6 |
| 25 | Mixed | 209 053 | 44 878 | 19.4 | 251 | 10 |
| 50 | Engagement | 332 249 | 85 473 | 20.6 | 263 | 14 |
| 50 | Wellbeing | 313 917 | 94 257 | 34.8 | 263 | 8 |
| 50 | Mixed | 392 111 | 96 736 | 20.8 | 263 | 16 |
| 100 | Engagement | 553 116 | — | — | — | 28 |
| 100 | Wellbeing | 584 718 | — | — | — | 15 |
| 100 | Mixed | 805 402 | — | — | — | 28 |
| 200 | Engagement | 1 012 518 | — | — | — | 21 |
| 200 | Wellbeing | 1 128 749 | — | — | — | 14 |
| 200 | Mixed | 1 790 794 | — | — | — | 48 |

N=25 and N=50 are real STARKs. At N≥100 the local `r0vm` process died mid-prove (`rx len failed` / OOM) after ~9 min; cycles and native times for those rows are guest execution only. Overhead at N=50 is about **5 000×–12 000×** vs native `rank()` (85–97 s vs 8–16 µs). Verification is ~20 ms and receipt size is ~250–263 KB — the STARK cost is almost independent of the algorithm, as expected for one guest image.

**ZKML baseline (qualitative).** ZKML (Halo2/KZG) proves a DLRM recommendation model in **34.4 s** and MNIST in **2.5 s**, with **~7–12 ms** verification and **~12–15 kB** SNARK proofs (paper Tables 6, 9, 10). Our guest is a tiny ranker, not a neural net: proving time is in the same *human* range (seconds to minutes on CPU), verification stays milliseconds, receipts are **larger** (STARK vs Groth16/KZG). We pay proof size and prover time to keep the ranking in real Rust and to avoid a trusted setup. That is the framework trade-off this project measures.

## 7. Scope

In: text posts, likes, algorithm choice, verification. Out: real auth, media, moderation, on-chain wrap, ML ranking, a transparency log of posts.

---

**Who did what.** Juan Pose — design, implementation, evaluation. Course: ECI 2026, *Building Cryptographic Proofs* (Marco Zecchini). Slides: [`docs/slides.html`](docs/slides.html).
