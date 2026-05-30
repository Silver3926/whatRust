from PIL import Image, ImageDraw
import os

SIZE = 1024
os.makedirs("src-tauri/icons", exist_ok=True)

def make(badge=False):
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([0, 0, SIZE, SIZE], radius=224, fill=(37, 211, 102, 255))  # WhatsApp green
    cx, cy, rad = SIZE // 2, SIZE // 2 - 40, 300
    d.ellipse([cx - rad, cy - rad, cx + rad, cy + rad], fill=(255, 255, 255, 255))  # bubble
    d.polygon([(cx - 120, cy + rad - 40), (cx - 40, cy + rad + 170), (cx + 40, cy + rad - 40)],
              fill=(255, 255, 255, 255))  # tail
    for dx in (-110, 0, 110):
        d.ellipse([cx + dx - 34, cy - 34, cx + dx + 34, cy + 34], fill=(37, 211, 102, 255))  # dots
    if badge:
        b = 220
        d.ellipse([SIZE - b - 24, 24, SIZE - 24, b + 24], fill=(234, 67, 53, 255))  # red dot
    return img

make().save("app-icon.png")
make().resize((128, 128)).save("src-tauri/icons/tray.png")
make(badge=True).resize((128, 128)).save("src-tauri/icons/tray-unread.png")
print("OK: app-icon.png + src-tauri/icons/tray*.png")
