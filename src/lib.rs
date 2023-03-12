use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub struct DevcadeControls<'w> {
  gamepads: Res<'w, Gamepads>,
  button_inputs: Res<'w, Input<GamepadButton>>,
  axes: Res<'w, Axis<GamepadAxis>>,
  keyboard_input: Res<'w, Input<KeyCode>>,
}

#[derive(Clone, Copy)]
pub enum Buttons {
  Menu,
  A4,
  A3,
  B1,
  B2,
  A1,
  A2,
  StickLeft,
  B3,
  B4,
  StickUp,
  StickDown,
  StickRight,
}

impl TryFrom<&Buttons> for GamepadButtonType {
  type Error = ();
  fn try_from(value: &Buttons) -> Result<Self, Self::Error> {
    match value {
      Buttons::Menu => Ok(GamepadButtonType::Start),
      Buttons::A1 => Ok(GamepadButtonType::West),
      Buttons::A2 => Ok(GamepadButtonType::North),
      Buttons::A3 => Ok(GamepadButtonType::RightTrigger),
      Buttons::A4 => Ok(GamepadButtonType::LeftTrigger),
      Buttons::B1 => Ok(GamepadButtonType::South),
      Buttons::B2 => Ok(GamepadButtonType::East),
      Buttons::B3 => Ok(GamepadButtonType::RightTrigger2),
      Buttons::B4 => Ok(GamepadButtonType::LeftTrigger2),
      _ => Err(()),
    }
  }
}

enum AxisConfig {
  Positive(GamepadAxisType),
  Negative(GamepadAxisType),
}

impl AxisConfig {
  fn get_axis(&self) -> GamepadAxisType {
    *match self {
      AxisConfig::Positive(axis_type) => axis_type,
      AxisConfig::Negative(axis_type) => axis_type,
    }
  }
}

impl TryFrom<&Buttons> for AxisConfig {
  type Error = ();
  fn try_from(value: &Buttons) -> Result<Self, Self::Error> {
    match value {
      Buttons::StickUp => Ok(AxisConfig::Positive(GamepadAxisType::LeftStickY)),
      Buttons::StickDown => Ok(AxisConfig::Negative(GamepadAxisType::LeftStickY)),
      Buttons::StickRight => Ok(AxisConfig::Positive(GamepadAxisType::LeftStickX)),
      Buttons::StickLeft => Ok(AxisConfig::Negative(GamepadAxisType::LeftStickX)),
      _ => Err(()),
    }
  }
}

pub struct PlayerButton {
  player: Player,
  button: Buttons,
}

impl From<PlayerButton> for KeyCode {
  fn from(value: PlayerButton) -> KeyCode {
    match (value.player, value.button) {
      (Player::P1, Buttons::A1) => KeyCode::Q,
      (Player::P1, Buttons::A2) => KeyCode::W,
      (Player::P1, Buttons::A3) => KeyCode::E,
      (Player::P1, Buttons::A4) => KeyCode::R,
      (Player::P1, Buttons::B1) => KeyCode::A,
      (Player::P1, Buttons::B2) => KeyCode::S,
      (Player::P1, Buttons::B3) => KeyCode::D,
      (Player::P1, Buttons::B4) => KeyCode::F,
      (Player::P1, Buttons::Menu) => KeyCode::Escape,
      (Player::P1, Buttons::StickUp) => KeyCode::G,
      (Player::P1, Buttons::StickDown) => KeyCode::B,
      (Player::P1, Buttons::StickLeft) => KeyCode::V,
      (Player::P1, Buttons::StickRight) => KeyCode::N,

      (Player::P2, Buttons::A1) => KeyCode::Q,
      (Player::P2, Buttons::A2) => KeyCode::W,
      (Player::P2, Buttons::A3) => KeyCode::E,
      (Player::P2, Buttons::A4) => KeyCode::R,
      (Player::P2, Buttons::B1) => KeyCode::A,
      (Player::P2, Buttons::B2) => KeyCode::S,
      (Player::P2, Buttons::B3) => KeyCode::D,
      (Player::P2, Buttons::B4) => KeyCode::F,
      (Player::P2, Buttons::Menu) => KeyCode::Escape,
      (Player::P2, Buttons::StickUp) => KeyCode::Up,
      (Player::P2, Buttons::StickDown) => KeyCode::Down,
      (Player::P2, Buttons::StickLeft) => KeyCode::Left,
      (Player::P2, Buttons::StickRight) => KeyCode::Right,
    }
  }
}

pub enum Player {
  P1,
  P2,
}

impl Player {
  fn index(&self) -> usize {
    match self {
      Self::P1 => 0,
      Self::P2 => 1,
    }
  }
}

impl<'w> DevcadeControls<'w> {
  /// True if either player pressed this button
  /// See [`DevcadeControls::pressed`]
  pub fn any_player_pressed(&self, button: Buttons) -> bool {
    self.pressed(button, Player::P1) || self.pressed(button, Player::P2)
  }
  fn gamepad_for_player(&self, player: &Player) -> Option<Gamepad> {
    self.gamepads.iter().nth(player.index())
  }
  /// Returns true if the button is pressed by the given player
  /// Uses keyboard if no controller is plugged in.
  /// See source for [`PlayerButton`] for more detailed mappings
  pub fn pressed(&self, button: Buttons, player: Player) -> bool {
    if let Some(gamepad) = self.gamepad_for_player(&player) {
      if let Ok(button) = GamepadButtonType::try_from(&button) {
        self
          .button_inputs
          .pressed(GamepadButton::new(gamepad, button))
      } else {
        let axis_config = AxisConfig::try_from(&button).unwrap();
        let value = self
          .axes
          .get(GamepadAxis::new(gamepad, axis_config.get_axis()))
          .unwrap();
        match axis_config {
          AxisConfig::Positive(_) => value > 0.0,
          AxisConfig::Negative(_) => value < 0.0,
        }
      }
    } else {
      self
        .keyboard_input
        .pressed(KeyCode::from(PlayerButton { button, player }))
    }
  }
}
