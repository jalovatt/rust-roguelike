use tcod::colors::*;
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use tcod::map::{FovAlgorithm, Map as FovMap};

mod messages;
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

static PANEL_HEIGHT: i32 = 7;
static PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

static BAR_WIDTH: i32 = 20;

static MSG_X: i32 = BAR_WIDTH + 2;
static MSG_WIDTH: i32 = SCREEN_WIDTH - MSG_X;
static MSG_HEIGHT: i32 = PANEL_HEIGHT - 1;

static MAP_WIDTH: i32 = 80;
static MAP_HEIGHT: i32 = 43;

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
  panel: Offscreen,
  fov: FovMap,
  key: Key,
  mouse: Mouse,
}

#[allow(clippy::too_many_arguments)]
fn render_bar(
  panel: &mut Offscreen,
  x: i32,
  y: i32,
  total_width: i32,
  name: &str,
  value: i32,
  maximum: i32,
  bar_color: Color,
  back_color: Color,
) {
  let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

  panel.set_default_background(back_color);
  panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

  panel.set_default_background(bar_color);
  if bar_width > 0 {
    panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
  }

  panel.set_default_foreground(WHITE);
  panel.print_ex(
    x + total_width / 2,
    y,
    BackgroundFlag::None,
    TextAlignment::Center,
    format!("{}: {}/{}", name, value, maximum),
  );
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

  tcod.panel.set_default_background(BLACK);
  tcod.panel.clear();

  let (hp, max_hp) = game.objects[PLAYER].fighter.map_or((0, 0), |f| (f.hp, f.max_hp));

  render_bar(
    &mut tcod.panel,
    1,
    1,
    BAR_WIDTH,
    "HP",
    hp,
    max_hp,
    LIGHT_RED,
    DARK_RED,
  );

  tcod.panel.set_default_foreground(LIGHT_GREY);
  tcod.panel.print_ex(
    1,
    0,
    BackgroundFlag::None,
    TextAlignment::Left,
    get_names_under_mouse(tcod, game),
  );

  let mut y = MSG_HEIGHT;
  for &(ref msg, color) in game.messages.iter().rev() {
    let height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    y -= height;

    if y < 0 {
      break;
    }

    tcod.panel.set_default_foreground(color);
    tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
  }

  blit(
    &tcod.panel,
    (0, 0),
    (SCREEN_WIDTH, PANEL_HEIGHT),
    &mut tcod.root,
    (0, PANEL_Y),
    1.0,
    1.0,
  );

}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
  TookTurn,
  DidntTakeTurn,
  Exit,
}

fn get_names_under_mouse(tcod: &Tcod, game: &Game) -> String {
  let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

  let names = game.objects
    .iter()
    .filter(|obj| obj.pos() == (x, y) && tcod.fov.is_in_fov(obj.x, obj.y))
    .map(|obj| obj.name.clone())
    .collect::<Vec<_>>();

  names.join(", ")
}

#[allow(clippy::ptr_arg)]
fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> PlayerAction {
  use tcod::input::KeyCode::*;

  let player_alive = game.objects[PLAYER].alive;

  match (tcod.key, tcod.key.text(), player_alive) {
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
    panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
    fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    key: Default::default(),
    mouse: Default::default(),
  };

  tcod::system::set_fps(LIMIT_FPS);

  let mut game = Game::new();
  set_fov_map(&mut tcod.fov, &game.map);

  let mut previous_player_position = (-1, -1);

  game.messages.add(
    "Welcome stranger! Prepare to perish in the Tombs of the Ancient Kings.",
    RED,
  );

  while !tcod.root.window_closed() {
    tcod.con.clear();

    let fov_recompute = previous_player_position != (game.objects[PLAYER].pos());

    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
      Some((_, Event::Mouse(m))) => tcod.mouse = m,
      Some((_, Event::Key(k))) => tcod.key = k,
      _ => tcod.key = Default::default(),
    }

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
