use super::{cmd::IgsCommands, CommandExecutor, IGS_VERSION};
use crate::{BitFont, Buffer, CallbackAction, Caret, Color, EngineResult, Position, Size, ATARI, IGS_PALETTE, IGS_SYSTEM_PALETTE};
use raqote::*;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawingMode {
    Replace,
    Transparent,
    Xor,
    ReverseTransparent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FillPatternType {
    Hollow,
    Solid,
    Pattern,
    Hatch,
    UserdDefined,
}
pub struct DrawExecutor {
    dt: DrawTarget,
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

    fill_pattern_type: FillPatternType,
    pattern_index_number: usize,
    draw_border: bool,

    font_8px: BitFont,
    hollow_set: bool,
    screen_memory: DrawTarget,

    /// for the G command.
    double_step: f32,
}

impl Default for DrawExecutor {
    fn default() -> Self {
        Self {
            dt: DrawTarget::new(320, 200),
            terminal_resolution: TerminalResolution::Low,
            x_scale: 1.0,
            y_scale: 1.0,
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
            screen_memory: DrawTarget::new(1, 1),

            fill_pattern_type: FillPatternType::Solid,
            pattern_index_number: 0,
            draw_border: false,
            hollow_set: false,
            double_step: -1.0,
        }
    }
}

impl DrawExecutor {
    pub fn clear(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        buf.clear_screen(0, caret);
        self.dt.clear(SolidSource::from_unpremultiplied_argb(0, 0, 0, 0));
    }

    pub fn set_resolution(&mut self, buf: &mut Buffer, caret: &mut Caret) {
        buf.clear_screen(0, caret);
        let res = self.get_resolution();
        self.dt = DrawTarget::new(res.width, res.height);
    }

    pub fn reset_attributes(&mut self) {
        // TODO
    }

    /*
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
    }*/

    fn translate_pos(&self, pt: Position) -> Position {
        Position::new((pt.x as f32 * self.x_scale) as i32, (pt.y as f32 * self.y_scale) as i32)
    }

    fn draw_line(&mut self, from: Position, to: Position) {
        // println!("draw line: {:?} -> {:?}", from, to);
        self.cur_position = to;

        let mut pb = PathBuilder::new();
        pb.move_to(from.x as f32, from.y as f32);
        pb.line_to(to.x as f32, to.y as f32);
        let path = pb.finish();

        let (r, g, b) = self.pen_colors[self.line_color].get_rgb();
        self.dt.stroke(
            &path,
            &Source::Solid(create_solid_source(r, g, b)),
            &StrokeStyle::default(),
            &DrawOptions::new(),
        );
    }

    fn write_text(&mut self, text_pos: Position, string_parameter: &str) {
        let mut pos = text_pos;
        let char_size = self.font_8px.size;
        let (r, g, b) = self.pen_colors[self.text_color].get_rgb();

        for ch in string_parameter.chars() {
            let data = self.font_8px.get_glyph(ch).unwrap().data.clone();
            for y in 0..char_size.height {
                for x in 0..char_size.width {
                    if data[y as usize] & (128 >> x) != 0 {
                        let p = pos + Position::new(x, y);
                        self.dt.fill_rect(
                            p.x as f32,
                            p.y as f32,
                            1.0,
                            1.0,
                            &Source::Solid(create_solid_source(r, g, b)),
                            &DrawOptions::default(),
                        );
                    }
                }
            }
            match self.text_rotation {
                TextRotation::RightReverse | TextRotation::Right => pos.x += char_size.width,
                TextRotation::Up => pos.y -= char_size.height,
                TextRotation::Down => pos.y += char_size.height,
                TextRotation::Left => pos.x -= char_size.width,
            }
        }
    }

    fn blit_screen_to_screen(&mut self, _write_mode: i32, from: Position, to: Position, dest: Position) {
        let mut copy = DrawTarget::new(to.x - from.x, to.y - from.y);
        copy.copy_surface(
            &self.dt,
            IntRect::from_points([IntPoint::new(from.x, from.y), IntPoint::new(to.x, to.y)]),
            IntPoint::new(0, 0),
        );
        let point2_d = IntPoint::new(copy.width(), copy.height());
        self.dt
            .copy_surface(&copy, IntRect::from_points([IntPoint::new(0, 0), point2_d]), IntPoint::new(dest.x, dest.y));
    }

    fn blit_memory_to_screen(&mut self, _write_mode: i32, from: Position, to: Position, dest: Position) {
        self.dt.copy_surface(
            &self.screen_memory,
            IntRect::new(IntPoint::new(from.x, from.y), IntPoint::new(to.x, to.y)),
            IntPoint::new(dest.x, dest.y),
        );
    }

    fn blit_screen_to_memory(&mut self, _write_mode: i32, from: Position, to: Position) {
        let mut copy = DrawTarget::new(to.x - from.x, to.y - from.y);
        copy.copy_surface(
            &self.dt,
            IntRect::from_points([IntPoint::new(from.x, from.y), IntPoint::new(to.x, to.y)]),
            IntPoint::new(0, 0),
        );

        self.screen_memory = copy;
    }
}

impl CommandExecutor for DrawExecutor {
    fn get_resolution(&self) -> Size {
        let s = self.terminal_resolution.get_resolution();
        Size::new((s.width as f32 * self.x_scale) as i32, (s.height as f32 * self.y_scale) as i32)
    }

    fn get_texture_data(&self) -> &[u8] {
        self.dt.get_data_u8()
    }

    fn get_picture_data(&self) -> Option<(Size, Vec<u8>)> {
        Some((self.get_resolution(), self.dt.get_data_u8().to_vec()))
    }
    fn execute_command(
        &mut self,
        buf: &mut Buffer,
        caret: &mut Caret,
        command: IgsCommands,
        parameters: &[i32],
        string_parameter: &str,
    ) -> EngineResult<CallbackAction> {
        println!("cmd:{command:?} params:{parameters:?}");
        match command {
            IgsCommands::Initialize => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match parameters[0] {
                    0 => {
                        self.set_resolution(buf, caret);
                        self.pen_colors = IGS_SYSTEM_PALETTE.to_vec();
                        self.reset_attributes();
                    }
                    1 => {
                        self.set_resolution(buf, caret);
                        self.pen_colors = IGS_SYSTEM_PALETTE.to_vec();
                    }
                    2 => {
                        self.reset_attributes();
                    }
                    3 => {
                        self.set_resolution(buf, caret);
                        self.pen_colors = IGS_PALETTE.to_vec();
                    }
                    x => return Err(anyhow::anyhow!("Initialize unknown/unsupported argument: {x}")),
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::ScreenClear => {
                self.clear(buf, caret);
                Ok(CallbackAction::Update)
            }
            IgsCommands::AskIG => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("Initialize command requires 1 argument"));
                }
                match parameters[0] {
                    0 => Ok(CallbackAction::SendString(IGS_VERSION.to_string())),
                    3 => Ok(CallbackAction::SendString(self.terminal_resolution.resolution_id() + ":")),
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
                let from = self.translate_pos(Position::new(parameters[0], parameters[1]));
                self.draw_line(from, next_pos);
                Ok(CallbackAction::Update)
            }
            IgsCommands::PolyFill => {
                if parameters.is_empty() {
                    return Err(anyhow::anyhow!("PolyLine requires minimun 1 arguments"));
                }
                let points = parameters[0];
                if points * 2 + 1 != parameters.len() as i32 {
                    return Err(anyhow::anyhow!("PolyLine requires {} arguments was {} ", points * 2 + 1, parameters.len()));
                }
                let mut pb = PathBuilder::new();
                pb.move_to(parameters[1] as f32, parameters[2] as f32);
                let mut i = 3;
                while i < parameters.len() {
                    pb.line_to(parameters[i] as f32, parameters[i + 1] as f32);
                    i += 2;
                }
                pb.line_to(parameters[1] as f32, parameters[2] as f32);

                let path = pb.finish();
                if matches!(self.fill_pattern_type, FillPatternType::Solid) {
                    let (r, g, b) = self.pen_colors[self.fill_color].get_rgb();
                    self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());
                }

                if self.draw_border {
                    let (r, g, b) = self.pen_colors[self.line_color].get_rgb();
                    self.dt.stroke(
                        &path,
                        &Source::Solid(create_solid_source(r, g, b)),
                        &StrokeStyle {
                            width: 1., // <--
                            ..StrokeStyle::default()
                        },
                        &DrawOptions::new(),
                    );
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::PolyLine => {
                if parameters.is_empty() {
                    return Err(anyhow::anyhow!("PolyLine requires minimun 1 arguments"));
                }
                let points = parameters[0];
                if points * 2 + 1 != parameters.len() as i32 {
                    return Err(anyhow::anyhow!("PolyLine requires {} arguments was {} ", points * 2 + 1, parameters.len()));
                }
                let mut pb = PathBuilder::new();
                pb.move_to(parameters[1] as f32, parameters[2] as f32);
                let mut i = 3;
                while i < parameters.len() {
                    pb.line_to(parameters[i] as f32, parameters[i + 1] as f32);
                    i += 2;
                }
                pb.line_to(parameters[1] as f32, parameters[2] as f32);

                let path = pb.finish();
                let (r, g, b) = self.pen_colors[self.line_color].get_rgb();
                self.dt.stroke(
                    &path,
                    &Source::Solid(create_solid_source(r, g, b)),
                    &StrokeStyle {
                        width: 1., // <--
                        ..StrokeStyle::default()
                    },
                    &DrawOptions::new(),
                );
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
                let mut pb = PathBuilder::new();
                pb.move_to(min.x as f32, min.y as f32);
                pb.line_to(max.x as f32, min.y as f32);
                pb.line_to(max.x as f32, max.y as f32);
                pb.line_to(min.x as f32, max.y as f32);
                pb.line_to(min.x as f32, min.y as f32);
                let path = pb.finish();

                if matches!(self.fill_pattern_type, FillPatternType::Solid) {
                    let (r, g, b) = self.pen_colors[self.fill_color].get_rgb();
                    self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());
                }

                if self.draw_border {
                    let (r, g, b) = self.pen_colors[self.line_color].get_rgb();
                    self.dt.stroke(
                        &path,
                        &Source::Solid(create_solid_source(r, g, b)),
                        &StrokeStyle {
                            width: 1., // <--
                            ..StrokeStyle::default()
                        },
                        &DrawOptions::new(),
                    );
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::HollowSet => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("HollowSet command requires 1 argument"));
                }
                match parameters[0] {
                    0 => self.hollow_set = false,
                    1 => self.hollow_set = true,
                    x => return Err(anyhow::anyhow!("HollowSet unknown/unsupported argument: {x}")),
                }
                Ok(CallbackAction::NoUpdate)
            }
            IgsCommands::Pieslice => {
                if parameters.len() != 5 {
                    return Err(anyhow::anyhow!("AttributeForFills command requires 3 arguments"));
                }
                let mut pb = PathBuilder::new();
                pb.arc(
                    parameters[0] as f32,
                    parameters[1] as f32,
                    parameters[2] as f32,
                    parameters[3] as f32 / 360.0 * 2.0 * std::f32::consts::PI,
                    parameters[4] as f32 / 360.0 * 2.0 * std::f32::consts::PI,
                );
                let path = pb.finish();

                let (r, g, b) = self.pen_colors[self.fill_color].get_rgb();
                self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());

                Ok(CallbackAction::Update)
            }

            IgsCommands::Circle => {
                if parameters.len() != 3 {
                    return Err(anyhow::anyhow!("AttributeForFills command requires 3 arguments"));
                }
                let mut pb = PathBuilder::new();
                pb.arc(
                    parameters[0] as f32,
                    parameters[1] as f32,
                    parameters[2] as f32,
                    0.0,
                    2.0 * std::f32::consts::PI,
                );
                let path = pb.finish();

                if self.hollow_set {
                    let (r, g, b) = self.pen_colors[self.line_color].get_rgb();
                    self.dt.stroke(
                        &path,
                        &Source::Solid(create_solid_source(r, g, b)),
                        &StrokeStyle {
                            width: 1., // <--
                            ..StrokeStyle::default()
                        },
                        &DrawOptions::new(),
                    );
                } else {
                    let (r, g, b) = self.pen_colors[self.fill_color].get_rgb();
                    self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());
                }

                Ok(CallbackAction::Update)
            }
            IgsCommands::QuickPause => {
                if parameters.len() != 1 {
                    return Err(anyhow::anyhow!("QuickPause command requires 1 arguments"));
                }
                match parameters[0] {
                    9995 => {
                        self.double_step = 4.0;
                        Ok(CallbackAction::NoUpdate)
                    }
                    9996 => {
                        self.double_step = 3.0;
                        Ok(CallbackAction::NoUpdate)
                    }
                    9997 => {
                        self.double_step = 2.0;
                        Ok(CallbackAction::NoUpdate)
                    }
                    9998 => {
                        self.double_step = 1.0;
                        Ok(CallbackAction::NoUpdate)
                    }
                    9999 => {
                        self.double_step = -1.0;
                        Ok(CallbackAction::NoUpdate)
                    }
                    p => {
                        if p < 180 {
                            Ok(CallbackAction::Pause((p as f32 * 1000.0 / 60.0) as u32))
                        } else {
                            Err(anyhow::anyhow!("Quick pause invalid {}", p))
                        }
                    }
                }
            }
            IgsCommands::AttributeForFills => {
                if parameters.len() != 3 {
                    return Err(anyhow::anyhow!("AttributeForFills command requires 3 arguments"));
                }
                match parameters[0] {
                    0 => self.fill_pattern_type = FillPatternType::Hollow,
                    1 => self.fill_pattern_type = FillPatternType::Solid,
                    2 => self.fill_pattern_type = FillPatternType::Pattern,
                    3 => self.fill_pattern_type = FillPatternType::Hatch,
                    4 => self.fill_pattern_type = FillPatternType::UserdDefined,
                    _ => return Err(anyhow::anyhow!("AttributeForFills unknown/unsupported argument: {}", parameters[0])),
                }
                self.pattern_index_number = parameters[1] as usize;
                match parameters[2] {
                    0 => self.draw_border = false,
                    1 => self.draw_border = true,
                    _ => return Err(anyhow::anyhow!("AttributeForFills unknown/unsupported argument: {}", parameters[2])),
                }
                Ok(CallbackAction::NoUpdate)
            }
            IgsCommands::FilledRectangle => {
                if parameters.len() != 4 {
                    return Err(anyhow::anyhow!("FilledRectangle command requires 4 arguments"));
                }
                let min = self.translate_pos(Position::new(parameters[0], parameters[1]));
                let max = self.translate_pos(Position::new(parameters[2], parameters[3]));

                let mut pb = PathBuilder::new();
                pb.move_to(min.x as f32, min.y as f32);
                pb.line_to(max.x as f32, min.y as f32);
                pb.line_to(max.x as f32, max.y as f32);
                pb.line_to(min.x as f32, max.y as f32);
                pb.line_to(min.x as f32, min.y as f32);
                let (r, g, b) = self.pen_colors[self.fill_color].get_rgb();
                let path = pb.finish();
                self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());

                Ok(CallbackAction::Update)
            }

            IgsCommands::TimeAPause => Ok(CallbackAction::Pause(1000 * parameters[0] as u32)),

            IgsCommands::PolymarkerPlot => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("PolymarkerPlot command requires 2 arguments"));
                }
                let next_pos = Position::new(parameters[0], parameters[1]);
                let mut pb = PathBuilder::new();
                pb.rect(next_pos.x as f32, next_pos.y as f32, 1.0, 1.0);
                let (r, g, b) = self.pen_colors[self.polymarker_color].get_rgb();
                let path = pb.finish();
                self.dt.fill(&path, &Source::Solid(create_solid_source(r, g, b)), &DrawOptions::new());
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
                // self.dt.fill(path, src, options);
                //      self.flood_fill(next_pos);
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
                            Position::new(self.screen_memory.width(), self.screen_memory.height()),
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

                if self.double_step >= 0.0 {
                    println!("double step:{} = {}ms", self.double_step, (self.double_step * 1000.0 / 60.0) as u32);
                    Ok(CallbackAction::Pause((self.double_step * 1000.0 / 60.0) as u32))
                } else {
                    Ok(CallbackAction::Update)
                }
            }

            IgsCommands::VTColor => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("VTColor command requires 2 argument"));
                }
                if let Some(pen) = REGISTER_TO_PEN.get(parameters[1] as usize) {
                    let color = self.pen_colors[*pen].clone();

                    if parameters[0] == 0 {
                        caret.set_background(buf.palette.insert_color(color));
                    } else if parameters[0] == 1 {
                        caret.set_foreground(buf.palette.insert_color(color));
                    } else {
                        return Err(anyhow::anyhow!("VTColor unknown/unsupported color mode: {}", parameters[0]));
                    }
                    Ok(CallbackAction::NoUpdate)
                } else {
                    Err(anyhow::anyhow!("VTColor unknown/unsupported color selector: {}", parameters[1]))
                }
            }
            IgsCommands::VTPosition => {
                if parameters.len() != 2 {
                    return Err(anyhow::anyhow!("VTPosition command requires 2 argument"));
                }
                caret.set_position(Position::new(parameters[0], parameters[1]));
                Ok(CallbackAction::NoUpdate)
            }
            _ => Err(anyhow::anyhow!("Unimplemented IGS command: {command:?}")),
        }
    }
}

