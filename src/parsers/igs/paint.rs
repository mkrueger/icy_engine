use super::{cmd::IgsCommands, CommandExecutor, IGS_VERSION};
use crate::{paint::get_line_points, BitFont, Buffer, CallbackAction, Caret, Color, EngineResult, Position, Size, ATARI, IGS_PALETTE, IGS_SYSTEM_PALETTE};

#[derive(Default)]
pub enum TerminalResolution {
    /// 320x200
    #[default]
    Low,
    /// 640x200
    Medium,
    /// 640x400  
    High,
}

impl TerminalResolution {
    pub fn resolution_id(&self) -> String {
        match self {
            TerminalResolution::Low => "0".to_string(),
            TerminalResolution::Medium => "1".to_string(),
            TerminalResolution::High => "2".to_string(),
        }
    }

    pub fn get_resolution(&self) -> Size {
        match self {
            TerminalResolution::Low => Size { width: 320, height: 200 },
            TerminalResolution::Medium => Size { width: 640, height: 200 },
            TerminalResolution::High => Size { width: 640, height: 400 },
        }
    }
}
pub enum TextEffects {
    Normal,
    Thickened,
    Ghosted,
    Skewed,
    Underlined,
    Outlined,
}

pub enum TextRotation {
    Right,
    Up,
    Down,
    Left,
    RightReverse,
}

pub enum PolymarkerType {
    Point,
    Plus,
    Star,
    Square,
    DiagonalCross,
    Diamond,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Solid,
    LongDash,
    DottedLine,
    DashDot,
    DashedLine,
    DashedDotDot,
    UserDefined,
}

pub enum DrawingMode {
    Replace,
    Transparent,
    Xor,
    ReverseTransparent,
}

pub struct DrawExecutor {
    igs_texture: Vec<u8>,
    terminal_resolution: TerminalResolution,
    x_scale: f32,
    y_scale: f32,

    cur_position: Position,
    pen_colors: Vec<Color>,
    polymarker_color: usize,
    line_color: usize,
    fill_color: usize,
    text_color: usize,

    text_effects: TextEffects,
    text_size: i32,
    text_rotation: TextRotation,

    polymaker_type: PolymarkerType,
    line_type: LineType,
    drawing_mode: DrawingMode,
    polymarker_size: usize,
    solidline_size: usize,
    user_defined_pattern_number: usize,

    font_8px: BitFont,
    screen_memory: Vec<Vec<u8>>,
}

impl Default for DrawExecutor {
    fn default() -> Self {
        let mut res = Self {
            igs_texture: Vec::new(),
            terminal_resolution: TerminalResolution::Low,
            x_scale: 2.0,
            y_scale: 2.0,
            pen_colors: IGS_SYSTEM_PALETTE.to_vec(),
            polymarker_color: 0,
            line_color: 0,
            fill_color: 0,
            text_color: 0,
            cur_position: Position::new(0, 0),
            text_effects: TextEffects::Normal,
            text_size: 8,
            text_rotation: TextRotation::Right,
            polymaker_type: PolymarkerType::Point,
            line_type: LineType::Solid,
            drawing_mode: DrawingMode::Replace,
            polymarker_size: 1,
            solidline_size: 1,
            user_defined_pattern_number: 1,
            font_8px: BitFont::from_bytes("ATARI", ATARI).unwrap(),
            screen_memory: Vec::new(),
        };

        res.set_resolution();
        res
    }
}

impl DrawExecutor {
    pub fn clear(&mut self) {
        let res = self.get_resolution();
        self.igs_texture = vec![0; res.width as usize * 4 * res.height as usize];
    }

    pub fn set_resolution(&mut self) {
        let res = self.get_resolution();
        self.igs_texture = vec![0; res.width as usize * 4 * res.height as usize];
    }

    pub fn reset_attributes(&mut self) {
        // TODO
    }

