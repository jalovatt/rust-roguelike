use std::cmp;
use rand::Rng;

use crate::tile::Tile;
use crate::object::Object;

use super::*;

#[derive(Clone, Copy, Debug)]
struct Rect {
  x1: i32,
  y1: i32,
  x2: i32,
  y2: i32,
}

impl Rect {
  pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
    Rect { x1: x, y1: y, x2: x + w, y2: y + h }
  }

  pub fn center(&self) -> (i32, i32) {
    let center_x = (self.x1 + self.x2) / 2;
    let center_y = (self.y1 + self.y2) / 2;

    (center_x, center_y)
  }

  pub fn intersects_with(&self, other: &Rect) -> bool {
    (self.x1 <= other.x2)
      && (self.x2 >= other.x1)
      && (self.y1 <= other.y2)
      && (self.y2 >= other.y1)
  }
}

pub type Map = Vec<Vec<Tile>>;

fn create_room(room: Rect, map: &mut Map) {
  for x in (room.x1 + 1)..room.x2 {
    for y in (room.y1 + 1)..room.y2 {
      map[x as usize][y as usize] = Tile::empty();
    }
  }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
  for x in cmp::min(x1, x2)..=(cmp::max(x1, x2)) {
    map[x as usize][y as usize] = Tile::empty();
  }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
  for y in cmp::min(y1, y2)..=(cmp::max(y1, y2)) {
    map[x as usize][y as usize] = Tile::empty();
  }
}

pub fn make_map(player: &mut Object) -> Map {
  let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

  let mut rooms = vec![];

  for _ in 0..MAX_ROOMS {
    let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
    let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
    let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
    let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

    let new_room = Rect::new(x, y, w, h);

    let failed = rooms.iter().any(|other| new_room.intersects_with(other));

    if failed { continue; }

    create_room(new_room, &mut map);

    let (new_x, new_y) = new_room.center();

    if rooms.is_empty() {
      player.x = new_x;
      player.y = new_y;
    } else {
      let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

      if rand::random() {
        create_h_tunnel(prev_x, new_x, prev_y, &mut map);
        create_v_tunnel(prev_y, new_y, new_x, &mut map);
      } else {
        create_v_tunnel(prev_y, new_y, prev_x, &mut map);
        create_h_tunnel(prev_x, new_x, new_y, &mut map);
      }
    }

    rooms.push(new_room);
  }

  map
}
