/**
 * Recomputa, EN EL CLIENTE, el hash del feed tal como lo define
 * `feed_core::hash_feed`: SHA-256("inzktagram.feed.v1" || post_id_1 (u64 LE)
 * || post_id_2 || ...). Comparar este hash contra el `feed_hash` del journal
 * garantiza que el feed que el navegador renderizó es exactamente el que la
 * prueba ZK certifica, sin confiar en el servidor.
 */
export async function computeFeedHash(postIds: number[]): Promise<string> {
  const prefix = new TextEncoder().encode("inzktagram.feed.v1");
  const buf = new Uint8Array(prefix.length + postIds.length * 8);
  buf.set(prefix, 0);
  const view = new DataView(buf.buffer);
  postIds.forEach((id, i) => {
    view.setBigUint64(prefix.length + i * 8, BigInt(id), true);
  });
  const digest = await crypto.subtle.digest("SHA-256", buf);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
