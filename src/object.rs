use tcod::colors::*;
use tcod::console::*;

use crate::fighter::Fighter;
use crate::ai::Ai;

#[derive(Debug)]
pub struct Object {
  pub x: i32,
  pub y: i32,
  pub char: char,
  pub color: Color,
  pub name: String,
  pub blocks: bool,
  pub alive: bool,
  pub fighter: Option<Fighter>,
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

  pub fn take_damage(&mut self, damage: i32) {
    let mut fighter = self.fighter.as_mut().unwrap();
    let mut hp = fighter.hp - damage;
    if hp < 0 { hp = 0; }

    fighter.hp = hp;

    println!("{} has {} hit points", self.name, fighter.hp);

    if hp == 0 {
      self.alive = false;
      fighter.on_death.callback(self);
    }
  }

  pub fn draw(&self, con: &mut dyn Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }
}
