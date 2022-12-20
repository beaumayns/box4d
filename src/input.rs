use winit::event::{ButtonId, VirtualKeyCode};

#[derive(Debug, Default, Copy, Clone)]
pub struct InputState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub ana: bool,
    pub kata: bool,

    pub roll_left: bool,
    pub roll_right: bool,
    pub yaw: f32,
    pub pitch: f32,

    pub hyperlook: bool,

    pub grab: bool,
    pub center_grab: bool,
}

impl InputState {
    pub fn new_events(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
    }

    pub fn keyboard_event(&mut self, key_code: VirtualKeyCode, pressed: bool) {
        match key_code {
            VirtualKeyCode::W => self.forward = pressed,
            VirtualKeyCode::S => self.back = pressed,
            VirtualKeyCode::A => self.left = pressed,
            VirtualKeyCode::D => self.right = pressed,
            VirtualKeyCode::Space => self.up = pressed,
            VirtualKeyCode::C => self.down = pressed,
            VirtualKeyCode::R => self.ana = pressed,
            VirtualKeyCode::F => self.kata = pressed,

            VirtualKeyCode::Q => self.roll_left = pressed,
            VirtualKeyCode::E => self.roll_right = pressed,

            VirtualKeyCode::LShift => self.hyperlook = pressed,

            _ => {}
        }
    }

    pub fn mouse_moved(&mut self, x: f64, y: f64) {
        self.yaw = x as f32;
        self.pitch = y as f32;
    }

    pub fn mouse_click(&mut self, button: ButtonId, pressed: bool) {
        if button == 0 {
            self.grab = pressed;
        }
    }
}
