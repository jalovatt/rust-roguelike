use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

mod tile;
mod map;
mod object;
mod game;
mod rect;
mod fighter;
mod ai;

use game::Game;
use map::Map;

static SCREEN_WIDTH: i32 = 80;
static SCREEN_HEIGHT: i32 = 50;

static MAP_WIDTH: i32 = 80;
static MAP_HEIGHT: i32 = 45;

static ROOM_MAX_SIZE: i32 = 10;
static ROOM_MIN_SIZE: i32 = 6;

static MAX_ROOMS: i32 = 30;

static FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
static FOV_LIGHT_WALLS: bool = true;
static TORCH_RADIUS: i32 = 10;

static COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
static COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };
static COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150, };
static COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50, };

static LIMIT_FPS: i32 = 20;

static MAX_ROOM_MONSTERS: i32 = 3;

static PLAYER: usize = 0;

struct Tcod {
  root: Root,
  con: Offscreen,
  fov: FovMap,
}

fn render_all(tcod: &mut Tcod, game: &mut Game, fov_recompute: bool) {
  if fov_recompute {
    let player = &game.objects[PLAYER];
    tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }

  let mut to_draw: Vec<_> = game.objects
    .iter()
    .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
    .collect();

  to_draw.sort_by(|o1, o2| { o1.blocks.cmp(&o2.blocks)});

  for object in &to_draw {
    if tcod.fov.is_in_fov(object.x, object.y) {
      object.draw(&mut tcod.con);
    }
  }

  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      let visible = tcod.fov.is_in_fov(x, y);

      let explored = &mut game.map.tiles[x as usize][y as usize].explored;

      if visible { *explored = true; }
      if !*explored { continue; }

      let wall = game.map.tiles[x as usize][y as usize].block_sight;

      let color = match (visible, wall) {
        (false, true) => COLOR_DARK_WALL,
        (false, false) => COLOR_DARK_GROUND,
        (true, true) => COLOR_LIGHT_WALL,
        (true, false) => COLOR_LIGHT_GROUND,
      };

      tcod.con.set_char_background(x, y, color, BackgroundFlag::Set);
    }
  }

  blit(
    &tcod.con,
    (0, 0),
    (MAP_WIDTH, MAP_HEIGHT),
    &mut tcod.root,
    (0, 0),
    1.0,
    1.0,
  );

  tcod.root.set_default_foreground(WHITE);
  if let Some(fighter) = game.objects[PLAYER].fighter {
    tcod.root.print_ex(
      1,
      SCREEN_HEIGHT - 2,
      BackgroundFlag::None,
      TextAlignment::Left,
      format!("HP: {}/{} ", fighter.hp, fighter.max_hp),
    );
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
  TookTurn,
  DidntTakeTurn,
  Exit,
}

#[allow(clippy::ptr_arg)]
fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> PlayerAction {
  use tcod::input::Key;
  use tcod::input::KeyCode::*;

  let key = tcod.root.wait_for_keypress(true);
  let player_alive = game.objects[PLAYER].alive;

  match (key, key.text(), player_alive) {
    ( Key { code: Enter, alt: true, .. }, _, _ ) => {
      let fullscreen = tcod.root.is_fullscreen();
      tcod.root.set_fullscreen(!fullscreen);

      return PlayerAction::DidntTakeTurn
    },
    ( Key { code: Escape, .. }, _, _ ) => return PlayerAction::Exit,
    ( Key { code: Up, .. }, _, true ) => game.player_move_or_attack(0, -1),
    ( Key { code: Down, .. }, _, true ) => game.player_move_or_attack(0, 1),
    ( Key { code: Left, .. }, _, true ) => game.player_move_or_attack(-1, 0),
    ( Key { code: Right, .. }, _, true ) => game.player_move_or_attack(1, 0),
    _ => return PlayerAction::DidntTakeTurn,
  }

  PlayerAction::TookTurn
}

fn set_fov_map(fov: &mut FovMap, map: &Map) {
  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      fov.set(
        x,
        y,
        !map.tiles[x as usize][y as usize].block_sight,
        !map.tiles[x as usize][y as usize].blocked,
      );
    }
  }
}

fn main() {
  let root = Root::initializer()
    .font("arial10x10.png", FontLayout::Tcod)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("Rust/libtcod tutorial")
    .init();

  let mut tcod = Tcod {
    root,
    con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
    fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
  };

  tcod::system::set_fps(LIMIT_FPS);

  let mut game = Game::new();
  set_fov_map(&mut tcod.fov, &game.map);

  let mut previous_player_position = (-1, -1);

  while !tcod.root.window_closed() {
    tcod.con.clear();

    let fov_recompute = previous_player_position != (game.objects[PLAYER].pos());

    render_all(&mut tcod, &mut game, fov_recompute);
    tcod.root.flush();

    let player = &mut game.objects[0];
    previous_player_position = (player.x, player.y);

    let action = handle_keys(&mut tcod, &mut game);
    if action == PlayerAction::Exit { break; }

    if game.objects[PLAYER].alive && action == PlayerAction::TookTurn {
      game.update_objects(&tcod.fov);
    }
  }
}
