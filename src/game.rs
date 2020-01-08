use rand::Rng;
use ::tcod::map::{Map as FovMap};
use ::tcod::{colors, colors::*};

use crate::constants::*;
use crate::messages::Messages;
use crate::object::Object;
use crate::fighter::Fighter;
use crate::death::Death;
use crate::ai::Ai;
use crate::map::Map;

fn mut_two<T>(first: usize, second: usize, items: &mut [T]) -> (&mut T, &mut T) {
  assert!(first != second);

  let split_at_index = std::cmp::max(first, second);
  let (first_slice, second_slice) = items.split_at_mut(split_at_index);

  if first < second {
    (&mut first_slice[first], &mut second_slice[0])
  } else {
    (&mut second_slice[0], &mut first_slice[second])
  }
}


fn player_death(player: &mut Object, messages: &mut Messages) {
  messages.add("You died!", RED);

  player.char = '%';
  player.color = DARK_RED;
}

fn monster_death(monster: &mut Object, messages: &mut Messages) {
  messages.add(format!("{} died!", monster.name), ORANGE);

  monster.char = '%';
  monster.color = DARK_RED;
  monster.blocks = false;
  monster.fighter = None;
  monster.ai = None;
  monster.name = format!("remains of {}", monster.name);
}

pub struct Game {
  pub map: Map,
  pub objects: Vec<Object>,
  pub messages: Messages,
}

impl Game {
  pub fn new() -> Game {
    let map = Map::new();
    let objects = Vec::new();
    let messages = Messages::new();

    let mut game = Game { map, objects, messages };
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

    let mut player = Object::new(x, y, '@', WHITE, "player", true);

    player.alive = true;
    player.fighter = Some((
      Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
      },
      Death::Player
    ));

    self.objects.push(player);

    for room in self.map.rooms.iter() {
      let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

      for _ in 0..num_monsters {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        if self.is_blocked(x, y) { continue; }

        let monster_type = if rand::random::<f32>() < 0.8 { "orc" } else { "troll" };

        let mut monster;

        if monster_type == "orc" {
          monster = Object::new(x, y, 'o', colors::DESATURATED_GREEN, "orc", true);
          monster.fighter = Some((
            Fighter {
              max_hp: 10,
              hp: 10,
              defense: 0,
              power: 3,
            },
            Death::Monster,
          ));
          monster.ai = Some(Ai::Basic);
        } else {
          monster = Object::new(x, y, 'T', colors::DARKER_GREEN, "troll", true);
          monster.fighter = Some((
            Fighter {
              max_hp: 16,
              hp: 16,
              defense: 1,
              power: 4,
            },
            Death::Monster,
          ));
          monster.ai = Some(Ai::Basic);
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

    let target_id = self.objects.iter().position(|obj| obj.pos() == (x, y) && obj.fighter.is_some());
    match target_id {
      Some(target_id) => {
        self.attack(PLAYER, target_id);
      }
      None => self.move_by(PLAYER, dx, dy)
    }
  }

  fn move_towards(&mut self, id: usize, other_id: usize) {
    let this = &self.objects[id];
    let target = &self.objects[other_id];

    let dx = target.x - this.x;
    let dy = target.y - this.y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    let nx = (dx as f32 / distance).round() as i32;
    let ny = (dy as f32 / distance).round() as i32;

    self.move_by(id, nx, ny);
  }

  pub fn attack(&mut self, id: usize, other_id: usize) {
    let (source, target) = mut_two(id, other_id, &mut self.objects);

    let damage = source.fighter.unwrap().0.power - target.fighter.unwrap().0.defense;
    if damage > 0 {
      self.messages.add(format!("{} attacks {} for {} hit points", source.name, target.name, damage), WHITE);
      if !target.take_damage(damage) {
        match target.fighter.unwrap().1 {
          Death::Player => player_death(target, &mut self.messages),
          Death::Monster => monster_death(target, &mut self.messages),
        }
      }
    } else {
      self.messages.add(format!("{} attacks {} but it has no effect", source.name, target.name), WHITE);
    }
  }

  fn ai_turn(&mut self, id: usize, fov_map: &FovMap) {
    let (ai_x, ai_y) = self.objects[id].pos();

    if fov_map.is_in_fov(ai_x, ai_y) {
      if self.objects[id].distance_to(&self.objects[PLAYER]) >= 2.0 {
        self.move_towards(id, PLAYER);
      } else if self.objects[id].fighter.is_some()
        && self.objects[PLAYER].fighter.map_or(false, |f| f.0.hp > 0) {
        self.attack(id, PLAYER);
      }
    }
  }

  pub fn update_objects(&mut self, fov_map: &FovMap) {
    for id in 0..self.objects.len() {
      if id == PLAYER { continue; }

      if self.objects[id].alive && self.objects[id].ai.is_some() {
        self.ai_turn(id, fov_map);
      }
    }
  }
}