    fn draw_pixel(&mut self, p: Position, color: &Color) {
        let res = self.get_resolution();
        if p.x < 0 || p.y < 0 || p.x >= res.width || p.y >= res.height {
            return;
        }
        let offset = p.x as usize * 4 + p.y as usize * res.width as usize * 4;
        let (r, g, b) = color.get_rgb();
        self.igs_texture[offset] = r;
        self.igs_texture[offset + 1] = g;
        self.igs_texture[offset + 2] = b;
        self.igs_texture[offset + 3] = 255;
    }

    fn get_pixel(&self, p: Position) -> [u8; 4] {
        let offset = p.x as usize * 4 + p.y as usize * self.get_resolution().width as usize * 4;
        self.igs_texture[offset..offset + 4].try_into().unwrap()
    }

    fn translate_pos(&self, pt: Position) -> Position {
        Position::new((pt.x as f32 * self.x_scale) as i32, (pt.y as f32 * self.y_scale) as i32)
    }

    fn flood_fill(&mut self, pos: Position) {
        let pos = self.translate_pos(pos);
        let col = self.pen_colors[self.fill_color].clone();
        let color = self.get_pixel(pos);
        println!("fill {:?} {:?} replace color:{:?}", pos, col.get_rgb(), color);
        self.fill(pos, color);
    }

    fn fill(&mut self, p: Position, color: [u8; 4]) {
        if self.get_pixel(p) != color || p.x < 0 || p.y < 0 || p.x >= self.get_resolution().width || p.y >= self.get_resolution().height {
            return;
        }
        let col = self.pen_colors[self.fill_color].clone();
        self.draw_pixel(p, &col);
        self.fill(Position::new(p.x - 1, p.y), color);
        self.fill(Position::new(p.x + 1, p.y), color);
        self.fill(Position::new(p.x, p.y - 1), color);
        self.fill(Position::new(p.x, p.y + 1), color);
    }

    fn draw_line(&mut self, from: Position, to: Position) {
        // println!("draw line: {:?} -> {:?}", from, to);
        let line = get_line_points(from, to);
        self.cur_position = to;
        let color = self.pen_colors[self.line_color].clone();
        for p in line {
            self.draw_pixel(p, &color);
        }
    }

    fn write_text(&mut self, text_pos: Position, string_parameter: &str) {
        let mut pos = text_pos;
        let char_size = self.font_8px.size;
        let color = self.pen_colors[self.text_color].clone();

        for ch in string_parameter.chars() {
            let data = self.font_8px.get_glyph(ch).unwrap().data.clone();
            for y in 0..char_size.height {
                for x in 0..char_size.width {
                    if data[y as usize] & (128 >> x) != 0 {
                        self.draw_pixel(pos + Position::new(x * 2, y * 2), &color);
                        self.draw_pixel(pos + Position::new(x * 2 + 1, y * 2), &color);
                        self.draw_pixel(pos + Position::new(x * 2, y * 2 + 1), &color);
                        self.draw_pixel(pos + Position::new(x * 2 + 1, y * 2 + 1), &color);
                    }
                }
            }
            match self.text_rotation {
                TextRotation::RightReverse | TextRotation::Right => pos.x += char_size.width * 2,
                TextRotation::Up => pos.y -= char_size.height * 2,
                TextRotation::Down => pos.y += char_size.height * 2,
                TextRotation::Left => pos.x -= char_size.width * 2,
            }
        }
    }

