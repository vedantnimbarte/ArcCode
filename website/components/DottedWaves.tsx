"use client";

import { useEffect, useRef } from "react";

type Props = {
  /** spacing between dots in px */
  gap?: number;
  /** base dot radius in px */
  dot?: number;
  /** how strongly dots brighten on the wave crest */
  intensity?: number;
  className?: string;
};

/**
 * Full-bleed animated field of dots whose brightness + size are modulated by
 * slow travelling sine waves. Sits behind content, ignores pointer events,
 * pauses when the tab is hidden, and renders a single static frame when the
 * user prefers reduced motion.
 */
export function DottedWaves({
  gap = 30,
  dot = 1.25,
  intensity = 1,
  className,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reduce = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;

    let width = 0;
    let height = 0;
    let dpr = 1;
    let raf = 0;
    let running = true;

    const resize = () => {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const draw = (t: number) => {
      ctx.clearRect(0, 0, width, height);
      const cols = Math.ceil(width / gap) + 1;
      const rows = Math.ceil(height / gap) + 1;
      const time = t * 0.00035;

      for (let y = 0; y < rows; y++) {
        for (let x = 0; x < cols; x++) {
          const px = x * gap;
          const py = y * gap;

          // two travelling waves crossing diagonally
          const w1 = Math.sin(px * 0.012 + py * 0.006 + time * 2.0);
          const w2 = Math.sin(px * 0.005 - py * 0.011 + time * 1.4);
          const wave = (w1 + w2) * 0.5; // -1..1

          const level = (wave + 1) * 0.5; // 0..1
          const radius = dot * (0.55 + level * 0.9);
          const alpha = (0.05 + level * 0.32) * intensity;

          // subtle vertical fade so the field melts into the page
          const fade = 1 - Math.min(py / Math.max(height, 1), 1) * 0.35;

          ctx.beginPath();
          ctx.arc(px, py + wave * 4, radius, 0, Math.PI * 2);
          ctx.fillStyle = `rgba(173, 181, 189, ${(alpha * fade).toFixed(3)})`;
          ctx.fill();
        }
      }

      if (running) raf = requestAnimationFrame(draw);
    };

    resize();

    if (reduce) {
      draw(0); // single static frame
    } else {
      raf = requestAnimationFrame(draw);
    }

    const onResize = () => resize();
    const onVisibility = () => {
      if (document.hidden) {
        running = false;
        cancelAnimationFrame(raf);
      } else if (!reduce) {
        running = true;
        raf = requestAnimationFrame(draw);
      }
    };

    window.addEventListener("resize", onResize);
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      running = false;
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", onResize);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [gap, dot, intensity]);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden="true"
      className={className}
      style={{ width: "100%", height: "100%", display: "block" }}
    />
  );
}
