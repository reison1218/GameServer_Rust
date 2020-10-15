use crate::goal_ai::cter::Cter;

pub struct TargetingSystem {
    owner_cter: &'static Cter,
    current_target: &'static Cter,
}

impl TargetingSystem {
    pub fn update(&self) {}

    pub fn get_target(&self) -> &'static Cter {
        self.current_target
    }
}
