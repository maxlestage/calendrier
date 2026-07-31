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

  const onOrient = (e: DeviceOrientationEvent) => {
    const gamma = e.gamma ?? 0; // left/right tilt  (-90…90)
    const beta = e.beta ?? 0; //  front/back tilt (-180…180)
    if (raf) return; // throttle to one update per frame
    raf = requestAnimationFrame(() => {
      raf = 0;
      root.style.setProperty("--steel-angle", `${135 + gamma * 0.9}deg`);
      root.style.setProperty("--steel-x", `${clamp(50 + gamma * 0.8, 0, 100)}%`);
      root.style.setProperty("--steel-y", `${clamp(50 + (beta - 45) * 0.6, 0, 100)}%`);
    });
  };

  const attach = () => window.addEventListener("deviceorientation", onOrient, true);

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
