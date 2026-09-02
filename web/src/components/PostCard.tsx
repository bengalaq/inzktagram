import { useRef, useState } from "react";
import { api, Post } from "../api";
import { useI18n } from "../i18n";
import { BookmarkIcon, CommentIcon, HeartIcon, MoreIcon, SendIcon } from "./Icons";

const GRADIENTS = 6;

export default function PostCard({ post }: { post: Post }) {
  const { t, timeAgo, locale, localeTag } = useI18n();
  const [likes, setLikes] = useState(post.likes);
  const [liked, setLiked] = useState(false);
  const [saved, setSaved] = useState(false);
  const [burst, setBurst] = useState(false);
  const lastTap = useRef(0);

  const like = async () => {
    if (liked) return;
    setLiked(true);
    setLikes((n) => n + 1);
    try {
      const res = await api.likePost(post.id);
      setLikes(res.likes);
    } catch {
      // demo: el like es cosmética
    }
  };

  const doubleTap = () => {
    const now = Date.now();
    if (now - lastTap.current < 320) {
      if (!liked) {
        setBurst(true);
        window.setTimeout(() => setBurst(false), 700);
        void like();
      }
    }
    lastTap.current = now;
  };

  const text =
    locale === "en" && post.content_en ? post.content_en : post.content;
  const bait = text.length < 140 && post.likes >= 400;
  const longform = text.length >= 300;

  return (
    <article className={`post-card ${bait ? "post-card-bait" : ""} ${longform ? "post-card-long" : ""}`}>
      <header className="post-head">
        <span className="avatar avatar-sm" style={{ background: post.avatar_color }}>
          {post.display_name[0]}
        </span>
        <div className="post-head-names">
          <span className="post-user">
            {post.username}
            {post.is_followed && <span className="follow-chip">{t("followed")}</span>}
            {bait && !post.is_followed && <span className="bait-chip">{t("trending")}</span>}
          </span>
          <span className="post-time">{timeAgo(post.created_at)}</span>
        </div>
        <button className="icon-btn post-more" aria-label={t("moreOptions")}>
          <MoreIcon />
        </button>
      </header>

      <div
        className={`post-media g${post.id % GRADIENTS} ${bait ? "post-media-bait" : ""} ${longform ? "post-media-long" : ""}`}
        onClick={doubleTap}
      >
        <p>{text}</p>
        {burst && (
          <span className="like-burst" aria-hidden>
            <HeartIcon size={72} filled />
          </span>
        )}
      </div>

      <div className="post-actions">
        <button
          className={`icon-btn ${liked ? "liked" : ""}`}
          onClick={like}
          aria-label={t("like")}
        >
          <HeartIcon filled={liked} />
        </button>
        <button className="icon-btn" aria-label={t("comment")}>
          <CommentIcon />
        </button>
        <button className="icon-btn" aria-label={t("share")}>
          <SendIcon />
        </button>
        <span className="post-actions-spacer" />
        <button
          className={`icon-btn ${saved ? "saved" : ""}`}
          onClick={() => setSaved((v) => !v)}
          aria-label={t("save")}
        >
          <BookmarkIcon filled={saved} />
        </button>
      </div>

      <div className="post-meta">
        <strong>{t("likesCount", { n: likes.toLocaleString(localeTag) })}</strong>
        <span className="post-comments">
          {post.comments > 0
            ? t("viewComments", { n: post.comments })
            : t("firstComment")}
        </span>
        <span className="post-time-foot">{timeAgo(post.created_at)}</span>
      </div>
    </article>
  );
}
