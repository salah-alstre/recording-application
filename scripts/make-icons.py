"""Generates the Rimlight logo (1024px PNG) + tray variants. Pure numpy/Pillow, no external assets.
Concept: a lens-ring lit from one edge (the 'rim light') with a solid core dot."""
import math, sys, os
import numpy as np
from PIL import Image

S = 2048  # supersample
def make(size_out=1024, rec=False):
    y, x = np.mgrid[0:S, 0:S].astype(np.float32)
    cx = cy = S / 2
    dx, dy = x - cx, y - cy
    r = np.hypot(dx, dy) / (S / 2)          # 0..1 at the edge of the canvas
    ang = np.arctan2(dy, dx)
    # background: rounded square with soft vertical gradient
    n = 5.0
    q = (np.abs(dx) / (S * 0.5)) ** n + (np.abs(dy) / (S * 0.5)) ** n
    inside = np.clip((1.0 - q) * (S * 0.5) / 3.0, 0, 1)   # antialiased squircle
    t = y / S
    bg = np.stack([11 + 10 * (1 - t), 13 + 12 * (1 - t), 20 + 22 * (1 - t)], -1)
    img = np.zeros((S, S, 4), np.float32)
    img[..., :3] = bg
    img[..., 3] = inside * 255
    # ring
    R, W = 0.52, 0.085
    d = np.abs(r - R)
    ring = np.clip((W - d) / 0.004, 0, 1)
    # rim light: strongest at upper-left (angle ~ -135deg), fading around the ring
    a = (ang + math.radians(135) + math.pi) % (2 * math.pi) - math.pi
    lit = np.clip(1 - np.abs(a) / math.radians(150), 0, 1) ** 1.6
    cold = np.array([122, 162, 255], np.float32)
    warm = np.array([255, 196, 150], np.float32)
    col = cold[None, None] * (1 - lit[..., None]) + warm[None, None] * lit[..., None]
    base = np.array([54, 62, 84], np.float32)
    col = base[None, None] * (1 - 0.15 - 0.85 * lit[..., None]) + col * (0.15 + 0.85 * lit[..., None])
    for c in range(3):
        img[..., c] = img[..., c] * (1 - ring) + col[..., c] * ring
    # glow outside the ring on the lit side
    glow = np.exp(-((d) / 0.09) ** 2) * lit * 0.35
    for c in range(3):
        img[..., c] = np.clip(img[..., c] + glow * col[..., c] * (1 - ring), 0, 255)
    # core dot
    core = np.clip((0.19 - r) / 0.004, 0, 1)
    ccol = np.array([255, 92, 92], np.float32) if rec else np.array([236, 240, 255], np.float32)
    for c in range(3):
        img[..., c] = img[..., c] * (1 - core) + ccol[c] * core
    out = Image.fromarray(np.clip(img, 0, 255).astype(np.uint8), 'RGBA').resize((size_out, size_out), Image.LANCZOS)
    return out

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
icons = os.path.join(root, 'src-tauri', 'icons')
os.makedirs(icons, exist_ok=True)
logo = make(1024)
logo.save(os.path.join(icons, 'source.png'))
logo.save(os.path.join(root, 'public', 'logo.png'))
make(64).save(os.path.join(icons, 'tray-idle.png'))
make(64, rec=True).save(os.path.join(icons, 'tray-rec.png'))
print('icons ok')
