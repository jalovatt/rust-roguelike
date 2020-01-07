use crate::game::Game;
use crate::object::Object;
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
