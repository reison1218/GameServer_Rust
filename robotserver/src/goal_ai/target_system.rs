use crate::goal_ai::cter::Cter;

///目标系统
#[derive(Default)]
pub struct TargetingSystem {
    current_target: u32,
}

impl TargetingSystem {
    pub fn update(&self) {}

    pub fn get_target(&self) -> u32 {
        0
    }
}
