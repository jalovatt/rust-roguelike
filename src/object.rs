use tcod::colors::*;
use tcod::console::*;

#[derive(Debug)]
pub struct Object {
  pub x: i32,
  pub y: i32,
  pub char: char,
  pub color: Color,
  pub name: String,
  pub blocks: bool,
  pub alive: bool,
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, blocks: bool) -> Self {
    Object { x, y, char, color, blocks, name: name.into(), alive: false }
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.x = x;
    self.y = y;
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.x, self.y)
  }

  pub fn draw(&self, con: &mut dyn Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }
}