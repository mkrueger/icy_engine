use crate::{rip::to_base_36, EngineResult, Position};

use super::{
    bgi::{Bgi, Direction, FontType},
    parse_base_36, Command,
};

#[derive(Default, Clone)]
pub struct TextWindow {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub wrap: bool,
    pub size: i32,
}

impl Command for TextWindow {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            6 | 7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            8 => {
                self.wrap = ch == '1';
                Ok(true)
            }

            9 => {
                self.size = ch.to_digit(32).unwrap() as i32;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|w{}{}{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            i32::from(self.wrap),
            self.size
        )
    }
}

#[derive(Default, Clone)]
pub struct ViewPort {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Command for ViewPort {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|v{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1)
        )
    }
}

#[derive(Default, Clone)]
pub struct ResetWindows {}

impl Command for ResetWindows {
    fn to_rip_string(&self) -> String {
        "|*".to_string()
    }
    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.graph_defaults();
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct EraseWindow {}

impl Command for EraseWindow {
    fn to_rip_string(&self) -> String {
        "|e".to_string()
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.clear_device();
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct EraseView {}

impl Command for EraseView {
    fn to_rip_string(&self) -> String {
        "|E".to_string()
    }
}

#[derive(Default, Clone)]
pub struct GotoXY {
    pub x: i32,
    pub y: i32,
}

impl Command for GotoXY {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }
            3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.move_to(self.x, self.y);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|g{}{}", to_base_36(2, self.x), to_base_36(2, self.y))
    }
}

#[derive(Default, Clone)]
pub struct Home {}

impl Command for Home {
    fn to_rip_string(&self) -> String {
        "|H".to_string()
    }
}

#[derive(Default, Clone)]
pub struct EraseEOL {}

impl Command for EraseEOL {
    fn to_rip_string(&self) -> String {
        "|>".to_string()
    }
}

#[derive(Default, Clone)]
pub struct Color {
    pub c: i32,
}

impl Command for Color {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 => {
                parse_base_36(&mut self.c, ch)?;
                Ok(true)
            }
            1 => {
                parse_base_36(&mut self.c, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_color(self.c as u8);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|c{}", to_base_36(2, self.c))
    }
}

#[derive(Default, Clone)]
pub struct SetPalette {
    pub palette: Vec<i32>,
}

impl Command for SetPalette {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        if *state % 2 == 0 {
            self.palette.push(0);
        }
        let mut c = self.palette.pop().unwrap();
        parse_base_36(&mut c, ch)?;
        self.palette.push(c);

        Ok(*state < 31)
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_palette(&self.palette);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        let mut res = String::from("|Q");
        for c in &self.palette {
            res.push_str(to_base_36(2, *c).as_str());
        }
        res
    }
}

#[derive(Default, Clone)]
pub struct OnePalette {
    pub color: i32,
    pub value: i32,
}

impl Command for OnePalette {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.color, ch)?;
                Ok(true)
            }
            2 => {
                parse_base_36(&mut self.value, ch)?;
                Ok(true)
            }
            3 => {
                parse_base_36(&mut self.value, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_palette_color(self.color, self.value as u8);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|a{}{}", to_base_36(2, self.color), to_base_36(2, self.value))
    }
}

#[derive(Default, Clone)]
pub struct WriteMode {
    pub mode: i32,
}

impl Command for WriteMode {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }
            1 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_write_mode(super::bgi::WriteMode::from(self.mode as u8));
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|W{}", to_base_36(2, self.mode))
    }
}

#[derive(Default, Clone)]
pub struct Move {
    pub x: i32,
    pub y: i32,
}

impl Command for Move {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }
            3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.move_to(self.x, self.y);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|m{}{}", to_base_36(2, self.x), to_base_36(2, self.y))
    }
}

#[derive(Default, Clone)]
pub struct Text {
    pub str: String,
}

impl Command for Text {
    fn parse(&mut self, _state: &mut i32, ch: char) -> EngineResult<bool> {
        self.str.push(ch);
        Ok(true)
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.out_text(&self.str);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|T{}", self.str)
    }
}

