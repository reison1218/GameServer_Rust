use log::error;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;

///回合行为类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ActorType {
    None = 0,
}

impl Default for ActorType {
    fn default() -> Self {
        ActorType::None
    }
}

impl ActorType {
    pub fn into_u32(self) -> u32 {
        let value: u32 = self.into();
        value
    }
}

#[derive(Clone)]
pub struct ActorData {}

unsafe impl Sync for ActorData {}

unsafe impl Send for ActorData {}

#[derive(Default, Clone)]
pub struct ActorSystem<F: Actor> {
    actor_map: HashMap<ActorType, F>,
}

impl<F: Actor> ActorSystem<F> {
    pub fn init() {
        let mut actor_system: ActorSystem<F> = ActorSystem {
            actor_map: HashMap::new(),
        };
        for actor in actor_system.actor_map.values_mut() {
            actor.run();
        }
    }
}

pub trait Actor {
    fn receive(&self) -> anyhow::Result<ActorData> {
        let rec = self.get_rec();
        let res = rec.recv();
        match res {
            Err(e) => anyhow::bail!("{:?}", e),
            Ok(data) => Ok(data),
        }
    }

    fn run(&mut self) {
        loop {
            let res = self.receive();
            if let Err(e) = res {
                error!("{:?}", e);
                continue;
            }
            let res = res.unwrap();
            self.action(res);
        }
    }
    fn get_rec(&self) -> &crossbeam::crossbeam_channel::Receiver<ActorData>;
    fn action(&mut self, t: ActorData);
}
