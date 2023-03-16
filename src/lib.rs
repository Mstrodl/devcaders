//! Library for Rusty Devcade games using bevy!
//!
//! # Input Handling
//! See [The example for `DevcadeControls`](DevcadeControls#examples)
use bevy::ecs::system::{SystemMeta, SystemParam};
use bevy::prelude::*;
use enum_iterator::Sequence;

#[derive(SystemParam)]
struct DevcadeControlsInner<'w> {
  gamepads: Res<'w, Gamepads>,
  button_inputs: Res<'w, Input<GamepadButton>>,
  axes: Res<'w, Axis<GamepadAxis>>,
  keyboard_input: Res<'w, Input<KeyCode>>,
}

/// [`SystemParam`] for devcade's control buttons
///
/// # Examples
/// Usage is simple, just add it as a parameter to one of your [`System`](bevy::ecs::system::System)s!
/// ```
/// use devcaders::{Button, Player, DevcadeControls};
///
/// fn input_system(button_inputs: DevcadeControls) {
///   // User is actively pressing Menu button
///   if button_inputs.pressed(Player::P1, Button::Menu) {
///     std::process::exit(0);
///   }
///   let mut x_vector = 0;
///   // User pressed StickRight button
///   if button_inputs.just_pressed(Player::P1, Button::StickRight) {
///     x_vector += 1;
///   }
///   // User released StickLeft button
///   if button_inputs.just_released(Player::P1, Button::StickLeft) {
///     x_vector -= 1;
///   }
/// }
/// ```
pub struct DevcadeControls {
  p1: PlayerControlState,
  p2: PlayerControlState,
}
#[derive(Default, Clone)]
struct ButtonState {
  pressed: bool,
  changed_this_frame: bool,
}
#[derive(Default, Clone)]
struct PlayerControlState {
  stick_up: ButtonState,
  stick_down: ButtonState,
  stick_left: ButtonState,
  stick_right: ButtonState,
  menu: ButtonState,
  a1: ButtonState,
  a2: ButtonState,
  a3: ButtonState,
  a4: ButtonState,
  b1: ButtonState,
  b2: ButtonState,
  b3: ButtonState,
  b4: ButtonState,
}

impl PlayerControlState {
  fn get_state_for(&self, button: Button) -> &ButtonState {
    match button {
      Button::StickUp => &self.stick_up,
      Button::StickDown => &self.stick_down,
      Button::StickLeft => &self.stick_left,
      Button::StickRight => &self.stick_right,
      Button::A1 => &self.a1,
      Button::A2 => &self.a2,
      Button::A3 => &self.a3,
      Button::A4 => &self.a4,
      Button::B1 => &self.b1,
      Button::B2 => &self.b2,
      Button::B3 => &self.b3,
      Button::B4 => &self.b4,
      Button::Menu => &self.menu,
    }
  }

  fn get_state_for_mut(&mut self, button: Button) -> &mut ButtonState {
    match button {
      Button::StickUp => &mut self.stick_up,
      Button::StickDown => &mut self.stick_down,
      Button::StickLeft => &mut self.stick_left,
      Button::StickRight => &mut self.stick_right,
      Button::A1 => &mut self.a1,
      Button::A2 => &mut self.a2,
      Button::A3 => &mut self.a3,
      Button::A4 => &mut self.a4,
      Button::B1 => &mut self.b1,
      Button::B2 => &mut self.b2,
      Button::B3 => &mut self.b3,
      Button::B4 => &mut self.b4,
      Button::Menu => &mut self.menu,
    }
  }
}

/// Underlying state of [`DevcadeControls`]
pub struct ControlState<'w> {
  p1: PlayerControlState,
  p2: PlayerControlState,
  inner: <DevcadeControlsInner<'w> as SystemParam>::State,
}

unsafe impl SystemParam for DevcadeControls {
  type State = ControlState<'static>;
  type Item<'w, 's> = DevcadeControls;
  fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
    Self::State {
      inner: DevcadeControlsInner::init_state(world, system_meta),
      p1: PlayerControlState::default(),
      p2: PlayerControlState::default(),
    }
  }
  unsafe fn get_param<'w, 's>(
    state: &'s mut Self::State,
    system_meta: &SystemMeta,
    world: &'w World,
    change_tick: u32,
  ) -> Self::Item<'w, 's> {
    let inner = DevcadeControlsInner::get_param(&mut state.inner, system_meta, world, change_tick);
    for player in enum_iterator::all::<Player>() {
      let player_state = match player {
        Player::P1 => &mut state.p1,
        Player::P2 => &mut state.p2,
      };
      for button in enum_iterator::all::<Button>() {
        let button_state = player_state.get_state_for_mut(button);
        let pressed = inner.pressed(button, player);
        button_state.changed_this_frame = pressed != button_state.pressed;
        button_state.pressed = pressed;
      }
    }
    DevcadeControls {
      p1: state.p1.clone(),
      p2: state.p2.clone(),
    }
  }
}

impl DevcadeControls {
  fn get_player(&self, player: Player) -> &PlayerControlState {
    match player {
      Player::P1 => &self.p1,
      Player::P2 => &self.p2,
    }
  }

  /// Returns true when button began being pressed on this frame, false otherwise
  pub fn just_pressed(&self, player: Player, button: Button) -> bool {
    let player = self.get_player(player);
    let button_state = player.get_state_for(button);
    button_state.pressed && button_state.changed_this_frame
  }
  /// Returns true when button began being unpressed on this frame, false otherwise
  pub fn just_released(&self, player: Player, button: Button) -> bool {
    let player = self.get_player(player);
    let button_state = player.get_state_for(button);
    !button_state.pressed && button_state.changed_this_frame
  }
  /// Returns true if the button is currently pressed
  pub fn pressed(&self, player: Player, button: Button) -> bool {
    self.get_player(player).get_state_for(button).pressed
  }
}

