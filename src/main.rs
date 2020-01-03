use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

mod tile;
mod map;
mod object;
mod game;

use map as Map;
use object::Object;
use game::Game;

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

struct Tcod {
  root: Root,
  con: Offscreen,
  fov: FovMap,
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
  if fov_recompute {
    let player = &objects[0];
    tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
  }

  for object in objects {
    if tcod.fov.is_in_fov(object.x, object.y) {
      object.draw(&mut tcod.con);
    }
  }

  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      let visible = tcod.fov.is_in_fov(x, y);

      let explored = &mut game.map[x as usize][y as usize].explored;

      if visible { *explored = true; }
      if !*explored { continue; }

      let wall = game.map[x as usize][y as usize].block_sight;

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
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
  use tcod::input::Key;
  use tcod::input::KeyCode::*;

  let key = tcod.root.wait_for_keypress(true);
  match key {
    Key { code: Enter, alt: true, .. } => {
      let fullscreen = tcod.root.is_fullscreen();
      tcod.root.set_fullscreen(!fullscreen);
    },
    Key { code: Escape, .. } => return true,
    Key { code: Up, .. } => player.move_by(0, -1, game),
    Key { code: Down, .. } => player.move_by(0, 1, game),
    Key { code: Left, .. } => player.move_by(-1, 0, game),
    Key { code: Right, .. } => player.move_by(1, 0, game),
    _ => {}
  }

  false
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

  let player = Object::new(0, 0, '@', WHITE);
  let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

  let mut objects = [player, npc];

  let mut game = Game {
    map: Map::make_map(&mut objects[0]),
  };

  for y in 0..MAP_HEIGHT {
    for x in 0..MAP_WIDTH {
      tcod.fov.set(
        x,
        y,
        !game.map[x as usize][y as usize].block_sight,
        !game.map[x as usize][y as usize].blocked,
      );
    }
  }

  let mut previous_player_position = (-1, -1);

  while !tcod.root.window_closed() {
    tcod.con.clear();

    let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
    render_all(&mut tcod, &mut game, &objects, fov_recompute);

    tcod.root.flush();

    let player = &mut objects[0];

    previous_player_position = (player.x, player.y);

    let exit = handle_keys(&mut tcod, &game, player);
    if exit { break; }
  }
}