fn create_solid_source(r: u8, g: u8, b: u8) -> SolidSource {
    SolidSource { r: b, g, b: r, a: 0xFF }
}

const REGISTER_TO_PEN: &[usize; 17] = &[0, 2, 3, 6, 4, 7, 5, 8, 9, 10, 11, 14, 12, 12, 15, 13, 1];

/*
https://github.com/servo/servo/blob/master/components/canvas/raqote_backend.rs#L701

    fn ellipse(
        &mut self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        let mut start = Angle::radians(start_angle);
        let mut end = Angle::radians(end_angle);

        // Wrap angles mod 2 * PI if necessary
        if !anticlockwise && start > end + Angle::two_pi() ||
            anticlockwise && end > start + Angle::two_pi()
        {
            start = start.positive();
            end = end.positive();
        }

        // Calculate the total arc we're going to sweep.
        let sweep = match anticlockwise {
            true => {
                if end - start == Angle::two_pi() {
                    -Angle::two_pi()
                } else if end > start {
                    -(Angle::two_pi() - (end - start))
                } else {
                    -(start - end)
                }
            },
            false => {
                if start - end == Angle::two_pi() {
                    Angle::two_pi()
                } else if start > end {
                    Angle::two_pi() - (start - end)
                } else {
                    end - start
                }
            },
        };

        let arc: Arc<f32> = Arc {
            center: origin,
            radii: Vector2D::new(radius_x, radius_y),
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: Angle::radians(rotation_angle),
        };

        self.line_to(arc.from());

        arc.for_each_quadratic_bezier(&mut |q| {
            self.quadratic_curve_to(&q.ctrl, &q.to);
        });
    }

*/
