use egui::{Pos2, Rect};

#[derive(Clone, Copy)]
pub struct Grid {
    pub w: i32,
    pub h: i32,
    pub outer: Rect,
}
impl Grid {
    pub fn new(w: i32, h: i32, outer: Rect) -> Self {
        Self { w, h, outer }
    }
    pub fn cell_size(&self) -> (f32, f32) {
        (
            self.outer.width() / self.w as f32,
            self.outer.height() / self.h as f32,
        )
    }
    pub fn cell_rect(&self, x: i32, y: i32, pad: f32) -> Rect {
        let (cw, ch) = self.cell_size();
        let x0 = self.outer.left() + x as f32 * cw + pad;
        let y0 = self.outer.top() + y as f32 * ch + pad;
        let x1 = self.outer.left() + (x + 1) as f32 * cw - pad;
        let y1 = self.outer.top() + (y + 1) as f32 * ch - pad;
        Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1))
    }
}
