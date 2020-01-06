use crate::object::Object;
use crate::game::Game;
use crate::messages::Messages;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
  Player,
  Monster,
}

impl DeathCallback {
  pub fn callback(self, object: &mut Object, messages: &mut Messages) {
    let callback: fn(&mut Object, &mut Messages) = match self {
      DeathCallback::Player => Game::player_death,
      DeathCallback::Monster => Game::monster_death,
    };
    callback(object, messages);
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