#[derive(Default, Clone)]
pub struct TextXY {
    pub x: i32,
    pub y: i32,
    pub str: String,
}

impl Command for TextXY {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }
            _ => {
                self.str.push(ch);
                Ok(true)
            }
        }
    }
    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.out_text_xy(self.x, self.y, &self.str);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|@{}{}{}", to_base_36(2, self.x), to_base_36(2, self.y), self.str)
    }
}

#[derive(Default, Clone)]
pub struct FontStyle {
    pub font: i32,
    pub direction: i32,
    pub size: i32,
    pub res: i32,
}

impl Command for FontStyle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.font, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.direction, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.size, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_text_style(FontType::from(self.font as u8), Direction::from(self.direction as u8), self.size);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|Y{}{}{}{}",
            to_base_36(2, self.font),
            to_base_36(2, self.direction),
            to_base_36(2, self.size),
            to_base_36(2, self.res)
        )
    }
}

#[derive(Default, Clone)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
}

impl Command for Pixel {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }
            3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.put_pixel(self.x, self.y, bgi.get_color());
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|X{}{}", to_base_36(2, self.x), to_base_36(2, self.y))
    }
}

#[derive(Default, Clone)]
pub struct Line {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Command for Line {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.line(self.x0, self.y0, self.x1, self.y1);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|L{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1)
        )
    }
}

#[derive(Default, Clone)]
pub struct Rectangle {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Command for Rectangle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.rectangle(self.x0, self.y0, self.x1, self.y1);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|R{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1)
        )
    }
}

#[derive(Default, Clone)]
pub struct Bar {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Command for Bar {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.bar(self.x0, self.y0, self.x1, self.y1);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|B{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1)
        )
    }
}

#[derive(Default, Clone)]
pub struct Circle {
    pub x_center: i32,
    pub y_center: i32,
    pub radius: i32,
}

impl Command for Circle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x_center, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y_center, ch)?;
                Ok(true)
            }

            4 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(true)
            }

            5 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.circle(self.x_center, self.y_center, self.radius);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|C{}{}{}",
            to_base_36(2, self.x_center),
            to_base_36(2, self.y_center),
            to_base_36(2, self.radius)
        )
    }
}

#[derive(Default, Clone)]
pub struct Oval {
    pub x: i32,
    pub y: i32,
    pub st_ang: i32,
    pub end_ang: i32,
    pub x_rad: i32,
    pub y_rad: i32,
}

impl Command for Oval {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.st_ang, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.end_ang, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.x_rad, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.ellipse(self.x, self.y, self.st_ang, self.end_ang, self.x_rad, self.y_rad);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|O{}{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.st_ang),
            to_base_36(2, self.end_ang),
            to_base_36(2, self.x_rad),
            to_base_36(2, self.y_rad)
        )
    }
}

#[derive(Default, Clone)]
pub struct FilledOval {
    pub x_center: i32,
    pub y_center: i32,
    pub x_rad: i32,
    pub y_rad: i32,
}

impl Command for FilledOval {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x_center, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y_center, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x_rad, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.fill_ellipse(self.x_center, self.y_center, 0, 360, self.x_rad, self.y_rad);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|o{}{}{}{}",
            to_base_36(2, self.x_center),
            to_base_36(2, self.y_center),
            to_base_36(2, self.x_rad),
            to_base_36(2, self.y_rad)
        )
    }
}

#[derive(Default, Clone)]
pub struct Arc {
    pub x: i32,
    pub y: i32,
    pub start_ang: i32,
    pub end_ang: i32,
    pub radius: i32,
}

impl Command for Arc {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.start_ang, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.end_ang, ch)?;
                Ok(true)
            }

            8 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(true)
            }

            9 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.arc(self.x, self.y, self.start_ang, self.end_ang, self.radius);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|A{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.start_ang),
            to_base_36(2, self.end_ang),
            to_base_36(2, self.radius)
        )
    }
}

