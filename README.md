<img src="media/demo.gif" width="800"/>

# Nemesis Molecular Visualization Engine

Nemesis is a real-time molecular visualization engine built in Rust and Vulkan. It renders PDB protein structures via custom parser feeding GPU instance buffers with per-atom position, radius, and element data; originally built to visualize MOGAD-relevant proteins.

## Features

Features

PDB Parser — loads real protein structures directly from .pdb files with native file dialog
Dual Render Modes — smooth runtime transitions between ball-and-stick and space-fill representations driven by a MoleculeObject abstraction over instanced draw calls
Phong Shading — procedural sphere mesh generation with index buffers and per-instance specular highlights
Full Vulkan Backend — explicit physical device selection, swapchain lifecycle management, pipeline recreation on resize, and deterministic GPU resource cleanup via Drop
egui Overlay — per-molecule intro animations with rotation, opacity fade, and slide-in text transitions

## Dependencies

- [Rust](https://rustup.rs) (stable)
- [Vulkan SDK](https://vulkan.lunarg.com/) 1.3+
- [glslc](https://github.com/google/shaderc)

## Building

```bash
git clone https://github.com/nicholasgallina/nemesis.git
cd nemesis
cargo run
```