    fn blit_screen_to_screen(&mut self, _write_mode: i32, from: Position, to: Position, dest: Position) {
        let res = self.get_resolution();

        for y in from.y..to.y {
            let y_pos = dest.y + y - from.y;
            if y < 0 || y_pos < 0 {
                continue;
            }

            if y_pos >= res.height {
                break;
            }
            if y >= res.height {
                break;
            }
            let sy = (y * res.width * 4) as usize;
            let dy = (y_pos * res.width * 4) as usize;

            for x in from.x..to.x {
                let x_pos = x - from.x + dest.x;
                if x < 0 || x_pos < 0 {
                    continue;
                }
                if x_pos >= res.width {
                    break;
                }
                if x >= res.width {
                    break;
                }

                let srcptr = sy + x as usize * 4;
                let dstptr = dy + x_pos as usize * 4;
                for i in 0..4 {
                    self.igs_texture[dstptr + i] = self.igs_texture[srcptr + i];
                }
            }
        }
    }

    fn blit_memory_to_screen(&mut self, _write_mode: i32, from: Position, to: Position, dest: Position) {
        let res = self.get_resolution();

        for y in from.y..to.y {
            let y_pos = dest.y + y - from.y;
            if y < 0 || y_pos < 0 {
                continue;
            }
            if y_pos >= res.height {
                break;
            }
            if y >= res.height {
                break;
            }

            let dy = (y_pos * res.width * 4) as usize;
            for x in from.x..to.x {
                let src = x as usize * 4;
                let x_pos = x - from.x + dest.x;
                if x < 0 || x_pos < 0 {
                    continue;
                }
                if src >= res.width as usize {
                    break;
                }
                if x_pos >= res.width {
                    break;
                }

                let dptr = dy + x_pos as usize * 4;
                for i in 0..4 {
                    self.igs_texture[dptr + i] = self.screen_memory[y as usize][src + i];
                }
            }
        }
    }

    fn blit_screen_to_memory(&mut self, _write_mode: i32, from: Position, to: Position) {
        let from = self.translate_pos(from);
        let to = self.translate_pos(to);

        let mut data = Vec::new();
        for y in from.y..to.y {
            let y = (y * self.get_resolution().width * 4) as usize;
            let o1 = y + from.x as usize * 4;
            let o2 = y + to.x as usize * 4;
            data.push(self.igs_texture[o1..o2].to_vec());
        }

        self.screen_memory = data;
    }
}

impl CommandExecutor for DrawExecutor {
    fn get_resolution(&self) -> Size {
        let s = self.terminal_resolution.get_resolution();
        Size::new((s.width as f32 * self.x_scale) as i32, (s.height as f32 * self.y_scale) as i32)
    }

    fn get_texture_data(&self) -> &[u8] {
        &self.igs_texture
    }

    fn execute_command(
        &mut self,
        _buf: &mut Buffer,
        caret: &mut Caret,
        command: IgsCommands,
        parameters: &[i32],
        string_parameter: &str,
    ) -> EngineResult<CallbackAction> {
        //  println!("cmd:{command:?}");
        match command {
            IgsCommands::Initialize => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match parameters[0] {
                    0 => {
                        self.set_resolution();
                        self.pen_colors = IGS_SYSTEM_PALETTE.to_vec();
                        self.reset_attributes();
                    }
                    1 => {
                        self.set_resolution();
                        self.pen_colors = IGS_SYSTEM_PALETTE.to_vec();
                    }
                    2 => {
                        self.reset_attributes();
                    }
                    3 => {
                        self.set_resolution();
                        self.pen_colors = IGS_PALETTE.to_vec();
                    }
                    x => return Err(anyhow::anyhow!("Initialize unknown/unsupported argument: {x}")),
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::ScreenClear => {
                self.clear();
                Ok(CallbackAction::Update)
            }
            IgsCommands::AskIG => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match parameters[0] {
                    0 => Ok(CallbackAction::SendString(IGS_VERSION.to_string())),
                    3 => Ok(CallbackAction::SendString(self.terminal_resolution.resolution_id())),
                    x => Err(anyhow::anyhow!("AskIG unknown/unsupported argument: {x}")),
                }
            }
            IgsCommands::Cursor => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Cursor command requires 1 argument"));
                }
                match parameters[0] {
                    0 => caret.set_is_visible(false),
                    1 => caret.set_is_visible(true),
                    2 | 3 => {
                        log::warn!("Backspace options not supported.");
                    }
                    x => return Err(anyhow::anyhow!("Cursor unknown/unsupported argument: {x}")),
                }
                Ok(CallbackAction::Update)
            }