#[derive(Default, Clone)]
pub struct OvalArc {
    pub x: i32,
    pub y: i32,
    pub start_ang: i32,
    pub end_ang: i32,
    pub x_rad: i32,
    pub y_rad: i32,
}

impl Command for OvalArc {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.start_ang, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.end_ang, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.x_rad, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.ellipse(self.x, self.y, self.start_ang, self.end_ang, self.x_rad, self.y_rad);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|V{}{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.start_ang),
            to_base_36(2, self.end_ang),
            to_base_36(2, self.x_rad),
            to_base_36(2, self.y_rad)
        )
    }
}

#[derive(Default, Clone)]
pub struct PieSlice {
    pub x: i32,
    pub y: i32,
    pub start_ang: i32,
    pub end_ang: i32,
    pub radius: i32,
}

impl Command for PieSlice {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.start_ang, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.end_ang, ch)?;
                Ok(true)
            }

            8 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(true)
            }

            9 => {
                parse_base_36(&mut self.radius, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.pie_slice(self.x, self.y, self.start_ang, self.end_ang, self.radius);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|I{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.start_ang),
            to_base_36(2, self.end_ang),
            to_base_36(2, self.radius)
        )
    }
}

#[derive(Default, Clone)]
pub struct OvalPieSlice {
    pub x: i32,
    pub y: i32,
    pub st_ang: i32,
    pub end_ang: i32,
    pub x_rad: i32,
    pub y_rad: i32,
}

impl Command for OvalPieSlice {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.st_ang, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.end_ang, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.x_rad, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.y_rad, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.sector(self.x, self.y, self.st_ang, self.end_ang, self.x_rad, self.y_rad);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|i{}{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.st_ang),
            to_base_36(2, self.end_ang),
            to_base_36(2, self.x_rad),
            to_base_36(2, self.y_rad)
        )
    }
}

#[derive(Default, Clone)]
pub struct Bezier {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub x3: i32,
    pub y3: i32,
    pub x4: i32,
    pub y4: i32,
    pub cnt: i32,
}

impl Command for Bezier {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x2, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.y2, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.x3, ch)?;
                Ok(true)
            }

            10 | 11 => {
                parse_base_36(&mut self.y3, ch)?;
                Ok(true)
            }

            12 | 13 => {
                parse_base_36(&mut self.x4, ch)?;
                Ok(true)
            }

            14 | 15 => {
                parse_base_36(&mut self.y4, ch)?;
                Ok(true)
            }

            16 => {
                parse_base_36(&mut self.cnt, ch)?;
                Ok(true)
            }

            17 => {
                parse_base_36(&mut self.cnt, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        // bgi.rip_bezier(self.x1, self.y1, self.x2, self.y2, self.x3, self.y3, self.x4, self.y4, self.cnt);

        let points = vec![
            Position::new(self.x1, self.y1),
            Position::new(self.x2, self.y2),
            Position::new(self.x3, self.y3),
            Position::new(self.x4, self.y4),
        ];
        bgi.draw_bezier(points.len() as i32, &points, self.cnt);

        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|Z{}{}{}{}{}{}{}{}{}",
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(2, self.x2),
            to_base_36(2, self.y2),
            to_base_36(2, self.x3),
            to_base_36(2, self.y3),
            to_base_36(2, self.x4),
            to_base_36(2, self.y4),
            to_base_36(2, self.cnt)
        )
    }
}

#[derive(Default, Clone)]
pub struct Polygon {
    pub points: Vec<i32>,
    pub npoints: i32,
}

impl Command for Polygon {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.npoints, ch)?;
                Ok(true)
            }
            _ => {
                if *state % 2 == 0 {
                    self.points.push(0);
                }
                let mut p = self.points.pop().unwrap();
                parse_base_36(&mut p, ch)?;
                self.points.push(p);

                Ok(*state < (self.npoints + 1) * 4)
            }
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        let mut points = Vec::new();
        for i in 0..self.points.len() / 2 {
            points.push(Position::new(self.points[i as usize * 2], self.points[i as usize * 2 + 1]));
        }
        bgi.draw_poly(&points);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        let mut res = String::from("|P");
        res.push_str(to_base_36(2, self.points.len() as i32 / 2).as_str());
        for p in &self.points {
            res.push_str(to_base_36(2, *p).as_str());
        }
        res
    }
}

