#![no_std]
use asr::{
    Process,
    future::next_tick,
    game_engine::unity::il2cpp::{Image, Module, UnityPointer},
    settings::{Gui, gui::Title},
    timer::{self, TimerState},
    watcher::Watcher,
};
use bytemuck::{Pod, Zeroable};

asr::async_main!(stable);
asr::panic_handler!();

const PROCESS_NAME: &str = "sonic_pico_park.exe";

#[derive(Gui)]
struct Settings {
    /// General settings
    _general: Title,
    /// Auto start
    #[default = true]
    start: bool,
    /// Split settings
    split: Title,
    /// Level 1-1
    #[default = true]
    level1_1: bool,
    /// Level 1-2
    #[default = true]
    level1_2: bool,
    /// Level 1-3
    #[default = true]
    level1_3: bool,
    /// Level 1-4
    #[default = true]
    level1_4: bool,
    /// Level 2-1
    #[default = true]
    level2_1: bool,
    /// Level 2-2
    #[default = true]
    level2_2: bool,
    /// Level 2-3
    #[default = true]
    level2_3: bool,
    /// Level 2-4
    #[default = true]
    level2_4: bool,
    /// Level 3-1
    #[default = true]
    level3_1: bool,
    /// Level 3-2
    #[default = true]
    level3_2: bool,
    /// Level 3-3
    #[default = true]
    level3_3: bool,
    /// Level 3-4
    #[default = true]
    level3_4: bool,
    /// Level 4-1
    #[default = true]
    level4_1: bool,
    /// Level 4-2
    #[default = true]
    level4_2: bool,
    /// Level 4-3
    #[default = true]
    level4_3: bool,
    /// Level 4-4
    #[default = true]
    level4_4: bool,
    /// Level 5-1
    #[default = true]
    level5_1: bool,
    /// Level 5-2
    #[default = true]
    level5_2: bool,
    /// Level 5-3
    #[default = true]
    level5_3: bool,
    /// Level 5-4
    #[default = true]
    level5_4: bool,
    /// Level 6-1
    #[default = true]
    level6_1: bool,
    /// Level 6-2
    #[default = true]
    level6_2: bool,
    /// Level 6-3
    #[default = true]
    level6_3: bool,
    /// Level 6-4
    #[default = true]
    level6_4: bool,
    /// Level 7-1
    #[default = true]
    level7_1: bool,
    /// Level 7-2
    #[default = true]
    level7_2: bool,
    /// Level 7-3
    #[default = true]
    level7_3: bool,
    /// Level 7-4
    #[default = true]
    level7_4: bool,
    /// Level 8-1
    #[default = true]
    level8_1: bool,
    /// Level 8-2
    #[default = true]
    level8_2: bool,
    /// Level 8-3
    #[default = true]
    level8_3: bool,
    /// Level 8-4
    #[default = true]
    level8_4: bool,
    /// Level 9-1
    #[default = true]
    level9_1: bool,
    /// Level 9-2
    #[default = true]
    level9_2: bool,
    /// Level 9-3
    #[default = true]
    level9_3: bool,
    /// Level 9-4
    #[default = true]
    level9_4: bool,
    /// Level 10-1
    #[default = true]
    level10_1: bool,
    /// Level 10-2
    #[default = true]
    level10_2: bool,
    /// Level 10-3
    #[default = true]
    level10_3: bool,
    /// Level 10-4
    #[default = true]
    level10_4: bool,
    /// Level 11-1
    #[default = true]
    level11_1: bool,
    /// Level 11-2
    #[default = true]
    level11_2: bool,
    /// Level 11-3
    #[default = true]
    level11_3: bool,
    /// Level 11-4
    #[default = true]
    level11_4: bool,
    /// Level 12-1
    #[default = true]
    level12_1: bool,
    /// Level 12-2
    #[default = true]
    level12_2: bool,
    /// Level 12-3
    #[default = true]
    level12_3: bool,
    /// Level 12-4
    #[default = true]
    level12_4: bool,
    /// Level 13-1
    #[default = true]
    level13_1: bool,
    /// Level 13-2
    #[default = true]
    level13_2: bool,
    /// Level 13-3
    #[default = true]
    level13_3: bool,
    /// Level 13-4
    #[default = true]
    level13_4: bool,
    /// Level 14-1
    #[default = true]
    level14_1: bool,
    /// Level 14-2
    #[default = true]
    level14_2: bool,
    /// Level 14-3
    #[default = true]
    level14_3: bool,
    /// Level 14-4
    #[default = true]
    level14_4: bool,
    /// Level 15-1
    #[default = true]
    level15_1: bool,
    /// Level 15-2
    #[default = true]
    level15_2: bool,
    /// Level 15-3
    #[default = true]
    level15_3: bool,
    /// Level 15-4
    #[default = true]
    level15_4: bool,
}

