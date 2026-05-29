"use client";

import { useEffect, useRef } from "react";

/**
 * Animated dotted-wave canvas background. A grid of dots whose vertical
 * offset and opacity ripple via stacked sine waves. Respects reduced motion.
 */
export function DottedWaves() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let raf = 0;
    let width = 0;
    let height = 0;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);

    const PALETTE = ["#adb5bd", "#6c757d", "#495057"];

    function resize() {
      if (!canvas) return;
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx?.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    const waveColors = PALETTE;

    function draw(t: number) {
      if (!ctx) return;
      ctx.clearRect(0, 0, width, height);
      const gap = 34;
      const cols = Math.ceil(width / gap) + 1;
      const rows = Math.ceil(height / gap) + 1;
      const amp = 7;
      for (let i = 0; i < cols; i++) {
        for (let j = 0; j < rows; j++) {
          const x = i * gap;
          const baseY = j * gap;
          const phase = (i + j) * 0.35;
          const y = baseY + Math.sin(t * 0.0013 + phase) * amp;
          const op =
            0.25 + 0.6 * (0.5 + 0.5 * Math.sin(t * 0.0013 + phase));
          const ci = (i + j) % waveColors.length;
          ctx.globalAlpha = op * 0.5;
          ctx.fillStyle = waveColors[ci];
          ctx.beginPath();
          ctx.arc(x, y, 1.6, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      ctx.globalAlpha = 1;
      raf = requestAnimationFrame(draw);
    }

    const reduce = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;

    resize();
    window.addEventListener("resize", resize);
    if (reduce) {
      // draw a single static frame
      draw(0);
      cancelAnimationFrame(raf);
    } else {
      raf = requestAnimationFrame(draw);
    }

    return () => {
      window.removeEventListener("resize", resize);
      cancelAnimationFrame(raf);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      aria-hidden
      className="pointer-events-none fixed inset-0 z-[1] h-full w-full opacity-70"
    />
  );
}