#[derive(Clone, Copy, Sequence, PartialEq, Eq)]
/// Gamepad buttons
pub enum Button {
  /// Top row, first button. Red
  A1,
  /// Top row, second button. Blue
  A2,
  /// Top row, third button. Green
  A3,
  /// Top row, fourth button. White
  A4,

  /// Second row, first button.
  B1,
  /// Second row, second button.
  B2,
  /// Second row, third button.
  B3,
  /// Second row, third button.
  B4,

  /// Center button. Black. Generally bound to pause or exit
  Menu,

  /// Joystick pointing left
  StickLeft,
  /// Joystick pointing up
  StickUp,
  /// Joystick pointing down
  StickDown,
  /// Joystick pointing right
  StickRight,
}

impl TryFrom<&Button> for GamepadButtonType {
  type Error = ();
  fn try_from(value: &Button) -> Result<Self, Self::Error> {
    match value {
      Button::Menu => Ok(GamepadButtonType::Start),
      Button::A1 => Ok(GamepadButtonType::West),
      Button::A2 => Ok(GamepadButtonType::North),
      Button::A3 => Ok(GamepadButtonType::RightTrigger),
      Button::A4 => Ok(GamepadButtonType::LeftTrigger),
      Button::B1 => Ok(GamepadButtonType::South),
      Button::B2 => Ok(GamepadButtonType::East),
      Button::B3 => Ok(GamepadButtonType::RightTrigger2),
      Button::B4 => Ok(GamepadButtonType::LeftTrigger2),
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

impl TryFrom<&Button> for AxisConfig {
  type Error = ();
  fn try_from(value: &Button) -> Result<Self, Self::Error> {
    match value {
      Button::StickUp => Ok(AxisConfig::Positive(GamepadAxisType::LeftStickY)),
      Button::StickDown => Ok(AxisConfig::Negative(GamepadAxisType::LeftStickY)),
      Button::StickRight => Ok(AxisConfig::Positive(GamepadAxisType::LeftStickX)),
      Button::StickLeft => Ok(AxisConfig::Negative(GamepadAxisType::LeftStickX)),
      _ => Err(()),
    }
  }
}

/// Internal. Tuple of [`Player`] and [`Button`]
pub struct PlayerButton {
  player: Player,
  button: Button,
}

impl From<PlayerButton> for KeyCode {
  fn from(value: PlayerButton) -> KeyCode {
    match (value.player, value.button) {
      (Player::P1, Button::A1) => KeyCode::Q,
      (Player::P1, Button::A2) => KeyCode::W,
      (Player::P1, Button::A3) => KeyCode::E,
      (Player::P1, Button::A4) => KeyCode::R,
      (Player::P1, Button::B1) => KeyCode::A,
      (Player::P1, Button::B2) => KeyCode::S,
      (Player::P1, Button::B3) => KeyCode::D,
      (Player::P1, Button::B4) => KeyCode::F,
      (Player::P1, Button::Menu) => KeyCode::Escape,
      (Player::P1, Button::StickUp) => KeyCode::G,
      (Player::P1, Button::StickDown) => KeyCode::B,
      (Player::P1, Button::StickLeft) => KeyCode::V,
      (Player::P1, Button::StickRight) => KeyCode::N,

      (Player::P2, Button::A1) => KeyCode::Y,
      (Player::P2, Button::A2) => KeyCode::U,
      (Player::P2, Button::A3) => KeyCode::I,
      (Player::P2, Button::A4) => KeyCode::O,
      (Player::P2, Button::B1) => KeyCode::H,
      (Player::P2, Button::B2) => KeyCode::J,
      (Player::P2, Button::B3) => KeyCode::K,
      (Player::P2, Button::B4) => KeyCode::L,
      (Player::P2, Button::Menu) => KeyCode::Escape,
      (Player::P2, Button::StickUp) => KeyCode::Up,
      (Player::P2, Button::StickDown) => KeyCode::Down,
      (Player::P2, Button::StickLeft) => KeyCode::Left,
      (Player::P2, Button::StickRight) => KeyCode::Right,
    }
  }
}

#[derive(Clone, Copy, Sequence, PartialEq, Eq, Component)]
/// Used to specify which player's controls to query
pub enum Player {
  /// First player, left set of controls
  P1,
  /// Second player, right set of controls
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

impl<'w> DevcadeControlsInner<'w> {
  fn gamepad_for_player(&self, player: &Player) -> Option<Gamepad> {
    self.gamepads.iter().nth(player.index())
  }
  /// Returns true if the button is pressed by the given player
  /// Uses keyboard if no controller is plugged in.
  /// See source for [`PlayerButton`] for more detailed mappings
  pub fn pressed(&self, button: Button, player: Player) -> bool {
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

/// Close the focused window when both menu buttons are pressed.
pub fn close_on_menu_buttons(
  mut commands: Commands,
  focused_windows: Query<(Entity, &Window)>,
  input: DevcadeControls,
) {
  for (window, focus) in focused_windows.iter() {
    if !focus.focused {
      continue;
    }
    if input.pressed(Player::P1, Button::Menu) && input.pressed(Player::P2, Button::Menu) {
      commands.entity(window).despawn();
    }
  }
}
