# 🎵 Rusic - Music Visualizer - Rust + WASM + Webgl2

**Rusic** is a high-performance WebGL music visualizer built with **Rust + WebAssembly** and driven by the **Web Audio API**.  
It renders a neon, mirrored frequency spectrum in real time using **WebGL 2.0**, compiled to the web via `wasm-pack`.

> Rust handles the rendering pipeline.  
> JavaScript handles audio analysis and UI.  
> The browser does the rest.

---

## ✨ Features

- ⚡ **Rust → WebAssembly** for fast rendering
- 🎨 **WebGL 2.0** neon bar visualizer
- 🎧 **Web Audio API** FFT (128 frequency bins)
- 🔁 Real-time audio → WASM frame updates
- 🖼️ Resolution-aware canvas resizing (HiDPI ready)
- 📂 Drag-and-drop MP3 / WAV uploads
- ⌨️ Keyboard & media controls
- 🌈 Gradient glow shaders (Cyan → Indigo → Pink)

---

## 🧠 How It Works

### Audio Pipeline
1. Browser loads audio into `<audio>`
2. `AudioContext + AnalyserNode` extracts:
   - Frequency data (`Uint8Array`)
   - Time-domain data (`Uint8Array`)
3. Audio data is sent **every frame** into Rust via:
   ```js
   render_frame(frequencyData, timeData)
   ```

### Rendering Pipeline

* Rust generates bar geometry per frame
* Data is uploaded to a dynamic WebGL buffer
* Vertex shader positions bars in screen space
* Fragment shader applies:

  * Gradient color by bar index
  * Glow intensity by volume
  * Subtle transparency for glass effect

---

## 🛠️ Tech Stack

* **Rust**
* **WebAssembly (wasm-bindgen)**
* **WebGL 2.0**
* **Web Audio API**
* **Vanilla JS + HTML + CSS**
* **GSAP** (background animations)
* **Lucide Icons**

---

## 📦 Build & Run

### Prerequisites

* Rust (stable)
* `wasm-pack`
* A local web server (required for WASM)

Install `wasm-pack` if needed:

```bash
cargo install wasm-pack
```

---

### Build WASM

```bash
wasm-pack build --target web
```

This generates:

```
/pkg
  ├── rusic.js
  ├── rusic_bg.wasm
```

---

### Run Locally

Because browsers block WASM over `file://`, you must use a server.

**Option 1: Python**

```bash
python3 -m http.server
```

**Option 2: Node**

```bash
npx serve .
```

Then open:

```
http://localhost:8000
```

---

## 🖼️ Canvas & Resize Handling

The canvas automatically:

* Matches container size
* Scales for device pixel ratio
* Notifies Rust on resize via:

```js
update_canvas_size(width, height)
```

Rust updates:

```rust
gl.viewport(0, 0, width as i32, height as i32);
```

---

## 🎛️ Controls

* ▶️ Play / Pause
* ⏮️ / ⏭️ Track controls
* 🔊 Volume slider + mute
* 🖱️ Click progress bar to seek
* 📁 Drag & drop audio files
* ⌨️ Spacebar toggles play/pause

---

## ⚠️ Browser Notes

* AudioContext **must be initialized via user gesture**
* WebGL 2.0 required
* Best tested on:

  * Chrome
  * Edge
  * Firefox

---

## 🚀 Ideas for Extensions

* Circular / radial visualizer
* Beat detection
* Shader-based blur & bloom
* Preset visual modes
* MIDI / microphone input
* OffscreenCanvas + Web Workers

---

Built with Rust, shaders, and questionable music taste.
