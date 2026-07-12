// HeroCanvas — the landing-page hero's particle-network animation: wandering green
// scouts mesh-linked when close, with a red pathogen that periodically spawns, gets
// surrounded by lavender soldiers, and is killed. Pure canvas 2D, no dependencies.
// Ported from Panacea Landing.dc.html's startSim().
import { useEffect, useRef } from "react";

const SCOUT_COUNT = 46;
const LINK_DIST = 120;

export default function HeroCanvas() {
  const canvasRef = useRef(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    const parent = canvas.parentElement;
    let width, height, dpr;

    const resize = () => {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = parent.clientWidth;
      height = parent.clientHeight;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      canvas.style.width = width + "px";
      canvas.style.height = height + "px";
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    resize();
    window.addEventListener("resize", resize);

    const scouts = Array.from({ length: SCOUT_COUNT }, () => ({
      x: Math.random() * 1600,
      y: Math.random() * 700,
      a: Math.random() * Math.PI * 2,
      s: 0.25 + Math.random() * 0.35,
      r: 1.4 + Math.random() * 1.2,
    }));
    let pathogen = null;
    let soldiers = [];
    let nextSpawn = 90;
    let raf;

    const loop = () => {
      ctx.clearRect(0, 0, width, height);

      ctx.lineWidth = 1;
      for (let i = 0; i < SCOUT_COUNT; i++) {
        const p = scouts[i];
        for (let j = i + 1; j < SCOUT_COUNT; j++) {
          const q = scouts[j];
          const dx = p.x - q.x, dy = p.y - q.y, d2 = dx * dx + dy * dy;
          if (d2 < LINK_DIST * LINK_DIST) {
            const o = 0.1 * (1 - Math.sqrt(d2) / LINK_DIST);
            ctx.strokeStyle = `rgba(197,179,230,${o.toFixed(3)})`;
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.stroke();
          }
        }
      }

      for (const p of scouts) {
        p.a += (Math.random() - 0.5) * 0.15;
        p.x += Math.cos(p.a) * p.s;
        p.y += Math.sin(p.a) * p.s;
        if (p.x < -10) p.x = width + 10;
        if (p.x > width + 10) p.x = -10;
        if (p.y < -10) p.y = height + 10;
        if (p.y > height + 10) p.y = -10;
        ctx.fillStyle = "rgba(120,190,67,0.75)";
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fill();
      }

      if (!pathogen) {
        if (--nextSpawn <= 0) {
          pathogen = {
            x: width * (0.2 + Math.random() * 0.6),
            y: height * (0.2 + Math.random() * 0.6),
            r: 0,
            life: 0,
            dying: 0,
          };
          soldiers = [];
          for (let k = 0; k < 4; k++) {
            const ang = Math.random() * Math.PI * 2, dist = 180 + Math.random() * 120;
            soldiers.push({ x: pathogen.x + Math.cos(ang) * dist, y: pathogen.y + Math.sin(ang) * dist, arrived: false });
          }
        }
      } else {
        pathogen.life++;
        if (!pathogen.dying) pathogen.r = Math.min(9, pathogen.r + 0.15);
        const pulse = (pathogen.life % 70) / 70;
        ctx.strokeStyle = `rgba(224,90,90,${(0.5 * (1 - pulse)).toFixed(3)})`;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.arc(pathogen.x, pathogen.y, pathogen.r + pulse * 34, 0, Math.PI * 2);
        ctx.stroke();
        ctx.fillStyle = pathogen.dying
          ? `rgba(224,90,90,${Math.max(0, 1 - pathogen.dying / 40).toFixed(3)})`
          : "rgba(224,90,90,0.9)";
        ctx.beginPath();
        ctx.arc(pathogen.x, pathogen.y, Math.max(0, pathogen.r * (pathogen.dying ? 1 - pathogen.dying / 40 : 1)), 0, Math.PI * 2);
        ctx.fill();

        let allArrived = true;
        if (pathogen.life > 80) {
          for (const s of soldiers) {
            const dx = pathogen.x - s.x, dy = pathogen.y - s.y;
            const d = Math.hypot(dx, dy);
            if (d > 16) {
              s.x += (dx / d) * 1.6;
              s.y += (dy / d) * 1.6;
              allArrived = false;
            } else {
              s.arrived = true;
            }
            const fade = pathogen.dying ? Math.max(0, 1 - pathogen.dying / 40) : 1;
            ctx.fillStyle = `rgba(197,179,230,${(0.9 * fade).toFixed(3)})`;
            ctx.beginPath();
            ctx.arc(s.x, s.y, 3.2, 0, Math.PI * 2);
            ctx.fill();
            if (d < 60) {
              ctx.strokeStyle = `rgba(197,179,230,${(0.35 * fade).toFixed(3)})`;
              ctx.lineWidth = 1;
              ctx.beginPath();
              ctx.moveTo(s.x, s.y);
              ctx.lineTo(pathogen.x, pathogen.y);
              ctx.stroke();
            }
          }
        } else {
          allArrived = false;
        }

        if (allArrived && soldiers.every((s) => s.arrived) && !pathogen.dying) pathogen.dying = 1;
        if (pathogen.dying) {
          pathogen.dying++;
          if (pathogen.dying > 45) {
            pathogen = null;
            nextSpawn = 200 + Math.random() * 260;
          }
        }
      }

      raf = requestAnimationFrame(loop);
    };
    loop();

    return () => {
      window.removeEventListener("resize", resize);
      cancelAnimationFrame(raf);
    };
  }, []);

  return <canvas ref={canvasRef} style={{ position: "absolute", inset: 0, width: "100%", height: "100%" }} />;
}
