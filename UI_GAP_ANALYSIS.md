# notchA: why the UI works but is not fluid, and what is missing

Correction: the pure-Rust port already exists at `notchA\notchA`. The earlier analysis
(`codenotch\windows\ui-port\06-PURE-RUST-UI-ANALYSIS.md`) assumed a greenfield rewrite;
this file is the code-specific version. Reference line numbers are from `notchA\_ref_ui\`
(the **original** Tauri HTML, with full i18n). The English-only edit was applied only to
`codenotch\windows\codenotch\ui\notch.html` and its copy in `tmp\` — `_ref_ui` is untouched.

## 1. Current architecture

| Layer | Where | State |
|---|---|---|
| UI framework | `Cargo.toml` 9: Iced 0.14 + `canvas` + `wgpu` + `svg` + `image` | Working |
| Notch drawing | `src\notch_view.rs`: `compute_layout` 241–384, `canvas::Program::draw` 434–598, `build_pill_contour` 600–658, `build_tail_path` 660–690, `draw_arc` 692–719, `draw_dashed_arc` 721–738, `draw_orb_and_grip` 740–776, `view` 780–913, `render_card` 954–1183 | Static shapes, all interaction wired |
| Animation clock | `src\main.rs` 718–740: 50 ms watchdog + conditional 33 ms `Msg::Tick` | Exists, but only drives a few scalars |
| Tick body | `src\main.rs` 224–257: `anim_secs`, `refreshing_ids`, `grip_out += diff*0.25`, fold timers, backdrop probe | No spring, no easing, no transitions |
| Event bridge | `src\main.rs` 743–758 `UI_RX`, `core.rs` 40–63 `UiEvent` | Working |
| Native window move | `src\main.rs` 626–648 `place_notch_task`; `core.rs` 579–581 emits `GlideNotch`, handled immediately at `main.rs` 336 | No interpolation |
| Drag | `src\carry.rs` 168–247: 12 ms pointer loop, edge/corner math | Position only, no carried visual |
| Taskbar | `src\taskbar_render.rs` 167–441 + `taskbar_native.rs` 250–325 (50 ms `SetTimer`) | Working, stepped spin/pulse |
| Backdrop | `src\backdrop.rs` 69–181 + GDI 192–252 | Detection works; switch is instant |
| Themes | `src\theme.rs` 37–128 (palettes, `tone`, `lerp_color`) | Only colour interpolation in the whole port |

## 2. What already works (don't rebuild)

- Pill silhouette + inverse fillets: `notch_view.rs` 472–475, 600–658.
- Rings: headline arc, weekly ring, dashed weekly arc, stale dimming, press shrink, glyphs,
  percentage text: `notch_view.rs` 477–585, 692–738; glyph loading in `glyphs.rs` 101–287.
- Activity spin (12 steps / 1.2 s) and attention pulse (6 steps / 1.1 s): `notch_view.rs` 553–565.
- Hover card content and tail path: `notch_view.rs` 334–370, 868–912, 954–1183.
- Settings, reset card, tray, fullscreen/topmost, taskbar native renderer: all present and functional.
- Adaptive pill detection (`DARK_PILL=0.0`, `LIGHT_PILL=0.914`, `HYSTERESIS=1.5`).

## 3. Why it feels static — the five root causes

1. **No motion primitives.** No `bezier()` solver, no `springStep()`, no easing library, no
   transition object (start/end/duration/cancel/retarget). `theme.rs::lerp_color` is the only
   interpolation in the crate. Reference: `_ref_ui\notch.html` 2069–2083.
2. **Boolean visual state.** `card_open`, `folded`, `pressed`, `stale`, `orb_hovered` switch
   geometry instantly: `notch_view.rs` 334–370, 482–485, 747–775. CSS did this with transitions;
   Iced redraws only what state says.
3. **Fragmented clocks.** 33 ms conditional tick (`main.rs` 735–737), 50 ms watchdog (721),
   50 ms taskbar `SetTimer`, ~12 ms drag loop (`carry.rs` 242). No shared clock, no `DwmFlush`
   (the original Rust app used it in `main.rs::next_frame`).
4. **No goo/metaball pipeline.** Orb and grip are static arcs/discs/dots (`notch_view.rs` 740–776).
   The SVG filter chain and its animated `stdDeviation` have no equivalent.
5. **Carry has no overlay.** `carry.rs` moves the window directly; the reference draws a whole
   second animated shape while dragging (`_ref_ui\carry.html` 102–312, 441–469).

## 4. Missing UI components, with reference source and target file

| Missing component | Reference source (`_ref_ui\...`) | Implement in |
|---|---|---|
| Fold clip/contour morph + staggered cells | `notch.html` 205–258 (clip `.36s cubic-bezier(.32,.72,.24,1)`, 28 ms stagger), 2381–2407 (`FOLD_GRACE=450`, `TURNS_BACK=500`, `ARC_RETURN=400`, `ARC_QUICK=260`) | new `src\motion.rs` + `notch_view.rs` 451–475 |
| Card/tail glide, follow, end, retarget | `notch.html` 104–107, 109–130, 151–203, 1663–1719, 1727–1747 | `motion.rs` + `notch_view.rs` 334–370 |
| Native window glide (fullscreen/taskbar shift) | original Rust `main.rs::glide_notch`/`glide_curve` (in `tmp\src\main.rs` 401–468) — **already written, copy back** | `main.rs` 626–648 |
| Press duration → turn → settle | `notch.html` 1899–1926 (`PRESS_MIN=380`, `PRESS_MAX=6000`, rotate 950 ms `cubic-bezier(.32,0,.14,1)`), 1312–1314 (bounce 430 ms) | `motion.rs` + `notch_view.rs` 482–483 |
| Grip spring + dot morph + neck | `notch.html` 1982–2156 (`bezier`, `springStep`, `gripTick`, `drawGrip`) | `motion.rs` + `notch_view.rs` 740–776 |
| Orb metaballs / strand / disc-to-arc / release | `notch.html` 260–306, 313–357, 2258–2375 | new `src\goo.rs` + `notch_view.rs` 740–776 |
| Carried handle frames | `notch.html` 2167–2229 and `carry.html` 321–411 (duplicated — implement once) | `goo.rs`, shared by both |
| Carried notch overlay: stretch, corner morph, flares | `carry.html` 162–240, 260–312 (`FOLLOW{.2,.84}`, `SETTLE{.3,.74}`, `MAX_STRETCH=.14`, `STRETCH_SPEED=2600`) + `tick` 441–457 | new `src\carry_view.rs` |
| Adaptive palette crossfade / rest outline fade | `notch.html` 232–247 (rest outline `.2s` with `.18s` delay) | `notch_view.rs` 451–470 |
| Reduced motion gate | `notch.html` 104–107, 227, 258, 282; `carry.html` 58, 415 | one global flag in `motion.rs` |
| Taskbar press easing | `notch.html` 73–74 + original `taskbar_render.rs` `PRESSED` transform | `taskbar_render.rs` 231–233 |

## 5. Implementation plan (smallest useful increment first)

**Phase 0 — motion foundation (pure math, unit-testable, no UI risk)**
- Add `src\motion.rs`: `bezier(x1,y1,x2,y2)` (bisection), `spring_step(gap, velocity, elapsed, response, damping)`,
  named curves (`GRIP_OUT`, `GRIP_IN`, `ARC_DIVIDE`, `EASE_IN_OUT`, `EASE_OUT`, `RELEASE`),
  `Tween<T>` with start/end/duration/easing/cancel/retarget, and `motion_enabled`.
- Port `glide_curve` from `tmp\src\main.rs` 409–421 verbatim.
- Add one owned animation clock in `main.rs`, replace the conditional 33 ms tick with `DwmFlush`-paced
  frames while any tween/spring is active. Keep the 50 ms watchdog separate.

**Phase 1 — the visible "snap" fixes (biggest perceived win, low risk)**
- Card/tail glide: replace boolean placement in `notch_view.rs` 334–370 with a `Tween` over
  card x/y, `.3s cubic-bezier(.33,1,.68,1)`, 400 ms fallback, retarget on ring change.
- Fold: drive a `fold_t: f64` 0→1 over 360 ms; interpolate the contour between
  `build_pill_contour` and the rest rect; stagger cells by `i * 28ms`.
- Press: spring the `0.93` scale instead of hard-switching; add `turnReading` rotation.
- Adaptive palette: lerp colours via `theme::lerp_color` over ~200 ms.

**Phase 2 — signature motion**
- `goo.rs`: shared `quarter`, `strand`, `discMerge`, `handleFrame`, neck/strand geometry.
  Start with an **analytic geometry approximation** (paths that look metaball-ish), then optionally
  add a shader/pixel pass behind a trait — this de-risks the hardest item.
- Grip spring and orb arc divide/return/release driven by the Phase 0 clock.

**Phase 3 — carry overlay**
- New transparent window + `carry_view.rs`; reuse `carry.rs` geometry and the shared `goo.rs`
  helpers so the handover frame matches. Add velocity stretch and corner morph.

**Phase 4 — polish**
- Reduced-motion, taskbar press easing, reset card `appear .18s ease-out`, final timing parity.

## 6. Constraints to preserve while adding motion

- `steps(12)` / `steps(6)` cadence for the activity arc is intentional — 60 fps transparent-window
  repaints stutter the whole desktop. Do not "smooth" those two by rendering every frame.
- DPI/zoom logic (`main.rs` `zoom_notch`/`report_dpr`) already handles multi-monitor scaling;
  re-test after any window-animation change.
- `notch.html` in `CARD_ONLY` mode is the taskbar hover card — one card renderer must serve both.
- Keep the backend untouched: `state.rs::Snapshot` is the data feed; `lang_resolved` is ignored now.
