# Embedding a Real Chromium Pane in Marley (CEF OSR → GPU Texture) — Feasibility Analysis

**Question:** With time *not* a constraint, is it feasible to embed a real Chromium browser as an
interactive pane in Marley — Chromium rendering web pages whose output is composited into Marley's
own GPU scene (a texture node in the warpui/GPUI Metal/wgpu retained tree), interactive — using
**CEF (Chromium Embedded Framework) offscreen rendering (OSR)**? What is the quality ceiling, and
what are the genuine hard limits that no amount of time removes?

**Date:** 2026-06-27 · **Target stack:** macOS-first, Metal/wgpu, warpui/GPUI · **Engine:** CEF OSR

---

## 1. Overall Verdict (blunt)

**Yes — it is feasible, and it is not research. It is a solved, production-proven engineering
pattern.** "Render a real Chromium page into a host-owned texture and feed it synthetic input" is
exactly what **OBS Studio's browser source** ships to millions of installs today, and it does so on
Marley's *exact* macOS data path: CEF OSR → `OnAcceleratedPaint` → an `IOSurface` →
`gs_texture_create_from_iosurface` → a node in OBS's own GPU scene
([OBS browser rendering, deepwiki](https://deepwiki.com/obsproject/obs-browser/2.3-browser-source-rendering)).
Swap "OBS scene" for "warpui/GPUI Metal scene" and the architecture is identical.

The realistic **quality ceiling is "real Chromium-grade web, minus DRM"**: the full open web at
native GPU fidelity — WebGL, WebGPU/Dawn, canvas, WebRTC, non-DRM `<video>`, full JS — composited
zero-copy into Marley's scene and fully interactive (mouse, keyboard, scroll, IME, popups,
context menus, intra-page drag). What you do **not** get, regardless of effort, is **premium
DRM/EME video** (Netflix, Disney+, Spotify-premium-web) and **legal redistribution of proprietary
codecs (H.264/AAC) and the Widevine CDM** without commercial agreements with Google/MPEG-LA.
Those are licensing/content-protection walls, not engineering ones.

**Proof of how proven:** OBS (open-source reference implementation, the closest analog to Marley's
goal), plus the broad CEF-OSR install base — Steam, Battle.net, Epic/GOG/Ubisoft launchers,
Spotify, Amazon Music, Evernote — all of which embed windowless Chromium and composite it
([howtogeek](https://www.howtogeek.com/436841/that-native-app-is-probably-just-an-old-web-browser/)).
The mechanism is among the most battle-tested embedding stacks in existence.

**Confidence: HIGH** on feasibility, the macOS zero-copy path, and the DRM ceiling. **MEDIUM** on
the day-one stability of the *mainline accelerated path* (young API, version-specific bugs — but a
permanently-reliable CPU fallback exists).

---

## 2. What is SOLVED / Proven (no effort risk)

### 2.1 The OSR model itself
CEF's offscreen rendering is a documented, first-class integration surface. You create a windowless
browser with `CefWindowInfo::SetAsWindowless`, implement `CefRenderHandler`
(`GetViewRect` / `GetScreenInfo` / `GetScreenPoint` / `OnPaint` / `OnAcceleratedPaint` /
`OnPopupShow` / `OnPopupSize` / `OnCursorChange` / `OnTooltip` / `OnImeCompositionRangeChanged`),
and CEF renders with no OS window while you own sizing and input
([CEF forum OSR](https://magpcss.org/ceforum/viewtopic.php?f=6&t=15495)). This is the intended,
supported contract — not a hack.

### 2.2 The CPU baseline (`OnPaint`) — always works
The software path is the universally-available floor: CEF software-composites the page into a BGRA
backing buffer and hands you a pixel pointer plus per-frame dirty rectangles; you sub-upload only
the dirty rects to a GPU texture. With `--disable-gpu-compositing` it's called straight from the
compositor. This is the path most production CEF apps use, and it is rock-solid
([OBS rendering](https://deepwiki.com/obsproject/obs-browser/2.3-browser-source-rendering)).
**This guarantees Marley a shipping path even if the accelerated path is ever broken by a Chromium
bump** — the cost is one CPU→GPU upload per frame.

### 2.3 The macOS zero-copy GPU path — proven in production
On macOS the accelerated path returns an **`IOSurface`**. Verified against the primary CEF struct
docs: `cef_accelerated_paint_info_t` carries `shared_texture_io_surface` (macOS),
`shared_texture_handle` (Windows, D3D11 without keyed mutex), `planes`/`plane_count`/`modifier`
(Linux dmabuf/EGL), and a `format` (`cef_color_type_t`)
([cef_accelerated_paint_info_t, CEF 132.3 docs](https://cef-builds.spotifycdn.com/docs/132.3/structcef__accelerated__paint__info__t.html)).
`IOSurface` is Apple's first-class zero-copy GPU-sharing primitive; it wraps directly into an
`IOSurface`-backed `MTLTexture` via `newTextureWithDescriptor:iosurface:plane:` with no copy
([MTLTexture iosurface, Apple](https://developer.apple.com/documentation/metal/mtltexture/1516104-iosurface)),
and on Apple-Silicon unified memory it is genuinely copy-free. OBS ships exactly this
(`gs_texture_create_from_iosurface((IOSurfaceRef)info.shared_texture_io_surface)`), confirmed
against the OBS implementation
([OBS rendering](https://deepwiki.com/obsproject/obs-browser/2.3-browser-source-rendering)).
**So "Chromium page → Metal texture node in Marley's GPUI scene, zero readback" is a supported,
shipping data path — the same primitive OBS uses.**

### 2.4 The accelerated path is now MAINLINE (not a downstream patch)
Important history correction: for years the shared-texture path was a fragile out-of-tree patch.
OBS's texture-sharing patch died at CEF 103 when it broke after Chromium ~102/103, and maintainer
Marshall Greenblatt stated `OnAcceleratedPaint` then "requires external changes on all platforms"
with official restoration "up to Google… not currently planned or staffed"
([OBS discussion #3853](https://github.com/obsproject/obs-studio/discussions/3853),
[CEF forum 'Future of onAcceleratedPaint'](https://magpcss.org/ceforum/viewtopic.php?f=10&t=19401)).
**That was superseded:** GPU shared-texture OSR was **re-mainlined into official CEF around the
v124 release (May 2024; the macOS IOSurface variant in CEF 124.3.1)** as the new
`cef_accelerated_paint_info_t` interface across all three platforms, and the same upstream fix
landed in Electron (PR #42001)
([CEF issue #4057](https://github.com/chromiumembedded/cef/issues/4057),
[Electron PR #42001](https://github.com/electron/electron/pull/42001)). In current CEF (132–149)
`OnAcceleratedPaint` is a built-in feature.

### 2.5 Interactivity is first-class
Mouse/keyboard/scroll/focus injection is the intended OSR contract:
`SendMouseClickEvent` / `SendMouseMoveEvent` / `SendMouseWheelEvent` / `SendKeyEvent` /
`SendFocusEvent` ([CEF forum](https://magpcss.org/ceforum/viewtopic.php?f=6&t=261),
[qwertzui11/cef_osr](https://github.com/qwertzui11/cef_osr)). Cursor (`OnCursorChange`), tooltips
(`OnTooltip`), and host-owned context menus (`CefContextMenuHandler::RunContextMenu`) are clean host
callbacks ([cef_context_menu_handler.h](https://github.com/chromiumembedded/cef/blob/master/include/cef_context_menu_handler.h)).

### 2.6 The Rust path is real (no C++ shim required for CEF)
`tauri-apps/cef-rs` is actively maintained and tracks current Chromium — verified latest release
**`cef-v149.2.0+149.0.5` (Jun 2026), 309 releases**, bindgen `sys` crate + a higher-level safe
`cef` crate, Linux/macOS/Windows on x86_64 + ARM64
([cef-rs GitHub](https://github.com/tauri-apps/cef-rs), [crates.io/crates/cef](https://crates.io/crates/cef)).
It exposes exactly the OSR surface area Marley needs: `RenderHandler`, `OnPaint`,
`OnAcceleratedPaint`, `AcceleratedPaintInfo` (+ `NativePixmapPlaneInfo`), `BrowserHost` windowless
control, `MouseEvent`/`KeyEvent`, and `execute_process` ([docs.rs/cef](https://docs.rs/cef)).
**You do not need a C++ shim for the CEF side.**

---

## 3. Solvable WITH EFFORT (real cost, no wall)

These are genuine engineering line-items. None is blocked by time, licensing, or architecture.

### 3.1 IOSurface → MTLTexture → wgpu interop glue
There is **no first-class public wgpu `IOSurface`-import API**. You drop to **`wgpu-hal`'s Metal
backend**, wrap the `IOSurface`-backed `MTLTexture` via unsafe hal interop, and hand it back as a
`wgpu::Texture` for the warpui/GPUI scene
([docs.rs/wgpu Texture](https://docs.rs/wgpu/latest/wgpu/struct.Texture.html), [docs.rs/cef](https://docs.rs/cef)).
This is **unsafe Rust (objc2/metal crates), not C++.** A thin Obj-C++ bridge for the
IOSurface→MTLTexture step is *optional convenience* (encse/cef-test does it that way) but avoidable
([encse/cef-test](https://github.com/encse/cef-test)).

### 3.2 Synchronization (the sharpest perf edge)
CEF hands you a **transient surface from a small rotating pool with NO fence/sync object** in the
callback. Without correct synchronization you get tearing/flicker or stale frames; you must
sample/copy within the callback's validity window
([scalibq CEF developers](https://scalibq.wordpress.com/2024/08/06/cef-developers/),
[CEF forum 16769](https://www.magpcss.org/ceforum/viewtopic.php?f=7&t=16769)). Solvable with correct
timing, but it is the part most likely to bite a naive implementation.

### 3.3 No damage rects on the accelerated path
The accelerated path does **not** expose dirty/damage rectangles — each callback is effectively a
full-surface treatment ([CEF issue #3730](https://github.com/chromiumembedded/cef/issues/3730)).
For a compositor that re-samples a texture every frame this is minor extra GPU work, not a problem.
(The CPU `OnPaint` path *does* give dirty rects.)

### 3.4 macOS multi-process packaging
macOS **mandates the multi-process model** (single-process unsupported). Marley must ship four
Helper `.app` bundles (Helper, Helper (GPU), Helper (Renderer), Helper (Plugin)) inside
`Contents/Frameworks`, each with its own `Info.plist`/bundle-id and **JIT entitlements**, plus the
`Chromium Embedded Framework.framework`, then notarize (Developer ID is the unconstrained route;
App Sandbox/MAS works but adds entitlement constraints)
([CEF macOS helper requirement](https://bitbucket.org/chromiumembedded/cef/issues/2737/macos-76-requires-multiple-helper-app),
[obs-browser PR #252](https://github.com/obsproject/obs-browser/pull/252)). Structural packaging
complexity, fixed and known — not a blocker.

### 3.5 Message-loop integration with GPUI/winit
`multi_threaded_message_loop` is **unsupported on macOS** (Cocoa requires UI on the main thread).
You drive `external_message_pump` + `CefDoMessageLoopWork`, interleaved with Marley's own
GPUI/winit run loop, pumped frequently (~30/s) plus on demand via `OnScheduleMessagePumpWork`
([cef_app.h apidocs](https://magpcss.org/ceforum/apidocs/projects/(default)/cef_app.h.html),
[external message pump](https://bitbucket.org/chromiumembedded/cef/issues/2968/documentation-of-external-message-pump)).
Known integration pattern.

### 3.6 IME / non-Latin (CJK) input
Solvable but the **single most labor-intensive input item**. CEF v55+ has the full API
(`ImeSetComposition` / `ImeCommitText` / `ImeFinishComposingText` / `ImeCancelComposition` +
`OnImeCompositionRangeChanged` delivering `character_bounds`), and cefclient ships OSR IME samples
on all three platforms ([CEF issue 1675](https://bitbucket.org/chromiumembedded/cef/issues/1675),
[mac text input client](https://github.com/bkeiren/cef/blob/master/libcef/browser/text_input_client_osr_mac.mm)).
But it is **not automatic**: Marley must wire the OS IME framework (`NSTextInputClient` on macOS) to
these calls and position the candidate window via `firstRectForCharacterRange`. The infamous
"candidate window stuck top-left" is a host bug, fixable with correct `GetScreenPoint`/`GetScreenInfo`
(device_scale_factor) ([CEF forum 14123](https://www.magpcss.org/ceforum/viewtopic.php?f=6&t=14123)).

### 3.7 `<select>` / autofill popups
Native dropdowns/autofill are **not OS windows** in OSR — Chromium renders them as a second
`PET_POPUP` layer. The host keeps a second buffer, honors `OnPopupShow`/`OnPopupSize`, and
composites the popup quad on top — *natural for a GPU host* (a second textured quad)
([CEF forum 12806](https://www.magpcss.org/ceforum/viewtopic.php?f=6&t=12806)). The lasting jank:
the popup is clipped to the CEF view rect and won't overflow the host window like a real OS menu
unless you route `PET_POPUP` into your own scene layer or oversize the OSR surface — CefSharp/WPF
embedders repeatedly hit this ([CefSharp #2820](https://github.com/cefsharp/CefSharp/issues/2820)).
Solvable in a custom compositor; never "free."

### 3.8 Drag-and-drop
Intra-page DnD is solvable via `CefRenderHandler::StartDragging` + `DragTarget*`/`DragSource*` driven
in lockstep with injected mouse events. **Cross-application DnD** (file into the page, page content to
another app) CEF explicitly **cannot do alone** — "CEF does not have a window to act as a drag source
or drop target" — so the host must bridge to the OS drag system (`NSDraggingSource`/`NSDraggingDestination`)
([CEF forum 560](https://www.magpcss.org/ceforum/viewtopic.php?f=6&t=560)). Achievable, but the most
fiddly piece and permanently host-maintained glue.

### 3.9 Recent-API stability
Version-specific breakage exists: `shared_texture_handle` reported null on some CEF 143 Windows
release builds (works in debug/142); `OnAcceleratedPaint` occasionally not called (falls back to
`OnPaint`); Linux accelerated path needs explicit ANGLE/Ozone config
([CEF #4057](https://github.com/chromiumembedded/cef/issues/4057),
[#3953](https://github.com/chromiumembedded/cef/issues/3953)). **Pin and validate a known-good CEF
build; keep the `OnPaint` fallback wired.** Not a ceiling — a maintenance discipline.

---

## 4. HARD CEILINGS (persist regardless of time)

These do not yield to engineering effort. They are licensing, content-protection, or fixed-API walls.

### 4.1 Widevine / premium EME video — GENUINELY BLOCKED
- The **Widevine CDM is a closed Google binary you may not legally redistribute/bundle** without a
  signed Google/Widevine Master License Agreement; recent CEF dropped the component-updater
  auto-download, so DRM does not work out of the box and Marley cannot ship a CDM
  ([CefSharp #1934](https://github.com/cefsharp/CefSharp/issues/1934),
  [CEF #3820](https://github.com/chromiumembedded/cef/issues/3820),
  [Google Widevine license](https://developers.google.com/widevine/open-source/license-1)).
- **VMP (Verified Media Path):** premium services require `.sig` files signed by Google's
  certificate, issued only to licensees. A self-built CEF **fails the VMP signature handshake at the
  license server** — Netflix/Disney+/Spotify-premium will refuse playback no matter the effort
  ([CEF #3404](https://github.com/chromiumembedded/cef/issues/3404),
  [bscan Widevine DRM](https://bscan.info/blog/widevine-drm)).
- **Hardware-secure L1 / 4K (HDCP 2.2 + secure decode + device provisioning)** is **structurally
  incompatible with OSR by design**: OSR composites the decoded frame into your app's untrusted
  memory, which is exactly what the secure path refuses — you get black frames or a downgrade
  ([pallycon why no UHD](https://medium.com/pallycon/why-cant-i-watch-netflix-in-ultra-hd-on-my-chrome-browser-525933dad5bb)).
- **Grey zone:** software **L3** DRM *can* be made to function if the **end user** supplies an
  extracted CDM, but redistribution is restricted, quality caps at ~720p/1080p, and many services
  still require VMP ([Netflix on Asahi](https://www.da.vidbuchanan.co.uk/blog/netflix-on-asahi.html)).

  **Net: you cannot ship a legal, premium-DRM-capable Netflix/Spotify pane.**

### 4.2 Proprietary codec redistribution — LICENSING GATE
Shipping CEF/Chromium with **H.264/AAC** support requires a redistribution license (MPEG-LA);
standard CEF builds omit proprietary codecs, breaking common video. Legal/cost gate, not code
([CEF #1631](https://bitbucket.org/chromiumembedded/cef/issues/1631/add-support-for-widevine-cdm)).

### 4.3 Fixed-API gaps (only patchable by forking CEF/Chromium)
- **No damage rects on the accelerated path** (issue #3730) — full-frame re-handling per paint.
- **No GPU completion fence** from CEF on the shared surface — you can only manage timing around the
  callback, never get an explicit fence.
- **No top-level OS popup window** — CEF will never spawn a real menu window for `<select>`; you own
  the compositing.

### 4.4 Permanent maintenance burden (the quiet ceiling)
This is the real long-term cost, not a one-time build: **CEF/Chromium upkeep is perpetual.** Chromium
ships a major version roughly every ~4 weeks; CEF tracks it; security fixes are continuous. Marley
inherits a multi-hundred-MB binary framework, a 4-helper process model, and a treadmill of revalidating
the (young) accelerated path against each bump. The per-OS IME/DnD/popup glue is also forever
host-maintained — it is never inherited from a CEF widget the way windowed mode would inherit it.
**None of this blocks shipping; all of it is a standing tax.**

---

## 5. Quality Ceiling vs Standalone Chrome

| Dimension | Embedded CEF-OSR pane in Marley | Standalone Chrome |
|---|---|---|
| Open-web rendering fidelity (HTML/CSS/JS) | Identical (same Chromium) | Identical |
| WebGL / WebGPU(Dawn) / canvas | Full, GPU-accelerated under accelerated OSR | Full |
| Non-DRM `<video>` (if codecs licensed) | Works; HW decode driver-fragile in OSR | Works |
| Compositing into Marley's Metal scene | **Yes, zero-copy via IOSurface** | N/A |
| Frame pacing | OSR often defaults ~30fps; tune external-begin-frame toward 60 | 60+ native |
| Premium DRM (Netflix/Disney+/Spotify) | **No** (VMP/L1 wall) | Yes |
| H.264/AAC out of the box | No (license) | Yes (Google-licensed) |
| IME / `<select>` / cross-app DnD | Host-reimplemented per OS | Inherited from OS widget |
| Accessibility tree | Degraded/extra work in OSR | Native |

**Ceiling in one line: real Chromium-grade open web at native GPU fidelity, composited zero-copy and
fully interactive — minus DRM/EME and minus out-of-the-box proprietary codecs.**

---

## 6. Why CEF (and not the alternatives) for Marley

Only CEF delivers "real Chromium-grade web." Each alternative is a *permanent* fidelity downgrade, not
a temporary one:

- **Ultralight** — game-proven, GPU-first, ~10x lighter, but a WebKit fork that **drops WebGL/WebRTC**,
  treats HTML5 video as experimental, and is **proprietary-licensed**
  ([ultralig.ht](https://ultralig.ht/)). Not real Chromium.
- **Servo** — best Rust-ecosystem fit (native Rust, `surfman` offscreen surfaces as cross-thread GL
  textures, active embedding work), but **not yet Chromium-grade web-compat**
  ([Servo embedding update](https://servo.org/blog/2024/01/19/embedding-update/),
  [servo/surfman](https://github.com/servo/surfman)). A real-content downgrade *today*.
- **WebView2** — Windows-only, can't control composition framerate, DComp doesn't cleanly interop
  with a D3D scene; **structurally unsuitable for a macOS-first cross-platform app**
  ([MS windowed-vs-visual hosting](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/windowed-vs-visual-hosting)).
- **OS WKWebView (macOS)** — no supported zero-copy texture-into-your-scene path; you'd be hosting a
  native subview, not compositing a texture node. Defeats the warpui/GPUI integration goal.

**For "real Chromium-grade in-app web composited into our GPU scene," CEF OSR is the only option.**

---

## 7. Concrete Recommendation for Marley

1. **Engine:** `tauri-apps/cef-rs` pinned to a validated recent build (149.x current). No C++ shim
   for CEF; one small block of **unsafe Rust** (objc2/metal + wgpu-hal Metal) for IOSurface→MTLTexture→wgpu.
2. **Render path:** Implement **both** — `OnAcceleratedPaint` (IOSurface → `MTLTexture` → wgpu →
   warpui/GPUI texture node, zero-copy) as the primary, **`OnPaint` (dirty-rect CPU upload) as the
   always-available fallback.** Auto-fall-back if the shared handle is null / callback never fires.
3. **Pane model:** one `CefBrowser` per pane, windowless; `GetViewRect` driven by the pane's layout
   rect; pause/teardown offscreen browsers when the pane isn't visible (OBS does this).
4. **Input:** forward GPUI input events to `SendMouse*`/`SendKey*`/`SendFocusEvent`; map cursor via
   `OnCursorChange`; render context menus/tooltips in Marley's own UI; composite `PET_POPUP` as a
   second scene quad that can overflow the pane.
5. **Message loop:** `external_message_pump` + `CefDoMessageLoopWork` interleaved with the GPUI run
   loop; pump on `OnScheduleMessagePumpWork`.
6. **Packaging:** 4 Helper `.app` bundles + framework in `Contents/Frameworks`, JIT entitlements,
   Developer ID notarization.
7. **Scope explicitly OUT:** premium DRM playback; assume open-web + (optionally licensed) codecs.
   Phase IME/CJK and cross-app DnD as later, clearly-bounded work.
8. **Budget the standing tax:** a recurring CEF-bump validation cycle, especially for the accelerated
   path.

---

## 8. Confidence & Gray Areas

- **HIGH confidence:** OSR feasibility; macOS IOSurface→Metal zero-copy (verified against CEF struct
  docs + OBS implementation); the DRM/VMP/L1 ceiling; cef-rs maturity (verified 149.x, 309 releases).
- **MEDIUM confidence / gray:** day-one robustness of the *mainline accelerated path* on the latest
  CEF (young API, real version-specific bugs — mitigated by the CPU fallback); WebGPU/Dawn behavior
  *specifically under OSR* (bundled, works as an in-page feature, less independently confirmed under
  OSR); HW video-decode reliability in OSR (driver-fragile). None of the gray areas threaten the core
  verdict — they affect the optimal-path polish, not whether the pane works.

---

## 9. Key Sources

- CEF struct (verified): https://cef-builds.spotifycdn.com/docs/132.3/structcef__accelerated__paint__info__t.html
- OBS browser rendering (verified): https://deepwiki.com/obsproject/obs-browser/2.3-browser-source-rendering
- cef-rs (verified 149.x): https://github.com/tauri-apps/cef-rs · https://docs.rs/cef · https://crates.io/crates/cef
- Accelerated path re-mainlined: https://github.com/chromiumembedded/cef/issues/4057 · https://github.com/electron/electron/pull/42001
- Cautionary history: https://github.com/obsproject/obs-studio/discussions/3853 · https://magpcss.org/ceforum/viewtopic.php?f=10&t=19401
- macOS helper requirement: https://bitbucket.org/chromiumembedded/cef/issues/2737/macos-76-requires-multiple-helper-app
- Damage-rect gap: https://github.com/chromiumembedded/cef/issues/3730
- Widevine licensing: https://developers.google.com/widevine/open-source/license-1 · https://github.com/cefsharp/CefSharp/issues/1934 · https://github.com/chromiumembedded/cef/issues/3404
- IOSurface→Metal: https://developer.apple.com/documentation/metal/mtltexture/1516104-iosurface
- Alternatives: https://ultralig.ht/ · https://servo.org/blog/2024/01/19/embedding-update/ · https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/windowed-vs-visual-hosting
