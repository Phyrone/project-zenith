use std::time::{Duration, Instant};

use bevy::app::{App, ScheduleRunnerPlugin};
use bevy::prelude::*;

use crate::transport::local::LocalTransportPlugin;

mod transport;

pub const TARGET_TPS: u16 = 40;


#[derive(Debug, Component)]
pub struct WorldController {}

impl WorldController {}

#[derive(Debug)]
pub struct WorldServer {
    pub app: App,
}

impl Default for WorldServer {
    fn default() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin {
            run_mode: bevy::app::RunMode::Loop {
                wait: Some(Duration::from_secs_f32(1.0 / TARGET_TPS as f32)),
            },
        })).add_plugins((LocalTransportPlugin));

        WorldServer { app }
    }
}

impl WorldServer {
    pub async fn run(&mut self) {
        loop {
            let tick_time = self.tick().await;
            Self::await_next_tick(tick_time).await;
        }
    }

    async fn tick(&mut self) -> Duration {
        let time = Instant::now();
        self.app.update();
        
        
        time.elapsed()
    }
    async fn await_next_tick(tick_time: Duration) {
        let target = Duration::from_secs_f32(1.0 / TARGET_TPS as f32);
        if tick_time < target {
            tokio::time::sleep(target - tick_time).await;
        }
    }
}

