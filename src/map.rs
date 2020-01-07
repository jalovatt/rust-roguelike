use std::cmp;
use rand::Rng;

use crate::constants::*;
use crate::tile::Tile;
use crate::rect::Rect;

static ROOM_MAX_SIZE: i32 = 10;
static ROOM_MIN_SIZE: i32 = 6;

pub struct Map {
  pub tiles: Vec<Vec<Tile>>,
  pub rooms: Vec<Rect>,
}

impl Map {
  fn create_room(&mut self, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
      for y in (room.y1 + 1)..room.y2 {
        self.tiles[x as usize][y as usize] = Tile::empty();
      }
    }
  }

  fn create_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=(cmp::max(x1, x2)) {
      self.tiles[x as usize][y as usize] = Tile::empty();
    }
  }

  fn create_v_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
    for y in cmp::min(y1, y2)..=(cmp::max(y1, y2)) {
      self.tiles[x as usize][y as usize] = Tile::empty();
    }
  }

  pub fn new() -> Map {
    let mut map = Map {
      tiles: vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize],
      rooms: vec![],
    };

    for _ in 0..MAX_ROOMS {
      let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
      let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
      let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
      let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

      let new_room = Rect::new(x, y, w, h);

      let failed = map.rooms.iter().any(|other| new_room.intersects_with(other));

      if failed { continue; }

      map.create_room(new_room);

      let (new_x, new_y) = new_room.center();

      if !map.rooms.is_empty() {
        let (prev_x, prev_y) = map.rooms.last().unwrap().center();

        if rand::random() {
          map.create_h_tunnel(prev_x, new_x, prev_y);
          map.create_v_tunnel(prev_y, new_y, new_x);
        } else {
          map.create_v_tunnel(prev_y, new_y, prev_x);
          map.create_h_tunnel(prev_x, new_x, new_y);
        }
      }

      map.rooms.push(new_room);
    }

    map
  }
}
