import { useState } from "react";
import { useI18n } from "../i18n";
import { XIcon } from "./Icons";

interface Props {
  open: boolean;
  onClose: () => void;
  onPublish: (content: string) => Promise<void>;
}

export default function NewPostModal({ open, onClose, onPublish }: Props) {
  const { t } = useI18n();
  const [content, setContent] = useState("");
  const [busy, setBusy] = useState(false);
  if (!open) return null;

  const publish = async () => {
    if (!content.trim() || busy) return;
    setBusy(true);
    try {
      await onPublish(content.trim());
      setContent("");
      onClose();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal modal-narrow" onClick={(e) => e.stopPropagation()}>
        <header className="modal-head">
          <h2>{t("newPost")}</h2>
          <button className="icon-btn" onClick={onClose} aria-label={t("close")}>
            <XIcon />
          </button>
        </header>
        <textarea
          className="post-input"
          rows={7}
          maxLength={2000}
          placeholder={t("postPlaceholder")}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          autoFocus
        />
        <div className="post-input-foot">
          <span className="muted">{content.length}/2000</span>
          <button className="btn" onClick={publish} disabled={!content.trim() || busy}>
            {busy ? t("publishing") : t("publish")}
          </button>
        </div>
      </div>
    </div>
  );
}
