use tcod::colors::*;
use tcod::console::*;

use crate::fighter::Fighter;
use crate::ai::Ai;

use crate::death::Death;

#[derive(Debug)]
pub struct Object {
  pub x: i32,
  pub y: i32,
  pub char: char,
  pub color: Color,
  pub name: String,
  pub blocks: bool,
  pub alive: bool,
  pub fighter: Option<(Fighter, Death)>,
  pub ai: Option<Ai>,
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, blocks: bool) -> Self {
    Object { x, y, char, color, blocks, name: name.into(), alive: false, fighter: None, ai: None }
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.x = x;
    self.y = y;
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.x, self.y)
  }

  pub fn distance_to(&self, other: &Object) -> f32 {
    let dx = other.x - self.x;
    let dy = other.y - self.y;
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
  }

  pub fn take_damage(&mut self, damage: i32) -> bool {
    if let Some((fighter, _)) = self.fighter.as_mut() {
      let alive = fighter.take_damage(damage);

      if !alive {
        self.alive = false;

        return false;
      }

      return true;
    }

    true
  }

  pub fn draw(&self, con: &mut dyn Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }
}
