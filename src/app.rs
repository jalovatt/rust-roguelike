use ::tcod::colors::*;
use ::tcod::console::*;
use ::tcod::input::{self, Event, Key};
use ::tcod::map::{FovAlgorithm, Map as FovMap};

use crate::constants::*;
use crate::game::Game;
use crate::map::Map;
use crate::object::Object;
use crate::render::{render_game, render_menu};
use crate::tcod::Tcod;

static FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
static FOV_LIGHT_WALLS: bool = true;
static TORCH_RADIUS: i32 = 10;

static INVENTORY_WIDTH: i32 = 50;

static LIMIT_FPS: i32 = 20;

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

fn show_inventory(tcod: &mut Tcod, inventory: &[Object], header: &str) -> Option<usize> {
  let options = if inventory.is_empty() {
    vec!["Inventory is empty.".into()]
  } else {
    inventory.iter().map(|item| item.name.clone()).collect()
  };

  render_menu(tcod, header, &options, INVENTORY_WIDTH);
  let key = tcod.root.wait_for_keypress(true);

  if key.printable.is_alphabetic() {
    let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
    if index < options.len() {
      return Some(index);
    }
  }

  None
}

#[allow(clippy::ptr_arg)]
fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> PlayerAction {
  use ::tcod::input::KeyCode::*;

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
    ( Key { code: Text, .. }, "g", true ) => {
      let item_id = game.objects
        .iter()
        .position(|obj| obj.pos() == game.objects[PLAYER].pos() && obj.item.is_some());

      if let Some(item_id) = item_id {
        game.pick_item_up(item_id);
      }

      return PlayerAction::DidntTakeTurn
    },
    ( Key { code: Text, .. }, "i", true ) => {
      if let Some(inventory_id) = show_inventory(
        tcod,
        &game.inventory,
        "Press the key listed next to an item to use it, or any other key to cancel.\n"
      ) {
        game.use_item(inventory_id);
      }

      return PlayerAction::DidntTakeTurn
    },
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

pub fn play() {
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

  ::tcod::system::set_fps(LIMIT_FPS);

  let mut game = Game::new();
  set_fov_map(&mut tcod.fov, &game.map);

  let mut previous_player_position = (-1, -1);

  game.messages.add(
    "Welcome stranger! Prepare to perish in the Tombs of the Ancient Kings.",
    RED,
  );

  while !tcod.root.window_closed() {
    let fov_recompute = previous_player_position != (game.objects[PLAYER].pos());

    if fov_recompute {
      let player = &game.objects[PLAYER];
      tcod.fov.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
      Some((_, Event::Mouse(m))) => tcod.mouse = m,
      Some((_, Event::Key(k))) => tcod.key = k,
      _ => tcod.key = Default::default(),
    }

    let names_under_mouse = get_names_under_mouse(&tcod, &game);

    render_game(&mut tcod, &mut game, names_under_mouse);

    let player = &mut game.objects[0];
    previous_player_position = (player.x, player.y);

    let action = handle_keys(&mut tcod, &mut game);
    if action == PlayerAction::Exit { break; }

    if game.objects[PLAYER].alive && action == PlayerAction::TookTurn {
      game.update_objects(&tcod.fov);
    }
  }
}
