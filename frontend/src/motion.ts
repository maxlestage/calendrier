// Makes the "inox" (stainless-steel) gradient react to the phone's motion:
// tilting the device slides the metallic reflection, via CSS custom properties
// read by the `.steel` rule (--steel-angle / --steel-x / --steel-y).

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

let started = false;

/** Begin driving the steel reflection from device orientation. Safe to call
 *  many times (no-op after the first). On iOS the motion permission is asked
 *  once, on the first tap (required to be inside a user gesture). */
export function initSteelMotion(): void {
  if (started) return;
  started = true;
  if (typeof window === "undefined" || !("DeviceOrientationEvent" in window)) return;

  const root = document.documentElement;
  let raf = 0;
  let lastX = 50;
  let lastY = 50;

  const onOrient = (e: DeviceOrientationEvent) => {
    const gamma = e.gamma ?? 0; // left/right tilt  (-90…90)
    const beta = e.beta ?? 0; //  front/back tilt (-180…180)
    const x = clamp(50 + gamma * 0.8, 0, 100);
    const y = clamp(50 + (beta - 45) * 0.6, 0, 100);
    // A phone held still still emits jitter; writing a custom property on
    // :root restyles the whole document, so ignore imperceptible moves.
    if (Math.abs(x - lastX) < 0.5 && Math.abs(y - lastY) < 0.5) return;
    if (raf) return; // at most one update per frame
    raf = requestAnimationFrame(() => {
      raf = 0;
      lastX = x;
      lastY = y;
      root.style.setProperty("--steel-x", `${x}%`);
      root.style.setProperty("--steel-y", `${y}%`);
    });
  };

  const attach = () =>
    window.addEventListener("deviceorientation", onOrient, { passive: true });

  const DOE = window.DeviceOrientationEvent as unknown as {
    requestPermission?: () => Promise<"granted" | "denied">;
  };

  if (DOE && typeof DOE.requestPermission === "function") {
    // iOS Safari: permission must be requested from a user gesture.
    const ask = () => {
      DOE.requestPermission?.()
        .then((state) => {
          if (state === "granted") attach();
        })
        .catch(() => {});
    };
    window.addEventListener("touchend", ask, { once: true });
    window.addEventListener("click", ask, { once: true });
  } else {
    // Android / desktop with a sensor: no permission needed.
    attach();
  }
}
