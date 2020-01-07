#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
}

impl Fighter {
  pub fn take_damage(&mut self, damage: i32) -> bool {
    let hp = self.hp - damage;
    self.hp = if hp < 0 { 0 } else { hp };

    self.hp > 0
  }
}