#[derive(Default, Clone)]
pub struct FilledPolygon {
    pub points: Vec<i32>,
    pub npoints: i32,
}

impl Command for FilledPolygon {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.npoints, ch)?;
                Ok(true)
            }
            _ => {
                if *state % 2 == 0 {
                    self.points.push(0);
                }
                let mut p = self.points.pop().unwrap();
                parse_base_36(&mut p, ch)?;
                self.points.push(p);

                Ok(*state < (self.npoints + 1) * 4)
            }
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        let mut points = Vec::new();
        for i in 0..self.points.len() / 2 {
            points.push(Position::new(self.points[i as usize * 2], self.points[i as usize * 2 + 1]));
        }
        bgi.fill_poly(&points);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        let mut res = String::from("|p");
        res.push_str(to_base_36(2, self.points.len() as i32 / 2).as_str());
        for p in &self.points {
            res.push_str(to_base_36(2, *p).as_str());
        }
        res
    }
}

#[derive(Default, Clone)]
pub struct PolyLine {
    pub points: Vec<i32>,
    pub npoints: i32,
}

impl Command for PolyLine {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.npoints, ch)?;
                Ok(true)
            }
            _ => {
                if *state % 2 == 0 {
                    self.points.push(0);
                }
                let mut p = self.points.pop().unwrap();
                parse_base_36(&mut p, ch)?;
                self.points.push(p);

                Ok(*state < (self.npoints + 1) * 4)
            }
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        let mut points = Vec::new();
        for i in 0..self.points.len() / 2 {
            points.push(Position::new(self.points[i * 2], self.points[i * 2 + 1]));
        }
        bgi.move_to(points[0].x, points[0].y);
        for p in points.iter().skip(1) {
            bgi.line_to(p.x, p.y);
        }
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        let mut res = String::from("|l");
        res.push_str(to_base_36(2, self.points.len() as i32 / 2).as_str());
        for p in &self.points {
            res.push_str(to_base_36(2, *p).as_str());
        }
        res
    }
}

#[derive(Default, Clone)]
pub struct Fill {
    pub x: i32,
    pub y: i32,
    pub border: i32,
}

impl Command for Fill {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 => {
                parse_base_36(&mut self.border, ch)?;
                Ok(true)
            }

            5 => {
                parse_base_36(&mut self.border, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.flood_fill(self.x, self.y, self.border as u8);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|F{}{}{}", to_base_36(2, self.x), to_base_36(2, self.y), to_base_36(2, self.border))
    }
}

#[derive(Default, Clone)]
pub struct LineStyle {
    pub style: i32,
    pub user_pat: i32,
    pub thick: i32,
}

impl Command for LineStyle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.style, ch)?;
                Ok(true)
            }
            2..=5 => {
                parse_base_36(&mut self.user_pat, ch)?;
                Ok(true)
            }
            6 => {
                parse_base_36(&mut self.thick, ch)?;
                Ok(true)
            }

            7 => {
                parse_base_36(&mut self.thick, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_line_style(super::bgi::LineStyle::from(self.style as u8));
        //  If the <style> parameter is not 4, then the <user_pat> parameter is ignored.
        if self.style == 4 {
            bgi.set_line_pattern(self.user_pat);
        }
        bgi.set_line_thickness(self.thick);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|={}{}{}", to_base_36(2, self.style), to_base_36(4, self.user_pat), to_base_36(2, self.thick))
    }
}

#[derive(Default, Clone)]
pub struct FillStyle {
    pub pattern: i32,
    pub color: i32,
}

impl Command for FillStyle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.pattern, ch)?;
                Ok(true)
            }
            2 => {
                parse_base_36(&mut self.color, ch)?;
                Ok(true)
            }
            3 => {
                parse_base_36(&mut self.color, ch)?;
                Ok(false)
            }
            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_fill_style(super::bgi::FillStyle::from(self.pattern as u8));
        bgi.set_fill_color(self.color as u8);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!("|S{}{}", to_base_36(2, self.pattern), to_base_36(2, self.color))
    }
}

