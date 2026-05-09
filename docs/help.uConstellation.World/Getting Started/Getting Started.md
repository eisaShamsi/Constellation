---
aliases:
  - Getting Started
  - System Requirements
  - Minimum Requirements
  - Hardware Requirements
  - PC Requirements
description: What you need on your computer to run Constellation, plus the recommended specs for comfortable everyday use and the optional Sight v5 source classifier.
---

# Getting Started — System Requirements

## What does Constellation need?

Constellation is designed to run smoothly on a 10-year-old laptop. The app itself is small, fast, and works fully offline — your notes never leave your machine.

That said, three different setups give three different experiences:

1. **Minimum** — what every Constellation user needs.
2. **Recommended** — what gives you a comfortable everyday experience, especially with large libraries (5,000+ notes) and the Second Screen feature.
3. **Constellation Sight v5** — what the optional larger source-classifier needs (the small built-in classifier runs on the same hardware as Constellation core, no extra requirements).

---

## Minimum — to run Constellation

You need a computer that can do all of these:

- **Operating system**: Windows 10 or 11, macOS 11 (Big Sur) or later, or 64-bit Linux from the last 3 years (Ubuntu 22.04, Fedora 38, Debian 12, or equivalent).
- **Processor**: Any 64-bit computer made in 2013 or later — any Intel or AMD desktop or laptop, or any Apple Silicon Mac.
- **Memory**: 4 GB free RAM.
- **Disk space**: 200 MB for Constellation itself, plus space for your notes (Markdown files — typically 1–10 MB per 1,000 notes).
- **Internet**: **Not required.** Constellation runs fully on your machine. Internet is only needed if you choose to download an optional add-on.

**If you can run a modern web browser, you can run Constellation.**

---

## Recommended — for comfortable everyday use

If you have a large library, plan to use the Second Screen feature, or want everything to feel snappy:

- **Processor**: 8-core modern processor (Intel or AMD from 2018 onward, or any Apple Silicon Mac).
- **Memory**: 8 GB free RAM.
- **Disk space**: 1 GB for Constellation and its caches.
- **Display**: Full HD (1920×1080) or higher. A second monitor unlocks the Second Screen feature.

---

## For Constellation Sight v5 — the source classifier

[Constellation Sight](../Constellation%20Sight/Constellation%20Sight.md) v5 includes a feature that suggests source-types for your notes (perception, inference, testimony, mass-transmission, comparison, postulation, non-apprehension, memory, innate disposition, inspiration, revelation — drawn from the Universal Epistemic Content Taxonomy).

The classifier runs in two tiers:

### Built-in classifier (default — no extra requirements)

A small classifier ships inside Constellation. **It runs on the same hardware as Constellation core — no extra setup, no internet, no download.** It works in all 15 supported languages out of the box.

Accuracy is good enough that you only have to confirm or correct what it suggests in the Source Review panel.

### Optional larger classifier (downloadable — for higher accuracy)

For higher classification accuracy — particularly with Arabic, Hebrew, Persian, and other non-Latin scripts — you can download a larger classifier from **Settings → AI**:

- **Processor**: 4-core or better.
- **Memory**: 4 GB free RAM during a classification run, on top of Constellation's normal usage.
- **Disk space**: 1.5 GB additional for the model file.
- **Internet**: required for the **one-time download** (~1.1 GB). After download, the classifier runs entirely on your machine — no internet ever.

**Optional GPU acceleration** (NVIDIA, Apple Metal, or Vulkan-compatible) speeds the classifier 5–20× but is **not required** — everything works on the CPU alone.

---

## What if I have an older or lower-spec machine?

You can still run Constellation. The minimum spec above is honest — anything that meets it will run the app, edit notes, search, and use every shipped feature including the built-in Sight classifier.

The only feature that benefits from more recent hardware is the optional larger Sight classifier. If your machine doesn't meet its requirements, the built-in classifier still works.

---

## How can I check my computer's specs?

- **Windows**: Press `Win + I` → **System** → **About**. You'll see processor, installed RAM, and Windows edition.
- **macOS**: Click the Apple menu (top-left) → **About This Mac**. You'll see chip/processor, memory, and macOS version.
- **Linux**: Open a terminal and run `lscpu` for processor, `free -h` for memory, `df -h` for disk space.

If anything is unclear, ask in the Constellation community — fellow users are happy to help confirm whether your specific machine will run the app.
