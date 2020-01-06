use crate::object::Object;
use crate::game::Game;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
  Player,
  Monster,
}

impl DeathCallback {
  pub fn callback(self, object: &mut Object) {
    let callback: fn(&mut Object) = match self {
      DeathCallback::Player => Game::player_death,
      DeathCallback::Monster => Game::monster_death,
    };
    callback(object);
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
  pub on_death: DeathCallback,
}