#[derive(Default, Clone)]
pub struct FillPattern {
    pub c1: i32,
    pub c2: i32,
    pub c3: i32,
    pub c4: i32,
    pub c5: i32,
    pub c6: i32,
    pub c7: i32,
    pub c8: i32,
    pub col: i32,
}

impl Command for FillPattern {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.c1, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.c2, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.c3, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.c4, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.c5, ch)?;
                Ok(true)
            }

            10 | 11 => {
                parse_base_36(&mut self.c6, ch)?;
                Ok(true)
            }

            12 | 13 => {
                parse_base_36(&mut self.c7, ch)?;
                Ok(true)
            }

            14 | 15 => {
                parse_base_36(&mut self.c8, ch)?;
                Ok(true)
            }

            16 => {
                parse_base_36(&mut self.col, ch)?;
                Ok(true)
            }

            17 => {
                parse_base_36(&mut self.col, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.set_user_fill_pattern(&[
            self.c1 as u8,
            self.c2 as u8,
            self.c3 as u8,
            self.c4 as u8,
            self.c5 as u8,
            self.c6 as u8,
            self.c7 as u8,
            self.c8 as u8,
        ]);
        bgi.set_fill_style(super::bgi::FillStyle::User);
        bgi.set_fill_color(self.col as u8);
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|s{}{}{}{}{}{}{}{}{}",
            to_base_36(2, self.c1),
            to_base_36(2, self.c2),
            to_base_36(2, self.c3),
            to_base_36(2, self.c4),
            to_base_36(2, self.c5),
            to_base_36(2, self.c6),
            to_base_36(2, self.c7),
            to_base_36(2, self.c8),
            to_base_36(2, self.col)
        )
    }
}

#[derive(Default, Clone)]
pub struct Mouse {
    pub num: i32,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub clk: i32,
    pub clr: i32,
    pub res: i32,
    pub text: String,
}

impl Command for Mouse {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.num, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.clk, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.clr, ch)?;
                Ok(true)
            }

            12..=16 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            _ => {
                self.text.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1M{}{}{}{}{}{}{}{}{}",
            to_base_36(2, self.num),
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(1, self.clk),
            to_base_36(1, self.clr),
            to_base_36(5, self.res),
            self.text
        )
    }
}

#[derive(Default, Clone)]
pub struct MouseFields {}

impl Command for MouseFields {
    fn to_rip_string(&self) -> String {
        "|1K".to_string()
    }
}

#[derive(Default, Clone)]
pub struct BeginText {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub res: i32,
}

impl Command for BeginText {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x2, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.y2, ch)?;
                Ok(true)
            }

            8 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            9 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        // Nothing?
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1T{}{}{}{}{}",
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(2, self.x2),
            to_base_36(2, self.y2),
            to_base_36(2, self.res)
        )
    }
}

#[derive(Default, Clone)]
pub struct RegionText {
    pub justify: bool,
    pub str: String,
}

impl Command for RegionText {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        if *state == 0 {
            self.justify = ch == '1';
        } else {
            self.str.push(ch);
        }
        Ok(true)
    }

    fn to_rip_string(&self) -> String {
        format!("|1t{}{}", i32::from(self.justify), self.str)
    }
}

#[derive(Default, Clone)]
pub struct EndText {}

impl Command for EndText {
    fn to_rip_string(&self) -> String {
        "|1E".to_string()
    }
    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        // Nothing
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct GetImage {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub res: i32,
}

impl Command for GetImage {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            8 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.rip_image = Some(bgi.get_image(self.x0, self.y0, self.x1, self.y1));
        Ok(())
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1C{}{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(1, self.res)
        )
    }
}

#[derive(Default, Clone)]
pub struct PutImage {
    pub x: i32,
    pub y: i32,
    pub mode: i32,
    pub res: i32,
}

