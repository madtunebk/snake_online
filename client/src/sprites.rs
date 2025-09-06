use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

pub struct SpriteAtlas {
    pub body: TextureHandle,
    pub apple: TextureHandle,
}

impl SpriteAtlas {
    pub fn new(ctx: &egui::Context, tile_px: usize) -> Self {
        let size = [tile_px, tile_px];

        let body_img = body_sprite(size);
        let apple_img = apple_sprite(size);

        let opts = TextureOptions::LINEAR;
        let body = ctx.load_texture("snake_body", body_img, opts);
        let apple = ctx.load_texture("apple", apple_img, opts);

        Self { body, apple }
    }
}

fn make_image(size: [usize; 2], mut f: impl FnMut(f32, f32, [usize; 2]) -> [u8; 4]) -> ColorImage {
    let (w, h) = (size[0], size[1]);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let v = (y as f32 + 0.5) / h as f32;
            let px = f(u, v, size);
            pixels.push(Color32::from_rgba_unmultiplied(px[0], px[1], px[2], px[3]));
        }
    }
    ColorImage {
        size: [w, h],
        pixels,
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn length2(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

fn sdf_circle(px: (f32, f32), center: (f32, f32), r: f32) -> f32 {
    length2(px.0 - center.0, px.1 - center.1) - r
}

fn sdf_rounded_rect(px: (f32, f32), center: (f32, f32), half: (f32, f32), r: f32) -> f32 {
    let dx = (px.0 - center.0).abs() - half.0;
    let dy = (px.1 - center.1).abs() - half.1;
    let ax = dx.max(0.0);
    let ay = dy.max(0.0);
    length2(ax, ay) + (dx.max(dy)).min(0.0) - r
}

fn body_sprite(size: [usize; 2]) -> ColorImage {
    make_image(size, |u, v, [w, h]| {
        let w = w as f32;
        let h = h as f32;
        let p = (u * w, v * h);
        let c = (w * 0.5, h * 0.5);
        let half = (w * 0.34, h * 0.34);
        let radius = w.min(h) * 0.22;
        let d = sdf_rounded_rect(p, c, half, radius);

        let a = smoothstep(1.5, -1.5, d);

        let light = (1.2 - ((p.0 + p.1) / (w + h))).clamp(0.8, 1.2);
        let mut r = (40.0 * light) as u8;
        let mut g = (180.0 * light) as u8;
        let mut b = (60.0 * light) as u8;

        let border = smoothstep(0.8, 0.0, d.abs());
        r = ((r as f32 * (1.0 - 0.2 * border)) as u8).min(255);
        g = ((g as f32 * (1.0 - 0.2 * border)) as u8).min(255);
        b = ((b as f32 * (1.0 - 0.2 * border)) as u8).min(255);

        let inner = smoothstep(-radius * 0.4, -radius * 0.9, d);
        r = r.saturating_add((20.0 * inner) as u8);
        g = g.saturating_add((25.0 * inner) as u8);
        b = b.saturating_add((20.0 * inner) as u8);

        let alpha = (255.0 * a) as u8;
        [r, g, b, alpha]
    })
}

fn apple_sprite(size: [usize; 2]) -> ColorImage {
    make_image(size, |u, v, [w, h]| {
        let w = w as f32;
        let h = h as f32;
        let p = (u * w, v * h);
        let c = (w * 0.5, h * 0.52);
        let r_apple = w.min(h) * 0.35;
        let d = sdf_circle(p, c, r_apple);

        let a = smoothstep(1.5, -1.5, d);
        let light = (1.15 - ((p.0 + p.1) / (w + h))).clamp(0.85, 1.15);
        let mut r = (210.0 * light) as u8;
        let mut g = (40.0 * light) as u8;
        let mut b = (40.0 * light) as u8;

        let rim = smoothstep(0.0, 2.0, d);
        r = ((r as f32 * (1.0 - 0.25 * rim)) as u8).min(255);
        g = ((g as f32 * (1.0 - 0.25 * rim)) as u8).min(255);
        b = ((b as f32 * (1.0 - 0.25 * rim)) as u8).min(255);

        let hl = ((p.0 - w * 0.38).powi(2) + (p.1 - h * 0.42).powi(2)).sqrt() / (w * 0.08);
        let spec = (1.0 - hl).clamp(0.0, 1.0);
        r = r.saturating_add((60.0 * spec * a) as u8);
        g = g.saturating_add((40.0 * spec * a) as u8);
        b = b.saturating_add((40.0 * spec * a) as u8);

        let stem_half = (w * 0.02, h * 0.10);
        let stem_center = (c.0, c.1 - r_apple - h * 0.04);
        let ds = sdf_rounded_rect(p, stem_center, stem_half, w * 0.01);
        let as_ = smoothstep(1.0, -1.0, ds);
        let sr = (90.0) as u8;
        let sg = (60.0) as u8;
        let sb = (30.0) as u8;

        let leaf_center = (c.0 + w * 0.11, c.1 - r_apple + h * 0.03);
        let dl = sdf_circle(p, leaf_center, w * 0.08);
        let al = smoothstep(1.0, -1.0, dl);
        let lr = (40.0) as u8;
        let lg = (160.0) as u8;
        let lb = (60.0) as u8;

        let mut out = over(
            [r, g, b, (255.0 * a) as u8],
            [sr, sg, sb, (200.0 * as_) as u8],
        );
        out = over(out, [lr, lg, lb, (220.0 * al) as u8]);
        out
    })
}

fn over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let da = dst[3] as f32 / 255.0;
    let sa = src[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);

    let blend = |dc: u8, sc: u8| -> u8 {
        let dc = dc as f32 / 255.0;
        let sc = sc as f32 / 255.0;
        let out = (sc * sa + dc * da * (1.0 - sa)) / out_a.max(1e-6);
        (out * 255.0).round() as u8
    };

    if out_a <= 0.0 {
        [0, 0, 0, 0]
    } else {
        [
            blend(dst[0], src[0]),
            blend(dst[1], src[1]),
            blend(dst[2], src[2]),
            (out_a * 255.0).round() as u8,
        ]
    }
}
