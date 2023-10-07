use super::{cmd::IgsCommands, CommandExecutor, IGS_VERSION};
use crate::{paint::get_line_points, Buffer, CallbackAction, Caret, Color, EngineResult, Position, Size, DOS_DEFAULT_PALETTE, IGS_PALETTE};

#[derive(Default)]
pub enum TerminalResolution {
    /// 320x200
    Low,
    /// 640x200
    #[default]
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
    Normal,
    Up,
    Down,
    Reverse,
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
}

impl Default for DrawExecutor {
    fn default() -> Self {
        let mut res = Self {
            igs_texture: Vec::new(),
            terminal_resolution: TerminalResolution::default(),
            x_scale: 2.0,
            y_scale: 2.0,
            pen_colors: IGS_PALETTE.to_vec(),
            polymarker_color: 0,
            line_color: 0,
            fill_color: 0,
            text_color: 0,
            cur_position: Position::new(0, 0),
            text_effects: TextEffects::Normal,
            text_size: 8,
            text_rotation: TextRotation::Normal,
            polymaker_type: PolymarkerType::Point,
            line_type: LineType::Solid,
            drawing_mode: DrawingMode::Replace,
            polymarker_size: 1,
            solidline_size: 1,
            user_defined_pattern_number: 1,
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

    pub fn set_palette(&mut self) {
        self.pen_colors = IGS_PALETTE.to_vec();
    }

    pub fn reset_attributes(&mut self) {
        // TODO
    }

    fn draw_pixel(&mut self, p: Position, color: &Color) {
        let offset = p.x as usize * 4 + p.y as usize * self.get_resolution().width as usize * 4;
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
        self.fill(pos, self.get_pixel(pos));
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
}

impl CommandExecutor for DrawExecutor {
    fn get_resolution(&self) -> Size {
        let s = self.terminal_resolution.get_resolution();
        Size::new((s.width as f32 * self.x_scale) as i32, (s.height as f32 * self.y_scale) as i32)
    }

    fn get_texture_data(&self) -> &[u8] {
        &self.igs_texture
    }

    fn execute_command(&mut self, _buf: &mut Buffer, caret: &mut Caret, command: IgsCommands, parameters: &[i32]) -> EngineResult<CallbackAction> {
        match command {
            IgsCommands::Initialize => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match parameters[0] {
                    0 => {
                        self.set_resolution();
                        self.set_palette();
                        self.reset_attributes();
                    }
                    1 => {
                        self.set_resolution();
                        self.set_palette();
                    }
                    2 => {
                        self.reset_attributes();
                    }
                    3 => {
                        self.set_resolution();
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
                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::DrawLine => {
                if parameters.len() != 4 {
                    return Err(anyhow::anyhow!("DrawLine command requires 4 arguments"));
                }
                let next_pos = self.translate_pos(Position::new(parameters[2], parameters[3]));
                let line = get_line_points(self.translate_pos(Position::new(parameters[0], parameters[1])), next_pos);
                self.cur_position = next_pos;
                let color = self.pen_colors[self.line_color].clone();
                for p in line {
                    self.draw_pixel(p, &color);
                }
                Ok(CallbackAction::Update)
            }

            IgsCommands::LineDrawTo => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("LineDrawTo command requires 2 arguments"));
                }
                let next_pos = self.translate_pos(Position::new(parameters[0], parameters[1]));
                let line: Vec<Position> = get_line_points(self.cur_position, next_pos);
                self.cur_position = next_pos;
                let color = self.pen_colors[self.line_color].clone();
                for p in line {
                    self.draw_pixel(p, &color);
                }
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
                    0 | 4 => self.text_rotation = TextRotation::Normal,
                    1 => self.text_rotation = TextRotation::Up,
                    2 => self.text_rotation = TextRotation::Down,
                    3 => self.text_rotation = TextRotation::Reverse,
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
                        self.pen_colors = DOS_DEFAULT_PALETTE.to_vec();
                    }
                    2 => {
                        // IG colors
                        self.pen_colors = IGS_PALETTE.to_vec();
                    }
                    _ => return Err(anyhow::anyhow!("SetResolution unknown/unsupported argument: {}", parameters[1])),
                }

                Ok(CallbackAction::NoUpdate)
            }

            IgsCommands::FloodFill => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("FloodFill command requires 2 argument"));
                }
                let next_pos = Position::new(parameters[0], parameters[1]);
                self.flood_fill(next_pos);
                Ok(CallbackAction::Update)
            }
            _ => Err(anyhow::anyhow!("Unimplemented IGS command: {command:?}")),
        }
    }
}