impl Command for PutImage {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }

            6 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn run(&self, bgi: &mut Bgi) -> EngineResult<()> {
        bgi.put_rip_image(self.x, self.y, super::bgi::WriteMode::from(self.mode as u8));
        Ok(())
    }
    fn to_rip_string(&self) -> String {
        format!(
            "|1P{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.mode),
            to_base_36(1, self.res)
        )
    }
}

#[derive(Default, Clone)]
pub struct WriteIcon {
    pub res: char,
    pub str: String,
}

impl Command for WriteIcon {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        if *state == 0 {
            self.res = ch;
        } else {
            self.str.push(ch);
        }
        Ok(true)
    }

    fn to_rip_string(&self) -> String {
        format!("|1W{}{}", self.res, self.str)
    }
}

#[derive(Default, Clone)]
pub struct LoadIcon {
    pub x: i32,
    pub y: i32,
    pub mode: i32,
    pub clipboard: i32,
    pub res: i32,
    pub file_name: String,
}

impl Command for LoadIcon {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }

            6 => {
                parse_base_36(&mut self.clipboard, ch)?;
                Ok(true)
            }

            7 | 8 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }
            _ => {
                self.file_name.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1I{}{}{}{}{}{}",
            to_base_36(2, self.x),
            to_base_36(2, self.y),
            to_base_36(2, self.mode),
            to_base_36(1, self.clipboard),
            to_base_36(2, self.res),
            self.file_name
        )
    }
}

#[derive(Default, Clone)]
pub struct ButtonStyle {
    pub wid: i32,
    pub hgt: i32,
    pub orient: i32,
    pub flags: i32,
    pub size: i32,
    pub dfore: i32,
    pub dback: i32,
    pub bright: i32,
    pub dark: i32,

    pub surface: i32,
    pub grp_no: i32,
    pub flags2: i32,
    pub uline_col: i32,
    pub corner_col: i32,
    pub res: i32,
}

impl Command for ButtonStyle {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.wid, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.hgt, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.orient, ch)?;
                Ok(true)
            }

            6..=9 => {
                parse_base_36(&mut self.flags, ch)?;
                Ok(true)
            }

            10 | 11 => {
                parse_base_36(&mut self.size, ch)?;
                Ok(true)
            }

            12 | 13 => {
                parse_base_36(&mut self.dfore, ch)?;
                Ok(true)
            }

            14 | 15 => {
                parse_base_36(&mut self.dback, ch)?;
                Ok(true)
            }

            16 | 17 => {
                parse_base_36(&mut self.bright, ch)?;
                Ok(true)
            }

            18 | 19 => {
                parse_base_36(&mut self.dark, ch)?;
                Ok(true)
            }

            20 | 21 => {
                parse_base_36(&mut self.surface, ch)?;
                Ok(true)
            }

            22 | 23 => {
                parse_base_36(&mut self.grp_no, ch)?;
                Ok(true)
            }

            24 | 25 => {
                parse_base_36(&mut self.flags2, ch)?;
                Ok(true)
            }

            26 | 27 => {
                parse_base_36(&mut self.uline_col, ch)?;
                Ok(true)
            }

            28 | 29 => {
                parse_base_36(&mut self.corner_col, ch)?;
                Ok(true)
            }
            30..=36 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(*state < 36)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1B{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            to_base_36(2, self.wid),
            to_base_36(2, self.hgt),
            to_base_36(2, self.orient),
            to_base_36(4, self.flags),
            to_base_36(2, self.size),
            to_base_36(2, self.dfore),
            to_base_36(2, self.dback),
            to_base_36(2, self.bright),
            to_base_36(2, self.dark),
            to_base_36(2, self.surface),
            to_base_36(2, self.grp_no),
            to_base_36(2, self.flags2),
            to_base_36(2, self.uline_col),
            to_base_36(2, self.corner_col),
            to_base_36(6, self.res)
        )
    }
}

#[derive(Default, Clone)]
pub struct Button {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub hotkey: i32,
    pub flags: i32,
    pub res: i32,
    pub text: String,
}

