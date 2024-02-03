use std::f64::consts;

use crate::{Position, Rectangle, Size};

#[derive(Clone, Copy)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White,
}

#[derive(Clone, Copy)]
pub enum WriteMode {
    Copy,
    Xor,
    Or,
    And,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineStyle {
    Solid,
    Dotted,
    Center,
    Dashed,
    User,
}

impl LineStyle {
    const LINE_STYLE_BITS: [u32; 5] = [0xFFFF, 0xCCCC, 0xF878, 0xF8F8, 0x0000];

    pub fn from(line_style: u8) -> LineStyle {
        match line_style {
            1 => LineStyle::Dotted,
            2 => LineStyle::Center,
            3 => LineStyle::Dashed,
            4 => LineStyle::User,
            _ => LineStyle::Solid,
        }
    }

    pub fn get_line_style_bits(&self) -> Vec<bool> {
        let offset = match self {
            LineStyle::Solid => 0,
            LineStyle::Dotted => 1,
            LineStyle::Center => 2,
            LineStyle::Dashed => 3,
            LineStyle::User => 4,
        };

        let mut res = Vec::new();
        for i in 0..16 {
            res.push(LineStyle::LINE_STYLE_BITS[offset] & (1 << i) != 0);
        }
        res
    }
}

#[derive(Clone, Copy)]
pub enum FillStyle {
    Empty,
    Solid,
    Line,
    LtSlash,
    Slash,
    BkSlash,
    LtBkSlash,
    Hatch,
    XHatch,
    Interleave,
    WideDot,
    CloseDot,
    User,
}

impl FillStyle {
    const FILL_STYLES: [[u8; 8]; 13] = [
        // Empty
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // Solid
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        // Line
        [0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // LtSlash
        [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80],
        // Slash
        [0xE0, 0xC1, 0x83, 0x07, 0x0E, 0x1C, 0x38, 0x70],
        // BkSlash
        [0xF0, 0x78, 0x3C, 0x1E, 0x0F, 0x87, 0xC3, 0xE1],
        // LtBkSlash
        [0xA5, 0xD2, 0x69, 0xB4, 0x5A, 0x2D, 0x96, 0x4B],
        // Hatch
        [0xFF, 0x88, 0x88, 0x88, 0xFF, 0x88, 0x88, 0x88],
        // XHatch
        [0x81, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x81],
        // Interleave
        [0xCC, 0x33, 0xCC, 0x33, 0xCC, 0x33, 0xCC, 0x33],
        // WideDot
        [0x80, 0x00, 0x08, 0x00, 0x80, 0x00, 0x08, 0x00],
        // CloseDot
        [0x88, 0x00, 0x22, 0x00, 0x88, 0x00, 0x22, 0x00],
        // User
        [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55],
    ];

    pub fn from(fill_style: u8) -> FillStyle {
        match fill_style {
            0 => FillStyle::Empty,
            1 => FillStyle::Solid,
            2 => FillStyle::Line,
            3 => FillStyle::LtSlash,
            4 => FillStyle::Slash,
            5 => FillStyle::BkSlash,
            6 => FillStyle::LtBkSlash,
            7 => FillStyle::Hatch,
            8 => FillStyle::XHatch,
            9 => FillStyle::Interleave,
            10 => FillStyle::WideDot,
            11 => FillStyle::CloseDot,
            12 => FillStyle::User,
            _ => FillStyle::Empty,
        }
    }

    pub fn get_fill_style(&self) -> &'static [u8; 8] {
        &FillStyle::FILL_STYLES[*self as usize]
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub enum FontType {
    Default,
    Triplex,
    Small,
    SansSerif,
    Gothic,
    Script,
    Simplex,
    TriplexScript,
    Complex,
    European,
    BoldOutline,
    User,
}

const SCREEN_SIZE: Size = Size { width: 640, height: 350 };

const DEFAULT_USER_PATTERN: [u8; 8] = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];
const RAD2DEG: f64 = 180.0 / consts::PI;
const DEG2RAD: f64 = consts::PI / 180.0;
const ASPECT: f64 = 350.0 / 480.0 * 1.06; //0.772; //7.0/9.0; //350.0 / 480.0 * 1.066666;

pub struct Bgi {
    color: u8,
    bkcolor: u8,
    write_mode: WriteMode,
    line_style: LineStyle,
    fill_style: FillStyle,
    fill_color: u8,
    direction: Direction,
    font: FontType,
    text_height: i32,
    text_width: i32,
    window: Size,
    viewport: Rectangle,
    palette: [Color; 16],
    line_thickness: i32,
    pub screen: Vec<u8>,
    line_style_bits: Vec<bool>,
    current_pos: Position,
}

mod bezier_handler {
    const ST_ARR: [f64; 4] = [1.0, 3.0, 3.0, 1.0];

    pub fn first(n: i32, v: f64) -> f64 {
        match n {
            1 => v,
            2 => v * v,
            3 => v * v * v,
            _ => 1.0,
        }
    }

