export interface User {
  id: number;
  username: string;
  display_name: string;
  avatar_color: string;
}

export interface Post {
  id: number;
  content: string;
  created_at: number;
  likes: number;
  comments: number;
  username: string;
  display_name: string;
  avatar_color: string;
  is_followed: boolean;
}

export interface FeedResponse {
  view_id: number;
  algorithm_id: number;
  generated_at: number;
  proof_status: string;
  posts: Post[];
}

export interface Journal {
  algorithm_id: number;
  algorithm_name: string;
  config_hash: string;
  params_hash: string;
  candidates_hash: string;
  feed_hash: string;
  timestamp: number;
}

export interface ProofStatus {
  view_id: number;
  status: "pending" | "proving" | "proved" | "failed";
  algorithm_claimed: number;
  journal: Journal | null;
  proving_ms: number | null;
  user_cycles: number | null;
  error: string | null;
  image_id: string;
  dev_mode: boolean;
}

export interface VerifyChecks {
  proof_valid: boolean;
  algorithm_matches: boolean;
  feed_matches: boolean;
}

export interface VerifyResponse {
  proof_valid: boolean;
  verify_ms: number;
  image_id: string;
  journal?: Journal;
  error?: string;
  algorithm_claimed: number;
  algorithm_served: number;
  displayed_feed_hash: string;
  checks: VerifyChecks;
  dev_mode: boolean;
}

export interface StateResponse {
  user: User;
  algorithm_id: number;
  nonce: number;
  malicious: boolean;
  image_id: string;
}

async function req<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error ?? res.statusText);
  }
  return res.json();
}

export const api = {
  users: () => req<User[]>("/api/users"),
  state: (userId: number) => req<StateResponse>(`/api/state/${userId}`),
  setAlgorithm: (userId: number, algorithmId: number) =>
    req<{ algorithm_id: number; nonce: number }>(`/api/settings/${userId}`, {
      method: "PUT",
      body: JSON.stringify({ algorithm_id: algorithmId }),
    }),
  setMalicious: (enabled: boolean) =>
    req<{ malicious: boolean }>("/api/demo/malicious", {
      method: "PUT",
      body: JSON.stringify({ enabled }),
    }),
  feed: (userId: number) => req<FeedResponse>(`/api/feed/${userId}`),
  createPost: (authorId: number, content: string) =>
    req<{ id: number }>("/api/posts", {
      method: "POST",
      body: JSON.stringify({ author_id: authorId, content }),
    }),
  likePost: (postId: number) =>
    req<{ likes: number }>(`/api/posts/${postId}/like`, { method: "POST" }),
  proofStatus: (viewId: number) => req<ProofStatus>(`/api/proofs/${viewId}`),
  verifyProof: (viewId: number) =>
    req<VerifyResponse>(`/api/proofs/${viewId}/verify`, { method: "POST" }),
  receiptUrl: (viewId: number) => `/api/proofs/${viewId}/receipt`,
};

export const ALGORITHMS = [
  {
    id: 1,
    name: "Engagement",
    tagline: "Como las redes de siempre",
    description:
      "Maximiza tu tiempo en la plataforma: recencia agresiva, ganchos virales cortos y cuentas que no seguís inyectadas como «novedad». El feed se llena de likes, FOMO y hilos. Es el modelo de negocio de la atención.",
  },
  {
    id: 2,
    name: "Bienestar",
    tagline: "Protege tu atención",
    description:
      "Solo cuentas que seguís, en orden mayormente cronológico. Prioriza textos largos, entierra los ganchos cortos y no usa likes ni viralidad. El feed se siente como hablar con gente conocida, sin tragamonedas.",
  },
  {
    id: 3,
    name: "Mixto",
    tagline: "Un punto medio",
    description:
      "Combinación ponderada (60% bienestar, 40% engagement): descubrís cosas nuevas sin que el feed se convierta en una máquina tragamonedas.",
  },
] as const;

export function algorithmName(id: number): string {
  return ALGORITHMS.find((a) => a.id === id)?.name ?? `#${id}`;
}

export function timeAgo(epochSecs: number): string {
  const mins = Math.max(0, Math.floor(Date.now() / 1000 - epochSecs) / 60);
  if (mins < 1) return "ahora";
  if (mins < 60) return `hace ${Math.floor(mins)} min`;
  const hours = mins / 60;
  if (hours < 24) return `hace ${Math.floor(hours)} h`;
  return `hace ${Math.floor(hours / 24)} d`;
}