impl Command for Button {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.hotkey, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.flags, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            _ => {
                self.text.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1U{}{}{}{}{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(2, self.hotkey),
            to_base_36(1, self.flags),
            to_base_36(1, self.res),
            self.text
        )
    }
}

#[derive(Default, Clone)]
pub struct Define {
    pub flags: i32,
    pub res: i32,
    pub text: String,
}

impl Command for Define {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0..=2 => {
                parse_base_36(&mut self.flags, ch)?;
                Ok(true)
            }
            3 | 4 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }
            _ => {
                self.text.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!("|1D{}{}{}", to_base_36(3, self.flags), to_base_36(2, self.res), self.text)
    }
}

#[derive(Default, Clone)]
pub struct Query {
    pub mode: i32,
    pub res: i32,
    pub text: String,
}

impl Command for Query {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }
            1..=3 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }
            _ => {
                self.text.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!("|1\x1B{}{}{}", to_base_36(1, self.mode), to_base_36(3, self.res), self.text)
    }
}

#[derive(Default, Clone)]
pub struct CopyRegion {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub res: i32,
    pub dest_line: i32,
}

impl Command for CopyRegion {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.x0, ch)?;
                Ok(true)
            }
            2 | 3 => {
                parse_base_36(&mut self.y0, ch)?;
                Ok(true)
            }

            4 | 5 => {
                parse_base_36(&mut self.x1, ch)?;
                Ok(true)
            }

            6 | 7 => {
                parse_base_36(&mut self.y1, ch)?;
                Ok(true)
            }

            8 | 9 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            10 => {
                parse_base_36(&mut self.dest_line, ch)?;
                Ok(true)
            }

            11 => {
                parse_base_36(&mut self.dest_line, ch)?;
                Ok(false)
            }

            _ => Err(anyhow::Error::msg("Invalid state")),
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|1G{}{}{}{}{}{}",
            to_base_36(2, self.x0),
            to_base_36(2, self.y0),
            to_base_36(2, self.x1),
            to_base_36(2, self.y1),
            to_base_36(2, self.res),
            to_base_36(2, self.dest_line)
        )
    }
}

#[derive(Default, Clone)]
pub struct ReadScene {
    pub res: String,
    pub str: String,
}

impl Command for ReadScene {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        if (0..=7).contains(state) {
            self.res.push(ch);
        } else {
            self.str.push(ch);
        }
        Ok(true)
    }

    fn to_rip_string(&self) -> String {
        format!("|1R{}{}", self.res, self.str)
    }
}
#[derive(Default, Clone)]
pub struct FileQuery {
    pub mode: i32,
    pub res: i32,
    pub file_name: String,
}

impl Command for FileQuery {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 | 1 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }
            2..=5 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }
            _ => {
                self.file_name.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!("|1F{}{}{}", to_base_36(2, self.mode), to_base_36(4, self.res), self.file_name)
    }
}

#[derive(Default, Clone)]
pub struct EnterBlockMode {
    pub mode: i32,
    pub proto: i32,
    pub file_type: i32,
    pub res: i32,
    pub file_name: String,
}

impl Command for EnterBlockMode {
    fn parse(&mut self, state: &mut i32, ch: char) -> EngineResult<bool> {
        match state {
            0 => {
                parse_base_36(&mut self.mode, ch)?;
                Ok(true)
            }
            1 => {
                parse_base_36(&mut self.proto, ch)?;
                Ok(true)
            }

            2 | 3 => {
                parse_base_36(&mut self.file_type, ch)?;
                Ok(true)
            }

            4..=7 => {
                parse_base_36(&mut self.res, ch)?;
                Ok(true)
            }

            _ => {
                self.file_name.push(ch);
                Ok(true)
            }
        }
    }

    fn to_rip_string(&self) -> String {
        format!(
            "|9\x1B{}{}{}{}{}",
            to_base_36(1, self.mode),
            to_base_36(1, self.proto),
            to_base_36(2, self.file_type),
            to_base_36(4, self.res),
            self.file_name
        )
    }
}