    pub fn second(n: i32, v: f64) -> f64 {
        match n {
            2 => (1.0 - v).log10().exp(),
            1 => (2.0 * (1.0 - v).log10()).exp(),
            0 => (3.0 * (1.0 - v).log10()).exp(),
            _ => 1.0,
        }
    }

    pub fn bezier(v: f64, n: i32) -> f64 {
        ST_ARR[n as usize] * first(n, v) * second(n, v)
    }
}

#[derive(Default, Clone)]
struct LineInfo {
    x1: i32,
    x2: i32,
    y: i32,
}

#[derive(Default, Clone)]
struct FillLineInfo {
    dir: i32,
    x1: i32,
    x2: i32,
    y: i32,
}

impl FillLineInfo {
    pub fn new(li: &LineInfo, dir: i32) -> Self {
        Self {
            dir,
            x1: li.x1,
            x2: li.x2,
            y: li.y,
        }
    }

    pub fn from(x1: i32, x2: i32, y: i32, dir: i32) -> Self {
        Self { dir, x1, x2, y }
    }
}

impl Bgi {
    pub fn new() -> Bgi {
        Bgi {
            color: 7,
            bkcolor: 0,
            write_mode: WriteMode::Copy,
            line_style: LineStyle::Solid,
            line_style_bits: LineStyle::Solid.get_line_style_bits(),
            fill_style: FillStyle::Solid,
            fill_color: 15,
            direction: Direction::Horizontal,
            font: FontType::Default,
            text_height: 8,
            text_width: 8,
            window: SCREEN_SIZE,
            viewport: Rectangle::from(0, 0, SCREEN_SIZE.width, SCREEN_SIZE.height),
            palette: [
                Color::Black,
                Color::Blue,
                Color::Green,
                Color::Cyan,
                Color::Red,
                Color::Magenta,
                Color::Brown,
                Color::LightGray,
                Color::DarkGray,
                Color::LightBlue,
                Color::LightGreen,
                Color::LightCyan,
                Color::LightRed,
                Color::LightMagenta,
                Color::Yellow,
                Color::White,
            ],
            line_thickness: 1,
            screen: vec![0; (SCREEN_SIZE.width * SCREEN_SIZE.height) as usize],
            current_pos: Position::new(0, 0),
        }
    }

    pub fn get_color(&self) -> u8 {
        self.color
    }

    pub fn set_color(&mut self, c: u8) -> u8 {
        let old = self.color;
        self.color = c % 16;
        old
    }

    pub fn get_bk_color(&self) -> u8 {
        self.bkcolor
    }

    pub fn set_bk_color(&mut self, c: u8) -> u8 {
        let old = self.color;
        self.bkcolor = c % 16;
        old
    }

    pub fn get_fill_style(&self) -> FillStyle {
        self.fill_style
    }

    pub fn set_fill_style(&mut self, style: FillStyle) -> FillStyle {
        let old = self.fill_style;
        self.fill_style = style;
        old
    }

    pub fn get_fill_color(&self) -> u8 {
        self.fill_color
    }

    pub fn set_fill_color(&mut self, color: u8) -> u8 {
        let old = self.fill_color;
        self.fill_color = color;
        old
    }

    pub fn get_line_style(&self) -> LineStyle {
        self.line_style
    }

    pub fn set_line_style(&mut self, style: LineStyle) -> LineStyle {
        let old = self.line_style;
        self.line_style = style;
        self.line_style_bits = style.get_line_style_bits();
        old
    }

    pub fn get_line_thickness(&self) -> i32 {
        self.line_thickness
    }

    pub fn set_line_thickness(&mut self, thickness: i32) {
        self.line_thickness = thickness;
    }

