use ::tcod::colors::*;
use ::tcod::console::*;

use crate::constants::*;
use crate::tcod::Tcod;
use crate::game::Game;

static PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

static BAR_WIDTH: i32 = 20;

static MSG_X: i32 = BAR_WIDTH + 2;
static MSG_WIDTH: i32 = SCREEN_WIDTH - MSG_X;
static MSG_HEIGHT: i32 = PANEL_HEIGHT - 1;

static COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
static COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };
static COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150, };
static COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50, };

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

fn render_objects(tcod: &mut Tcod, game: &Game) {
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
}

fn render_map(tcod: &mut Tcod, game: &mut Game) {
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
}

fn render_mouselook(tcod: &mut Tcod, names_under_mouse: String) {
  tcod.panel.set_default_foreground(LIGHT_GREY);
  tcod.panel.print_ex(
    1,
    0,
    BackgroundFlag::None,
    TextAlignment::Left,
    names_under_mouse,
  );
}

fn render_messages(tcod: &mut Tcod, game: &Game) {
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
}

pub fn render_all(tcod: &mut Tcod, game: &mut Game, names_under_mouse: String) {
  tcod.con.clear();

  render_objects(tcod, game);
  render_map(tcod, game);

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

  let (hp, max_hp) = game.objects[PLAYER].fighter.map_or((0, 0), |(f, _)| (f.hp, f.max_hp));
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
  render_mouselook(tcod, names_under_mouse);
  render_messages(tcod, game);

  blit(
    &tcod.panel,
    (0, 0),
    (SCREEN_WIDTH, PANEL_HEIGHT),
    &mut tcod.root,
    (0, PANEL_Y),
    1.0,
    1.0,
  );

  tcod.root.flush();
}
