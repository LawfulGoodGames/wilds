use color_eyre::eyre::OptionExt;
use crossterm::event::Event as CrosstermEvent;
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

const TICK_FPS: f64 = 30.0;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    SelectUp,
    SelectDown,
    Confirm,
    Back,
    LoadNewCharacter,
    LoadRenameStart,
    LoadRenameSubmit,
    LoadDeleteConfirm,
    LoadDeleteSelected,
    Left,
    Right,
    NextTab,
    OpenExplore,
    OpenPeople,
    OpenCharacter,
    OpenInventory,
    OpenEquipment,
    OpenQuests,
    OpenAchievements,
    OpenShop,
    OpenTraining,
    RestAtInn,
    ExploreSelected,
    ShopToggleMode,
    ShopNextVendor,
    ShopPreviousVendor,
    ShopTransaction,
    QuestAccept,
    QuestToggleCompleted,
    QuestToggleLocked,
    CombatTabWeapon,
    CombatTabAbility,
    CombatTabItem,
    CombatNextTab,
    CombatCycleOptionUp,
    CombatCycleOptionDown,
    CombatCycleTarget,
    CombatUseSelected,
    CombatDefend,
    CombatFlee,
    DetailScrollUp,
    DetailScrollDown,
    TrainingInput(char),
    Quit,
}

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async move {
            let _ = actor.run().await;
        });
        Self { sender, receiver }
    }

    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}

struct EventTask {
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                _ = self.sender.closed() => break,
                _ = tick_delay => {
                    let _ = self.sender.send(Event::Tick);
                }
                Some(Ok(evt)) = crossterm_event => {
                    let _ = self.sender.send(Event::Crossterm(evt));
                }
            }
        }
        Ok(())
    }
}