    pub fn set_line_pattern(&mut self, pattern: i32) {
        let mut res = Vec::new();
        for i in 0..16 {
            res.push(pattern & (1 << i) != 0);
        }
        self.line_style_bits = res;
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> u8 {
        self.screen[(y * self.window.width + x) as usize]
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: u8) {
        let pos = (y * self.window.width + x) as usize;
        match self.write_mode {
            WriteMode::Copy => {
                self.screen[pos] = color;
            }
            WriteMode::Xor => {
                self.screen[pos] ^= color;
            }
            WriteMode::Or => {
                self.screen[pos] |= color;
            }
            WriteMode::And => {
                self.screen[pos] &= color;
            }
            WriteMode::Not => {
                self.screen[pos] &= !color;
            }
        }
    }

    pub fn get_write_mode(&self) -> WriteMode {
        self.write_mode
    }

    pub fn set_write_mode(&mut self, mode: WriteMode) -> WriteMode {
        let old = self.write_mode;
        self.write_mode = mode;
        old
    }

    fn fill_x(&mut self, y: i32, startx: i32, count: i32, offset: &mut i32) {
        let mut start_y = y - self.line_thickness / 2;
        let mut end_y = start_y + self.line_thickness - 1;
        let mut end_x = startx + count;
        if count > 0 {
            end_x -= 1;
        } else {
            end_x += 1;
            *offset -= count;
        }

        if start_y < 0 {
            start_y = 0;
        }

        end_y = end_y.min(self.viewport.bottom() - 1);

        let inc = if count >= 0 { 1 } else { -1 };
        let mut startx = startx;
        if startx > end_x {
            std::mem::swap(&mut startx, &mut end_x);
        }

        if startx >= self.viewport.right() {
            return;
        }

        if startx < 0 {
            startx = 0;
        }

        end_x = end_x.min(self.viewport.right() - 1);

        for x in startx..=end_x {
            if self.line_style_bits[offset.abs() as usize % self.line_style_bits.len()] {
                for cy in start_y..=end_y {
                    self.put_pixel(x, cy, self.color);
                }
            }
            *offset += inc;
        }
        if count < 0 {
            *offset -= count;
        }
    }

    pub fn fill_y(&mut self, x: i32, start_y: i32, count: i32, offset: &mut i32) {
        let mut start_x = x - self.line_thickness / 2;
        let mut end_x = start_x + self.line_thickness - 1;
        let mut end_y = start_y + count;
        if count > 0 {
            end_y -= 1;
        } else {
            end_y += 1;
            *offset -= count;
        }

        if start_x < 0 {
            start_x = 0;
        }

        end_x = end_x.min(self.viewport.right() - 1);
        let mut start_y = start_y;
        if start_y > end_y {
            std::mem::swap(&mut start_y, &mut end_y);
        }

        if start_y >= self.viewport.bottom() {
            return;
        }

        if start_y < 0 {
            start_y = 0;
        }

        end_y = end_y.min(self.viewport.bottom() - 1);

        for y in start_y..=end_y {
            if self.line_style_bits[offset.abs() as usize % self.line_style_bits.len()] {
                for cx in start_x..=end_x {
                    self.put_pixel(cx, y, self.color);
                }
            }
            *offset += 1;
        }
        if count < 0 {
            *offset += count;
        }
    }

    pub fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let ly_delta = (y2 - y1).abs();
        let lx_delta = (x2 - x1).abs();
        let mut offset = 0;

        if lx_delta == 0 {
            self.fill_y(x1, y1.min(y2), ly_delta + 1, &mut offset);
        } else if ly_delta == 0 {
            self.fill_x(y1, x1.min(x2), lx_delta + 1, &mut offset);
        } else if lx_delta >= ly_delta {
            let l_advance = 1;
            let (mut pos, l_step) = if y1 < y2 {
                (Position::new(x1, y1), if x1 > x2 { -1 } else { 1 })
            } else {
                (Position::new(x2, y2), if x2 > x1 { -1 } else { 1 })
            };

            let l_whole_step = (lx_delta / ly_delta) * l_step;
            let mut l_adj_up = lx_delta % ly_delta;
            let l_adj_down = ly_delta * 2;
            let mut l_error = l_adj_up - l_adj_down;
            l_adj_up *= 2;

            let mut l_start_length = (l_whole_step / 2) + l_step;
            let l_end_length = l_start_length;
            if (l_adj_up == 0) && ((l_whole_step & 0x01) == 0) {
                l_start_length -= l_step;
            }

            if (l_whole_step & 0x01) != 0 {
                l_error += ly_delta;
            }

            self.fill_x(pos.y, pos.x, l_start_length, &mut offset);
            pos.x += l_start_length;
            pos.y += l_advance;
            for _ in 0..(ly_delta - 1) {
                let mut l_run_length = l_whole_step;
                l_error += l_adj_up;
                if l_error > 0 {
                    l_run_length += l_step;
                    l_error -= l_adj_down;
                }
                self.fill_x(pos.y, pos.x, l_run_length, &mut offset);
                pos.x += l_run_length;
                pos.y += l_advance;
            }
            self.fill_x(pos.y, pos.x, l_end_length, &mut offset);
        } else if lx_delta < ly_delta {
            let (mut pos, l_advance) = if y1 < y2 {
                (Position::new(x1, y1), if x1 > x2 { -1 } else { 1 })
            } else {
                (Position::new(x2, y2), if x2 > x1 { -1 } else { 1 })
            };

            let l_whole_step = ly_delta / lx_delta;
            let mut l_adj_up = ly_delta % lx_delta;
            let l_adj_down = lx_delta * 2;
            let mut l_error = l_adj_up - l_adj_down;
            l_adj_up *= 2;
            let mut l_start_length = (l_whole_step / 2) + 1;
            let l_end_length = l_start_length;
            if (l_adj_up == 0) && ((l_whole_step & 0x01) == 0) {
                l_start_length -= 1;
            }
            if (l_whole_step & 0x01) != 0 {
                l_error += lx_delta;
            }

            self.fill_y(pos.x, pos.y, l_start_length, &mut offset);
            pos.y += l_start_length;
            pos.x += l_advance;

            for _ in 0..(lx_delta - 1) {
                let mut l_run_length = l_whole_step;
                l_error += l_adj_up;
                if l_error > 0 {
                    l_run_length += 1;
                    l_error -= l_adj_down;
                }

                self.fill_y(pos.x, pos.y, l_run_length, &mut offset);
                pos.y += l_run_length;
                pos.x += l_advance;
            }
            self.fill_y(pos.x, pos.y, l_end_length, &mut offset);
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.current_pos = Position::new(x, y);
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        self.line(self.current_pos.x, self.current_pos.y, x, y);
        self.move_to(x, y);
    }

    pub fn line_rel(&mut self, dx: i32, dy: i32) {
        let x = self.current_pos.x + dx;
        let y = self.current_pos.y + dy;
        self.line(self.current_pos.x, self.current_pos.y, x, y);
        self.move_to(x, y);
    }

    fn find_line(&self, x: i32, y: i32, border: u8) -> Option<LineInfo> {
        // find end pixel
        let mut endx = self.viewport.get_width();
        let mut pos = y * self.window.width + x;
        for ex in x..self.viewport.get_width() {
            let col = self.screen[pos as usize];
            pos += 1;
            if col == border {
                endx = ex;
                break;
            }
        }
        endx -= 1;

        // find beginning pixel
        let mut pos = y * self.window.width + x - 1;
        let mut startx = -1;
        for sx in (0..x).rev() {
            let col = self.screen[pos as usize];
            pos -= 1;
            if col == border {
                startx = sx;
                break;
            }
        }
        startx += 1;

        // a weird condition for solid fills and the sides of the screen
        if (startx == 0 || endx == self.window.width - 1) && (endx == startx) {
            return None;
        }

        Some(LineInfo { x1: startx, x2: endx, y })
    }

    pub fn rectangle(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.line(left, top, right, top);
        self.line(left, bottom, right, bottom);
        self.line(right, top, right, bottom);
        self.line(left, top, left, bottom);
    }

    fn symmetry_scan(
        &mut self,
        x: i32,
        y: i32,
        start_angle: i32,
        end_angle: i32,
        xoffset: i32,
        yoffset: i32,
        angle: i32,
        horizontal: bool,
        rows: &mut Vec<Vec<i32>>,
    ) {
        if self.line_thickness == 1 {
            if in_angle(angle, start_angle, end_angle) {
                add_scan_row(rows, x + xoffset, y - yoffset);
            }
            if in_angle(180 - angle, start_angle, end_angle) {
                add_scan_row(rows, x - xoffset, y - yoffset);
            }
            if in_angle(180 + angle, start_angle, end_angle) {
                add_scan_row(rows, x - xoffset, y + yoffset);
            }
            if in_angle(360 - angle, start_angle, end_angle) {
                add_scan_row(rows, x + xoffset, y + yoffset);
            }
        } else {
            let offset = self.line_thickness / 2;
            if horizontal {
                if in_angle(angle, start_angle, end_angle) {
                    add_scan_horizontal(rows, x + xoffset - offset, y - yoffset, self.line_thickness);
                }
                if in_angle(180 - angle, start_angle, end_angle) {
                    add_scan_horizontal(rows, x - xoffset - offset, y - yoffset, self.line_thickness);
                }
                if in_angle(180 + angle, start_angle, end_angle) {
                    add_scan_horizontal(rows, x - xoffset - offset, y + yoffset, self.line_thickness);
                }
                if in_angle(360 - angle, start_angle, end_angle) {
                    add_scan_horizontal(rows, x + xoffset - offset, y + yoffset, self.line_thickness);
                }
            } else {
                if in_angle(angle, start_angle, end_angle) {
                    add_scan_vertical(rows, x + xoffset, y - yoffset - offset, self.line_thickness);
                }
                if in_angle(180 - angle, start_angle, end_angle) {
                    add_scan_vertical(rows, x - xoffset, y - yoffset - offset, self.line_thickness);
                }
                if in_angle(180 + angle, start_angle, end_angle) {
                    add_scan_vertical(rows, x - xoffset, y + yoffset - offset, self.line_thickness);
                }
                if in_angle(360 - angle, start_angle, end_angle) {
                    add_scan_vertical(rows, x + xoffset, y + yoffset - offset, self.line_thickness);
                }
            }
        }
    }

    fn symmetry_vertical(&mut self, x: i32, y: i32, xoffset: i32, yoffset: i32) {
        let ls = -self.line_thickness / 2;
        let le = (self.line_thickness + 1) / 2;
        for i in ls..le {
            self.put_pixel(x + xoffset, y + yoffset - i, self.color);
            self.put_pixel(x - xoffset, y + yoffset - i, self.color);
            self.put_pixel(x - xoffset, y - yoffset + i - ls, self.color);
            self.put_pixel(x + xoffset, y - yoffset + i - ls, self.color);
        }
    }

    fn symmetry_horizontal(&mut self, x: i32, y: i32, xoffset: i32, yoffset: i32) {
        let ls = -self.line_thickness / 2;
        //int le = (lineThickness + 1) / 2;
        for i in 0..self.line_thickness {
            self.put_pixel(x + xoffset - i, y + yoffset, self.color);
            self.put_pixel(x - xoffset - i - ls, y + yoffset, self.color);
            self.put_pixel(x - xoffset - i - ls, y - yoffset, self.color);
            self.put_pixel(x + xoffset - i, y - yoffset, self.color);
        }
    }

    fn symmetry_line(&mut self, x: i32, y: i32, xoffset: i32, yoffset: i32, length: i32) {
        self.line(x + xoffset, y + yoffset, x + xoffset + length, y + yoffset);
        self.line(x - xoffset, y + yoffset, x - xoffset - length, y + yoffset);
        self.line(x - xoffset, y - yoffset, x - xoffset - length, y - yoffset);
        self.line(x + xoffset, y - yoffset, x + xoffset + length, y - yoffset);
    }

    pub fn flood_fill(&mut self, x: i32, y: i32, border: u8) {
        let mut fillLines = vec![Vec::new(); self.viewport.get_height() as usize + 1];

        let mut pointStack = Vec::new();

        if !self.viewport.contains(x, y) {
            return;
        }

        if self.screen[(y * self.window.width + x) as usize] != border {
            let li = self.find_line(x, y, border);
            if let Some(li) = li {
                pointStack.push(FillLineInfo::new(&li, 1));
                pointStack.push(FillLineInfo::new(&li, -1));

                fillLines[li.y as usize].push(li);
                //this.Bar(li.x1, li.y, li.x2, li.y);

                while !pointStack.is_empty() {
                    let fli = pointStack.pop().unwrap();

                    let cury = fli.y + fli.dir;
                    if cury < self.viewport.bottom() && cury >= self.viewport.top() {
                        let ypos = cury * self.window.width;
                        for cx in fli.x1..=fli.x2 {
                            if self.screen[(ypos + cx) as usize] == border {
                                continue; // it's a border color, so don't scan any more this direction
                            }

                            //if (AlreadyDrawn(fillLines, cx, cury))
                            //  continue; // already been here

                            let li = self.find_line(cx, cury, border); // find the borders on this line
                            if let Some(li) = li {
                                // let cx = li.x2;
                                pointStack.push(FillLineInfo::new(&li, fli.dir));
                                if self.fill_color != 0 {
                                    // bgi doesn't go backwards when filling black!  why?  dunno.  it just does.
                                    // if we go out of current line's bounds, check the opposite dir for those
                                    if li.x2 > fli.x2 {
                                        pointStack.push(FillLineInfo::from(li.y, fli.x2 + 1, li.x2, -fli.dir));
                                    }
                                    if li.x1 < fli.x1 {
                                        pointStack.push(FillLineInfo::from(li.y, li.x1, fli.x1 - 1, -fli.dir));
                                    }
                                }

                                fillLines[li.y as usize].push(li);
                            }
                        }
                    }
                }
            }
        }
        for i in 0..fillLines.len() {
            for cli in &fillLines[i] {
                self.bar(cli.x1, cli.y, cli.x2, cli.y);
            }
        }
    }

    pub fn bar(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.bar_rect(Rectangle::from(left, top, right - left + 1, bottom - top + 1));
    }

    pub fn bar_rect(&mut self, rect: Rectangle) {
        let rect = rect.intersect(&self.viewport);
        if rect.get_width() == 0 || rect.get_height() == 0 {
            return;
        }

        let right = rect.right();
        let bottom = rect.bottom();
        let mut ystart = rect.top() * self.window.width + rect.left();
        if matches!(self.fill_style, FillStyle::Solid) {
            for y in rect.top()..bottom {
                let mut xstart = ystart;
                for _ in rect.left()..right {
                    self.screen[xstart as usize] = self.fill_color;
                    xstart += 1;
                }
                ystart += self.window.width;
            }
        } else {
            let pattern = self.fill_style.get_fill_style();
            let mut ypat = rect.top() % 8;
            for y in rect.top()..bottom {
                let mut xstart = ystart as usize;
                let mut xpatmask = (128 >> (rect.left() % 8)) as u8;
                let pat = pattern[ypat as usize];
                for x in rect.left()..right {
                    self.screen[xstart] = if (pat & xpatmask) != 0 { self.fill_color } else { self.bkcolor };
                    xstart += 1;
                    xpatmask >>= 1;
                    if xpatmask == 0 {
                        xpatmask = 128;
                    }
                }
                ypat = (ypat + 1) % 8;
                ystart += self.window.width;
            }
        }
    }

    pub fn draw_bezier(&mut self, count: i32, points: &[Position], segments: i32) {
        let mut x1 = points[0].x;
        let mut y1 = points[0].y;
        let mut v = 1.0;
        loop {
            let mut x3 = 0.0;
            let mut y3 = 0.0;
            let br = v / segments as f64;
            for i in 0..4usize {
                let ar = bezier_handler::bezier(br, i as i32);
                x3 += points[i].x as f64 * ar;
                y3 += points[i].y as f64 * ar;
            }
            let x2 = (x3).round() as i32;
            let y2 = (y3).round() as i32;
            self.line(x1, y1, x2, y2);
            x1 = x2;
            y1 = y2;
            v += 1.0;
            if v >= segments as f64 {
                break;
            }
        }

        self.line(x1, y1, points[count as usize - 1].x, points[count as usize - 1].y);
    }

    pub fn arc(&mut self, x: i32, y: i32, start_angle: i32, end_angle: i32, radius: i32) {
        self.ellipse(x, y, start_angle, end_angle, radius, (radius as f64 * ASPECT) as i32);
    }

    pub fn scan_ellipse(&mut self, x: i32, y: i32, mut start_angle: i32, mut end_angle: i32, radiusx: i32, radiusy: i32, rows: &mut Vec<Vec<i32>>) {
        // check if valid angles
        if start_angle > end_angle {
            std::mem::swap(&mut start_angle, &mut end_angle);
        }

        let radiusx = radiusx.max(1);
        let radiusy = radiusy.max(1);

        let diameterx = radiusx * 2;
        let diametery = radiusy * 2;
        let b1 = diametery & 1;
        let mut stopx = 4 * (1 - diameterx) * diametery * diametery;
        let mut stopy = 4 * (b1 + 1) * diameterx * diameterx; // error increment
        let mut err = stopx + stopy + b1 * diameterx * diameterx; // error of 1 step

        let mut xoffset = radiusx;
        let mut yoffset = 0;
        let incx = 8 * diameterx * diameterx;
        let incy = 8 * diametery * diametery;

        let aspect = radiusx as f64 / radiusy as f64;

        // calculate horizontal fill angle
        let horizontal_angle = if radiusx < radiusy { 90.0 - (45.0 * aspect) } else { 45.0 / aspect };

        loop {
            let e2 = 2 * err;
            let angle = (yoffset as f64 * aspect / xoffset as f64).atan() * RAD2DEG;

            self.symmetry_scan(
                x,
                y,
                start_angle,
                end_angle,
                xoffset,
                yoffset,
                angle.round() as i32,
                angle <= horizontal_angle,
                rows,
            );
            if (angle - horizontal_angle).abs() < 1.0 {
                self.symmetry_scan(
                    x,
                    y,
                    start_angle,
                    end_angle,
                    xoffset,
                    yoffset,
                    angle.round() as i32,
                    !(angle <= horizontal_angle),
                    rows,
                );
            }

            if e2 <= stopy {
                yoffset += 1;
                stopy += incx;
                err += stopy;
            }
            if e2 >= stopx {
                xoffset -= 1;
                stopx += incy;
                err += stopx;
            }
            if xoffset < 0 {
                break;
            }
        }

        xoffset += 1;
        while yoffset < radiusy {
            let angle = (yoffset as f64 * aspect / xoffset as f64).atan() * RAD2DEG;
            self.symmetry_scan(
                x,
                y,
                start_angle,
                end_angle,
                xoffset,
                yoffset,
                angle.round() as i32,
                angle <= horizontal_angle,
                rows,
            );
            if angle == horizontal_angle {
                self.symmetry_scan(
                    x,
                    y,
                    start_angle,
                    end_angle,
                    xoffset,
                    yoffset,
                    angle.round() as i32,
                    !(angle <= horizontal_angle),
                    rows,
                );
            }
            yoffset += 1;
        }
    }

    pub fn fill_scan(&mut self, rows: &mut Vec<Vec<i32>>) {
        for y in 0..rows.len() - 2 {
            let row = &mut rows[y + 1];
            if !row.is_empty() {
                row.sort_unstable();
                self.bar(row[0], y as i32, row[row.len() - 1], y as i32);
            }
        }
    }

    pub fn draw_scan(&mut self, rows: &mut Vec<Vec<i32>>) {
        for i in 0..rows.len() {
            let row = &mut rows[i];
            let y = i - 1;
            row.dedup();
            for x in row {
                self.put_pixel(*x, y as i32, self.color);
            }
        }
    }

    pub fn outline_scan(&mut self, rows: &mut Vec<Vec<i32>>) {
        let old_line_style = self.get_line_style();
        if !matches!(old_line_style, LineStyle::Solid) {
            self.set_line_style(LineStyle::Solid);
        }

        let mut lastminx = 0;
        let mut lastmaxx = 0;
        let mut first = true;
        let rows_len = rows.len();
        for i in 0..rows_len {
            rows[i].sort_unstable();
            if rows[i].len() > 2 {
                let a = (rows[i]).len() - 2;
                rows[i].drain(1..a);
            }
            let y = i - 1;
            if !rows[i].is_empty() {
                let minx = (&mut rows[i])[0];
                let a = rows[i].len() - 1;
                let maxx = (&mut rows[i])[a];
                let mut hasnext = i < rows_len - 1;
                let mut last = false;
                let mut nextminx = 0;
                let mut nextmaxx = 0;
                //let mut nextrow = if hasnext { Some(&rows[i + 1]) } else { None };

                if hasnext && !rows[i + 1].is_empty() {
                    nextminx = rows[i + 1][0];
                    nextmaxx = rows[i + 1][rows[i + 1].len() - 1];
                } else {
                    last = true;
                    hasnext = false;
                }

                if first {
                    if hasnext {
                        if nextmaxx > nextminx {
                            self.line(nextminx + 1, y as i32, nextmaxx - 1, y as i32);
                        } else {
                            self.line(nextminx, y as i32, nextmaxx, y as i32);
                        }
                    }
                    first = false;
                } else if last {
                    if lastmaxx > lastminx {
                        self.line(lastminx + 1, y as i32, lastmaxx - 1, y as i32);
                    } else {
                        self.line(lastminx, y as i32, lastmaxx, y as i32);
                    }
                } else {
                    if minx >= lastminx {
                        let mn_x = if minx > lastminx { lastminx + 1 } else { lastminx };
                        self.line(mn_x, y as i32, minx, y as i32);
                    }

                    if rows[i].len() > 1 && maxx <= lastmaxx {
                        let mx_x = if maxx < lastmaxx { lastmaxx - 1 } else { lastmaxx };
                        self.line(mx_x, y as i32, maxx, y as i32);
                    }
                }
                if hasnext {
                    if minx < lastminx && minx >= nextminx {
                        let mn_x = if minx > nextminx { nextminx + 1 } else { nextminx };
                        self.line(mn_x, y as i32, minx, y as i32);
                    }

                    if rows[i].len() > 1 && hasnext && rows[i + 1].len() > 1 && maxx > lastmaxx && maxx <= nextmaxx {
                        let mx_x = if maxx < nextmaxx { nextmaxx - 1 } else { nextmaxx };
                        self.line(mx_x, y as i32, maxx, y as i32);
                    }
                }
                lastminx = minx;
                lastmaxx = maxx;
            }
        }

        if !matches!(old_line_style, LineStyle::Solid) {
            self.set_line_style(old_line_style);
        }
    }

    pub fn symmetry_fill(&mut self, x: i32, y: i32, xoffset: i32, yoffset: i32) {
        self.bar(x - xoffset, y - yoffset, x + xoffset, y - yoffset);
        self.bar(x - xoffset, y + yoffset, x + xoffset, y + yoffset);
    }

    pub fn circle(&mut self, x: i32, y: i32, radius: i32) {
        let ry = (radius as f64 * ASPECT) as i32;
        let rx = radius;
        self.ellipse(x, y, 0, 360, rx, ry);
    }

    pub fn ellipse(&mut self, x: i32, y: i32, start_angle: i32, end_angle: i32, radius_x: i32, radius_y: i32) {
        let mut rows = create_scan_rows();
        if start_angle > end_angle {
            self.scan_ellipse(x, y, 0, end_angle, radius_x, radius_y, &mut rows);
            self.scan_ellipse(x, y, start_angle, 360, radius_x, radius_y, &mut rows);
        } else {
            self.scan_ellipse(x, y, start_angle, end_angle, radius_x, radius_y, &mut rows);
        }
        self.draw_scan(&mut rows);
    }

    pub fn fill_ellipse(&mut self, x: i32, y: i32, start_angle: i32, end_angle: i32, radius_x: i32, radius_y: i32) {
        let mut rows = create_scan_rows();
        self.scan_ellipse(x, y, start_angle, end_angle, radius_x, radius_y, &mut rows);
        self.fill_scan(&mut rows);
        self.draw_scan(&mut rows);
    }

    pub fn clear_device(&mut self) {
        self.bar(0, 0, self.window.width, self.window.height);
        self.move_to(0, 0);
    }

    pub fn sector(&mut self, x: i32, y: i32, start_angle: i32, end_angle: i32, radiusx: i32, radiusy: i32) {
        let center = Position::new(x, y);
        let mut rows = create_scan_rows();
        let start_point = center + get_angle_size(start_angle, radiusx, radiusy);
        let end_point = center + get_angle_size(end_angle, radiusx, radiusy);

        let oldthickness = self.get_line_thickness();
        if !matches!(self.line_style, LineStyle::Solid) {
            self.set_line_thickness(1);
        }

        self.scan_ellipse(x, y, start_angle, end_angle, radiusx, radiusy, &mut rows);

        scan_line(center, start_point, &mut rows, true);
        scan_line(center, end_point, &mut rows, true);

        if !matches!(self.fill_style, FillStyle::Empty) {
            self.fill_scan(&mut rows);
        }

        if matches!(self.line_style, LineStyle::Solid) {
            rows = create_scan_rows(); // ugh, twice, really?!
            self.scan_ellipse(x, y, start_angle, end_angle, radiusx, radiusy, &mut rows);
            self.draw_scan(&mut rows);
        }

        if !matches!(self.line_style, LineStyle::Solid) {
            self.set_line_thickness(oldthickness);
        }

        self.line(center.x, center.y, start_point.x, start_point.y);
        self.line(center.x, center.y, end_point.x, end_point.y);
    }

    pub fn pie_slice(&mut self, x: i32, y: i32, start_angle: i32, end_angle: i32, radius: i32) {
        self.sector(x, y, start_angle, end_angle, radius, (radius as f64 * ASPECT) as i32);
    }
    /*

    public void Bar3d(int left, int top, int right, int bottom, int depth, int topflag, IList<Rectangle> updates = null)
    {
        int temp;
        const double tan30 = 1.0 / 1.73205080756887729352;
        if (left > right)
        {
            temp = left;
            left = right;
            right = temp;
        }
        if (bottom < top)
        {
            temp = bottom;
            bottom = top;
            top = temp;
        }
        var drawUpdates = updates ?? new List<Rectangle>();
        Bar(left + lineThickness, top + lineThickness, right - lineThickness + 1, bottom - lineThickness + 1, drawUpdates);

        int dy = (int)(depth * tan30);
        var p = new Point[topflag != 0 ? 11 : 8];
        p[0].X = right;
        p[0].Y = bottom;
        p[1].X = right;
        p[1].Y = top;
        p[2].X = left;
        p[2].Y = top;
        p[3].X = left;
        p[3].Y = bottom;
        p[4].X = right;
        p[4].Y = bottom;
        p[5].X = right + depth;
        p[5].Y = bottom - dy;
        p[6].X = right + depth;
        p[6].Y = top - dy;
        p[7].X = right;
        p[7].Y = top;

        if (topflag != 0)
        {
            p[8].X = right + depth;
            p[8].Y = top - dy;
            p[9].X = left + depth;
            p[9].Y = top - dy;
            p[10].X = left;
            p[10].Y = top;
        }
        DrawPoly(p, drawUpdates);
        UpdateRegion(drawUpdates);
        if (updates == null)
            UpdateRegion(drawUpdates);
    }*/
}

impl Default for Bgi {
    fn default() -> Self {
        Self::new()
    }
}

fn scan_line(start: Position, end: Position, rows: &mut Vec<Vec<i32>>, full: bool) {
    let ydelta = (end.y - start.y).abs();

    if full || start.y < end.y {
        add_scan_row(rows, start.x, start.y);
    }
    if ydelta > 0 {
        let x_delta = if start.y > end.y { start.x - end.x } else { end.x - start.x };
        let min_x = if start.y > end.y { end.x } else { start.x };
        let mut pos_y = start.y.min(end.y);

        pos_y += 1;
        for count in 1..ydelta {
            let pos_x = (x_delta * count / ydelta) + min_x;

            if pos_y >= 0 && pos_y < rows.len() as i32 {
                add_scan_row(rows, pos_x, pos_y);
            }
            pos_y += 1;
        }
    }
    if full || end.y < start.y {
        add_scan_row(rows, end.x, end.y);
    }
}

fn scan_lines(start_index: i32, end_index: i32, rows: &mut Vec<Vec<i32>>, points: &[Position], full: bool) {
    scan_line(points[start_index as usize], points[end_index as usize], rows, full);
}

fn create_scan_rows() -> Vec<Vec<i32>> {
    vec![Vec::new(); 352]
}

fn add_scan_vertical(rows: &mut Vec<Vec<i32>>, x: i32, y: i32, count: i32) {
    for i in 0..count {
        add_scan_row(rows, x, y + i);
    }
}

fn add_scan_horizontal(rows: &mut Vec<Vec<i32>>, x: i32, y: i32, count: i32) {
    for i in 0..count {
        add_scan_row(rows, x + i, y);
    }
}

fn add_scan_row(rows: &mut Vec<Vec<i32>>, x: i32, y: i32) {
    if !(-1..=350).contains(&y) {
        return;
    }
    let y = y + 1;
    if rows.len() <= y as usize {
        rows.resize(y as usize + 1, Vec::new());
    }
    let row = &mut rows[y as usize];
    row.push(x);
}

fn in_angle(angle: i32, start_angle: i32, end_angle: i32) -> bool {
    angle >= start_angle && angle <= end_angle
}

pub fn arc_coords(angle: f64, rx: f64, ry: f64) -> Position {
    if rx == 0.0 || ry == 0.0 {
        return Position::new(0, 0);
    }

    let s = (angle * DEG2RAD).sin();
    let c = (angle * DEG2RAD).cos();
    if s.abs() < c.abs() {
        let tg = s / c;
        let xr = (rx * rx * ry * ry / (ry * ry + rx * rx * tg * tg)).sqrt();
        Position::new(if c >= 0.0 { xr } else { -xr } as i32, if s >= 0.0 { -xr * tg } else { xr * tg } as i32)
    } else {
        let ctg = c / s;
        let yr = (rx * rx * ry * ry / (rx * rx + ry * ry * ctg * ctg)).sqrt();
        Position::new(if c >= 0.0 { yr * ctg } else { -yr * ctg } as i32, if s >= 0.0 { -yr } else { yr } as i32)
    }
}

pub fn get_angle_size(angle: i32, radiusx: i32, radiusy: i32) -> Position {
    Position::new(
        ((angle as f64 * DEG2RAD).cos() * radiusx as f64).round() as i32,
        -((angle as f64 * DEG2RAD).sin() * radiusy as f64).round() as i32,
    )
}
