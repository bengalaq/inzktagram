interface IconProps {
  size?: number;
  filled?: boolean;
}

const base = (size: number) => ({
  width: size,
  height: size,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.8,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
});

export const HomeIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M3 10.5 12 3l9 7.5" />
    <path d="M5 9.5V21h5v-6h4v6h5V9.5" />
  </svg>
);

export const PlusIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <rect x="3" y="3" width="18" height="18" rx="4" />
    <path d="M12 8v8M8 12h8" />
  </svg>
);

export const ShieldIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M12 3l7 3v5c0 5-3.5 8.5-7 10-3.5-1.5-7-5-7-10V6l7-3z" />
    <path d="M9 12l2 2 4-4" />
  </svg>
);

export const GearIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09a1.7 1.7 0 0 0 1.55-1 1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.87.34h.01a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55h.01a1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.87v.01a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.55 1z" />
  </svg>
);

export const HeartIcon = ({ size = 24, filled = false }: IconProps) => (
  <svg {...base(size)} fill={filled ? "currentColor" : "none"}>
    <path d="M12 21s-7.5-4.7-9.5-9.2C1 8 3 4.5 6.6 4.5c2.2 0 3.8 1.2 5.4 3.3 1.6-2.1 3.2-3.3 5.4-3.3C21 4.5 23 8 21.5 11.8 19.5 16.3 12 21 12 21z" />
  </svg>
);

export const CommentIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M21 12a8.5 8.5 0 0 1-8.5 8.5c-1.5 0-3-.4-4.2-1L3 21l1.5-5.3a8.5 8.5 0 1 1 16.5-3.7z" />
  </svg>
);

export const SendIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M21 3 10 14" />
    <path d="M21 3l-7 18-4-7-7-4 18-7z" />
  </svg>
);

export const BookmarkIcon = ({ size = 24, filled = false }: IconProps) => (
  <svg {...base(size)} fill={filled ? "currentColor" : "none"}>
    <path d="M6 3h12v18l-6-4.5L6 21V3z" />
  </svg>
);

export const RefreshIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M21 12a9 9 0 1 1-2.6-6.3" />
    <path d="M21 3v6h-6" />
  </svg>
);

export const CheckIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M4 12.5l5 5L20 6.5" />
  </svg>
);

export const XIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M5 5l14 14M19 5L5 19" />
  </svg>
);

export const AlertIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M12 3 1.8 20.2h20.4L12 3z" />
    <path d="M12 10v4.5M12 17.8v.2" />
  </svg>
);

export const DownloadIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M12 3v12M7 10l5 5 5-5" />
    <path d="M4 19h16" />
  </svg>
);

export const LeafIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M5 19C5 9 12 4 20 4c0 8-5 15-15 15z" />
    <path d="M5 19c3-5 7-8 11-10" />
  </svg>
);

export const MoreIcon = ({ size = 24 }: IconProps) => (
  <svg {...base(size)}>
    <circle cx="5" cy="12" r="1.2" fill="currentColor" stroke="none" />
    <circle cx="12" cy="12" r="1.2" fill="currentColor" stroke="none" />
    <circle cx="19" cy="12" r="1.2" fill="currentColor" stroke="none" />
  </svg>
);
