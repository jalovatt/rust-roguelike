use rand::Rng;
use tcod::{colors, colors::*};

use crate::object::Object;
use crate::map::Map;

use super::*;

pub struct Game {
  pub map: map::Map,
  pub objects: Vec<Object>,
}

impl Game {
  pub fn new() -> Game {
    let map = Map::new();
    let objects = Vec::new();

    let mut game = Game { map, objects };
    game.create_objects();

    game
  }

  #[allow(clippy::ptr_arg)]
  fn is_blocked(&self, x: i32, y: i32) -> bool {
    if self.map.tiles[x as usize][y as usize].blocked { return true; }

    self.objects.iter().any(|obj| obj.blocks && obj.pos() == (x, y))
  }

  fn create_objects(&mut self) {
    let (x, y) = self.map.rooms[0].center();
    self.objects.push(Object::new(x, y, '@', WHITE, "player", true));
    self.objects[PLAYER].alive = true;

    for room in self.map.rooms.iter() {
      let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

      for _ in 0..num_monsters {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        if self.is_blocked(x, y) { continue; }

        let mut monster = if rand::random::<f32>() < 0.8 {
          Object::new(x, y, 'o', colors::DESATURATED_GREEN, "orc", true)
        } else {
          Object::new(x, y, 'T', colors::DARKER_GREEN, "something", true)
        };

        monster.alive = true;

        self.objects.push(monster);
      }
    }
  }

  #[allow(clippy::ptr_arg)]
  pub fn move_by(&mut self, id: usize, dx: i32, dy: i32) {
    let (x, y) = self.objects[id].pos();

    if !self.is_blocked(x + dx, y + dy) {
      self.objects[id].set_pos(x + dx, y + dy);
    }
  }

  pub fn player_move_or_attack(&mut self, dx: i32, dy: i32) {
    let x = self.objects[PLAYER].x + dx;
    let y = self.objects[PLAYER].y + dy;

    let target_id = self.objects.iter().position(|obj| obj.pos() == (x, y));
    match target_id {
      Some(target_id) => {
        println!(
          "The {} laughs at your puny efforts to attack it!",
          self.objects[target_id].name
        );
      }
      None => self.move_by(PLAYER, dx, dy)
    }
  }

  pub fn update_objects(&mut self) {
    for (id, object) in self.objects.iter().enumerate() {
      if id == PLAYER { continue; }

      println!("The {} growls!", object.name);
    }
  }
}