async fn main() {
    let mut settings = Settings::register();

    loop {
        let process = Process::wait_attach(PROCESS_NAME).await;

        process
            .until_closes(async {
                let mut watchers = Watchers::default();
                let memory = Memory::init(&process).await;

                loop {
                    settings.update();
                    update_loop(&process, &memory, &mut watchers);
                    let timer_state = timer::state();
                    if timer_state == TimerState::Running || timer_state == TimerState::Paused {
                        if reset(&watchers, &settings) {
                            timer::reset()
                        } else if split(&watchers, &settings) {
                            timer::split()
                        }
                    }

                    if timer::state() == TimerState::NotRunning && start(&watchers, &settings) {
                        timer::start();
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(Default)]
struct Watchers {
    game_mode: Watcher<u8>,
    stage: Watcher<u32>,
    stage_result: Watcher<u8>,
}

struct Memory {
    il2cpp_module: Module,
    game_assembly: Image,
    stage_scene_manager: UnityPointer<2>,
}

impl Memory {
    async fn init(process: &Process) -> Self {
        let il2cpp_module = Module::wait_attach_auto_detect(process).await;
        let game_assembly = il2cpp_module
            .wait_get_image(process, "Assembly-CSharp")
            .await;
        let stage_scene_manager =
            UnityPointer::<2>::new("StageSceneManager", 3, &["instance", "m_GameMode"]);

        Self {
            il2cpp_module,
            game_assembly,
            stage_scene_manager,
        }
    }
}

fn update_loop(process: &Process, memory: &Memory, watchers: &mut Watchers) {
    match memory.stage_scene_manager.deref::<StageSceneManager>(
        process,
        &memory.il2cpp_module,
        &memory.game_assembly,
    ) {
        Ok(data) => {
            watchers.game_mode.update_infallible(data.game_mode as u8);
            watchers
                .stage
                .update_infallible(data.stage_parent_no * 100 + data.stage_child_no);
            watchers
                .stage_result
                .update_infallible(data.stage_result as u8);
        }
        _ => {
            watchers.game_mode.update_infallible(0);
            watchers.stage.update_infallible(0);
            watchers.stage_result.update_infallible(0);
        }
    };
}

fn start(watchers: &Watchers, settings: &Settings) -> bool {
    settings.start
        && watchers.game_mode.pair.is_some_and(|val| val.current == 0)
        && watchers.stage.pair.is_some_and(|val| val.changed_to(&101))
}

fn split(watchers: &Watchers, settings: &Settings) -> bool {
    watchers.game_mode.pair.is_some_and(|val| val.current == 0)
        && watchers
            .stage_result
            .pair
            .is_some_and(|val| val.changed_to(&1))
        && watchers.stage.pair.is_some_and(|val| match val.current {
            101 => settings.level1_1,
            102 => settings.level1_2,
            103 => settings.level1_3,
            104 => settings.level1_4,
            201 => settings.level2_1,
            202 => settings.level2_2,
            203 => settings.level2_3,
            204 => settings.level2_4,
            301 => settings.level3_1,
            302 => settings.level3_2,
            303 => settings.level3_3,
            304 => settings.level3_4,
            401 => settings.level4_1,
            402 => settings.level4_2,
            403 => settings.level4_3,
            404 => settings.level4_4,
            501 => settings.level5_1,
            502 => settings.level5_2,
            503 => settings.level5_3,
            504 => settings.level5_4,
            601 => settings.level6_1,
            602 => settings.level6_2,
            603 => settings.level6_3,
            604 => settings.level6_4,
            701 => settings.level7_1,
            702 => settings.level7_2,
            703 => settings.level7_3,
            704 => settings.level7_4,
            801 => settings.level8_1,
            802 => settings.level8_2,
            803 => settings.level8_3,
            804 => settings.level8_4,
            901 => settings.level9_1,
            902 => settings.level9_2,
            903 => settings.level9_3,
            904 => settings.level9_4,
            1001 => settings.level10_1,
            1002 => settings.level10_2,
            1003 => settings.level10_3,
            1004 => settings.level10_4,
            1101 => settings.level11_1,
            1102 => settings.level11_2,
            1103 => settings.level11_3,
            1104 => settings.level11_4,
            1201 => settings.level12_1,
            1202 => settings.level12_2,
            1203 => settings.level12_3,
            1204 => settings.level12_4,
            1301 => settings.level13_1,
            1302 => settings.level13_2,
            1303 => settings.level13_3,
            1304 => settings.level13_4,
            1401 => settings.level14_1,
            1402 => settings.level14_2,
            1403 => settings.level14_3,
            1404 => settings.level14_4,
            1501 => settings.level14_1,
            1502 => settings.level14_2,
            1503 => settings.level14_3,
            1504 => settings.level14_4,
            _ => false,
        })
}

fn reset(_watchers: &Watchers, _settings: &Settings) -> bool {
    false
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
struct StageSceneManager {
    game_mode: u32, // 0 = world, 1 = battle, 2 = endless
    stage_parent_no: u32,
    stage_child_no: u32,
    _battle_round: u32,
    stage_result: u32,
}