            IgsCommands::ColorSet => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("ColorSet command requires 2 arguments"));
                }
                match parameters[0] {
                    0 => self.polymarker_color = parameters[1] as usize,
                    1 => self.line_color = parameters[1] as usize,
                    2 => self.fill_color = parameters[1] as usize,
                    3 => self.text_color = parameters[1] as usize,
                    x => return Err(anyhow::anyhow!("ColorSet unknown/unsupported argument: {x}")),
                }
                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::SetPenColor => {
                if parameters.len() != 4 {
                    return Err(anyhow::anyhow!("SetPenColor command requires 4 arguments"));
                }

                let color = parameters[0];
                if !(0..=15).contains(&color) {
                    return Err(anyhow::anyhow!("ColorSet unknown/unsupported argument: {color}"));
                }
                self.pen_colors[color as usize] = Color::new(
                    (parameters[1] as u8) << 5 | parameters[1] as u8,
                    (parameters[2] as u8) << 5 | parameters[2] as u8,
                    (parameters[3] as u8) << 5 | parameters[3] as u8,
                );
                println!("set color: {} {:?} param?{:?}", color, self.pen_colors[color as usize], parameters);
                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::DrawLine => {
                if parameters.len() != 4 {
                    return Err(anyhow::anyhow!("DrawLine command requires 4 arguments"));
                }
                let next_pos = self.translate_pos(Position::new(parameters[2], parameters[3]));
                let from = self.translate_pos(Position::new(parameters[0], parameters[1]));
                self.draw_line(from, next_pos);
                Ok(CallbackAction::Update)
            }

            IgsCommands::LineDrawTo => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("LineDrawTo command requires 2 arguments"));
                }
                let next_pos = self.translate_pos(Position::new(parameters[0], parameters[1]));
                self.draw_line(self.cur_position, next_pos);
                Ok(CallbackAction::Update)
            }
            IgsCommands::Box => {
                if parameters.len() != 5 {
                    return Err(anyhow::anyhow!("Box command requires 5 arguments"));
                }
                let min = self.translate_pos(Position::new(parameters[0], parameters[1]));
                let max = self.translate_pos(Position::new(parameters[2], parameters[3]));
                let line = get_line_points(Position::new(min.x, min.y), Position::new(max.x, min.y));
                let color = self.pen_colors[self.line_color].clone();
                for p in line {
                    self.draw_pixel(p, &color);
                }
                let line = get_line_points(Position::new(min.x, max.y), Position::new(max.x, max.y));
                for p in line {
                    self.draw_pixel(p, &color);
                }
                let line = get_line_points(Position::new(min.x, min.y), Position::new(min.x, max.y));
                for p in line {
                    self.draw_pixel(p, &color);
                }
                let line = get_line_points(Position::new(max.x, min.y), Position::new(max.x, max.y));
                for p in line {
                    self.draw_pixel(p, &color);
                }

                Ok(CallbackAction::Update)
            }

            IgsCommands::FilledRectangle => {
                if parameters.len() != 4 {
                    return Err(anyhow::anyhow!("FilledRectangle command requires 4 arguments"));
                }
                let min = self.translate_pos(Position::new(parameters[0], parameters[1]));
                let max = self.translate_pos(Position::new(parameters[2], parameters[3]));

                let c = self.pen_colors[self.fill_color].clone();
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        self.draw_pixel(Position::new(x, y), &c);
                    }
                }

                Ok(CallbackAction::Update)
            }

            IgsCommands::TimeAPause => Ok(CallbackAction::Pause(1000 * parameters[0] as u32)),

            IgsCommands::PolymarkerPlot => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("PolymarkerPlot command requires 2 arguments"));
                }
                let next_pos = Position::new(parameters[0], parameters[1]);
                let color = self.pen_colors[self.line_color].clone();
                self.draw_pixel(next_pos, &color);
                self.cur_position = next_pos;
                Ok(CallbackAction::Update)
            }

            IgsCommands::TextEffects => {
                if parameters.len() != 3 {
                    return Err(anyhow::anyhow!("PolymarkerPlot command requires 2 arguments"));
                }
                match parameters[0] {
                    0 => self.text_effects = TextEffects::Normal,
                    1 => self.text_effects = TextEffects::Thickened,
                    2 => self.text_effects = TextEffects::Ghosted,
                    4 => self.text_effects = TextEffects::Skewed,
                    8 => self.text_effects = TextEffects::Underlined,
                    16 => self.text_effects = TextEffects::Outlined,
                    _ => return Err(anyhow::anyhow!("TextEffects unknown/unsupported argument: {}", parameters[0])),
                }

                match parameters[1] {
                    8 | 9 | 10 | 16 | 18 | 20 => self.text_size = parameters[1],
                    _ => return Err(anyhow::anyhow!("TextEffects unknown/unsupported argument: {}", parameters[1])),
                }

                match parameters[2] {
                    0 => self.text_rotation = TextRotation::Right,
                    1 => self.text_rotation = TextRotation::Up,
                    2 => self.text_rotation = TextRotation::Down,
                    3 => self.text_rotation = TextRotation::Left,
                    4 => self.text_rotation = TextRotation::RightReverse,
                    _ => return Err(anyhow::anyhow!("TextEffects unknown/unsupported argument: {}", parameters[2])),
                }
                Ok(CallbackAction::Update)
            }

            IgsCommands::LineMarkerTypes => {
                if parameters.len() != 3 {
                    return Err(anyhow::anyhow!("LineMarkerTypes command requires 3 arguments"));
                }
                if parameters[0] == 1 {
                    match parameters[1] {
                        1 => self.polymaker_type = PolymarkerType::Point,
                        2 => self.polymaker_type = PolymarkerType::Plus,
                        3 => self.polymaker_type = PolymarkerType::Star,
                        4 => self.polymaker_type = PolymarkerType::Square,
                        5 => self.polymaker_type = PolymarkerType::DiagonalCross,
                        6 => self.polymaker_type = PolymarkerType::Diamond,
                        _ => return Err(anyhow::anyhow!("LineMarkerTypes unknown/unsupported argument: {}", parameters[0])),
                    }
                    self.polymarker_size = parameters[2] as usize;
                } else if parameters[0] == 2 {
                    match parameters[1] {
                        1 => self.line_type = LineType::Solid,
                        2 => self.line_type = LineType::LongDash,
                        3 => self.line_type = LineType::DottedLine,
                        4 => self.line_type = LineType::DashDot,
                        5 => self.line_type = LineType::DashedLine,
                        6 => self.line_type = LineType::DashedDotDot,
                        7 => self.line_type = LineType::UserDefined,
                        _ => return Err(anyhow::anyhow!("LineMarkerTypes unknown/unsupported argument: {}", parameters[1])),
                    }
                    if self.line_type == LineType::UserDefined {
                        self.user_defined_pattern_number = parameters[2] as usize;
                    } else {
                        self.solidline_size = parameters[2] as usize;
                    }
                } else {
                    return Err(anyhow::anyhow!("LineMarkerTypes unknown/unsupported argument: {}", parameters[0]));
                }
                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::DrawingMode => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("DrawingMode command requires 1 argument"));
                }
                match parameters[0] {
                    1 => self.drawing_mode = DrawingMode::Replace,
                    2 => self.drawing_mode = DrawingMode::Transparent,
                    3 => self.drawing_mode = DrawingMode::Xor,
                    4 => self.drawing_mode = DrawingMode::ReverseTransparent,
                    _ => return Err(anyhow::anyhow!("DrawingMode unknown/unsupported argument: {}", parameters[0])),
                }
                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::SetResolution => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("SetResolution command requires 2 argument"));
                }
                match parameters[0] {
                    0 => self.terminal_resolution = TerminalResolution::Low,
                    1 => self.terminal_resolution = TerminalResolution::Medium,
                    _ => return Err(anyhow::anyhow!("SetResolution unknown/unsupported argument: {}", parameters[0])),
                }
                match parameters[1] {
                    0 => { // no change
                    }
                    1 => {
                        // default system colors
                        self.pen_colors = IGS_SYSTEM_PALETTE.to_vec();
                    }
                    2 => {
                        // IG colors
                        self.pen_colors = IGS_PALETTE.to_vec();
                    }
                    _ => return Err(anyhow::anyhow!("SetResolution unknown/unsupported argument: {}", parameters[1])),
                }

                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::WriteText => {
                if parameters.len() != 3 {
                    return Err(anyhow::anyhow!("WriteText command requires 3 arguments"));
                }
                let text_pos = self.translate_pos(Position::new(parameters[0], parameters[1]));
                self.write_text(text_pos, string_parameter);
                Ok(CallbackAction::Update)
            }

            IgsCommands::FloodFill => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("FloodFill command requires 2 arguments"));
                }
                let next_pos = Position::new(parameters[0], parameters[1]);
                self.flood_fill(next_pos);
                println!("fill {}", self.fill_color);
                Ok(CallbackAction::Pause(100))
            }

            IgsCommands::GrabScreen => {
                if parameters.len() < 2 {
                    return Err(anyhow::anyhow!("GrabScreen command requires > 2 argument"));
                }
                let write_mode = parameters[1];

                match parameters[0] {
                    0 => {
                        if parameters.len() != 8 {
                            return Err(anyhow::anyhow!("GrabScreen screen to screen command requires 8 argument"));
                        }
                        let from_start = Position::new(parameters[2], parameters[3]);
                        let from_end = Position::new(parameters[4], parameters[5]);
                        let dest = Position::new(parameters[6], parameters[7]);
                        self.blit_screen_to_screen(write_mode, from_start, from_end, dest);
                    }

                    1 => {
                        if parameters.len() != 6 {
                            return Err(anyhow::anyhow!("GrabScreen screen to memory command requires 6 argument"));
                        }
                        let from_start = Position::new(parameters[2], parameters[3]);
                        let from_end = Position::new(parameters[4], parameters[5]);
                        self.blit_screen_to_memory(write_mode, from_start, from_end);
                    }

                    2 => {
                        if parameters.len() != 4 {
                            return Err(anyhow::anyhow!("GrabScreen memory to screen command requires 4 argument"));
                        }
                        let dest = Position::new(parameters[2], parameters[3]);
                        self.blit_memory_to_screen(
                            write_mode,
                            Position::new(0, 0),
                            Position::new(self.screen_memory[0].len() as i32, self.screen_memory.len() as i32),
                            dest,
                        );
                    }

                    3 => {
                        if parameters.len() != 8 {
                            return Err(anyhow::anyhow!("GrabScreen piece of memory to screen command requires 4 argument"));
                        }
                        let from_start = Position::new(parameters[2], parameters[3]);
                        let from_end = Position::new(parameters[4], parameters[5]);
                        let dest = Position::new(parameters[6], parameters[7]);
                        self.blit_memory_to_screen(write_mode, from_start, from_end, dest);
                    }
                    _ => return Err(anyhow::anyhow!("GrabScreen unknown/unsupported grab screen mode: {}", parameters[0])),
                }

                Ok(CallbackAction::Update)
            }
            _ => Err(anyhow::anyhow!("Unimplemented IGS command: {command:?}")),
        }
    }
}
