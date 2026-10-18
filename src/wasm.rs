use std::cell::RefCell;
use std::sync::Arc;

use enumset::EnumSet;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

use crate::bot::{Bot, BotConfig, BotOptions, Statistics};
use crate::data::GameState;
use crate::tbp::{BotMessage, FrontendMessage, MoveInfo, Randomizer, Start};

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default());
}

#[derive(Default)]
struct Runtime {
    config: Option<Arc<BotConfig>>,
    bot: Option<Bot>,
    waiting_on_first_piece: Option<Start>,
    stats: Statistics,
    nodes_since_start: u64,
    started_at: f64,
    last_advance: f64,
}

#[wasm_bindgen]
pub fn initialize() -> Result<JsValue, JsValue> {
    serialize(&BotMessage::Info {
        name: "Cold Clear 2",
        version: concat!(env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH")),
        author: "MinusKelvin",
        features: &[],
    })
}

#[wasm_bindgen]
pub fn configure(config: JsValue) -> Result<(), JsValue> {
    let config: BotConfig = deserialize(config)?;
    RUNTIME.with(|runtime| runtime.borrow_mut().config = Some(Arc::new(config)));
    Ok(())
}

#[wasm_bindgen]
pub fn process_message(message: JsValue) -> Result<JsValue, JsValue> {
    let message: FrontendMessage = deserialize(message)?;
    RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        let output = match message {
            FrontendMessage::Start(start) => {
                if start.hold.is_none() && start.queue.is_empty() {
                    runtime.waiting_on_first_piece = Some(start);
                } else {
                    start_bot(&mut runtime, start);
                }
                None
            }
            FrontendMessage::Stop | FrontendMessage::Quit => {
                runtime.bot = None;
                runtime.waiting_on_first_piece = None;
                None
            }
            FrontendMessage::Suggest => {
                let initial_stats = if runtime.stats.selections == 0 {
                    runtime.bot.as_ref().map(Bot::do_work)
                } else {
                    None
                };
                if let Some(stats) = initial_stats {
                    runtime.stats.accumulate(stats);
                    runtime.nodes_since_start += stats.nodes;
                }
                runtime.bot.as_ref().map(|bot| BotMessage::Suggestion {
                    moves: bot.suggest(),
                    move_info: move_info(&runtime),
                })
            }
            FrontendMessage::Play { mv } => {
                if let Some(bot) = &mut runtime.bot {
                    bot.advance(mv);
                    runtime.stats = Statistics::default();
                    runtime.last_advance = now();
                }
                None
            }
            FrontendMessage::NewPiece { piece } => {
                if let Some(mut start) = runtime.waiting_on_first_piece.take() {
                    if let Randomizer::SevenBag { bag_state } = &mut start.randomizer {
                        if bag_state.is_empty() {
                            *bag_state = EnumSet::all();
                        }
                        bag_state.remove(piece);
                    }
                    start.queue.push(piece);
                    start_bot(&mut runtime, start);
                } else if let Some(bot) = &mut runtime.bot {
                    bot.new_piece(piece);
                }
                None
            }
            FrontendMessage::Rules => Some(BotMessage::Ready),
            FrontendMessage::Unknown => None,
        };
        serialize(&output)
    })
}

#[wasm_bindgen]
pub fn think(iterations: u32) -> Result<JsValue, JsValue> {
    RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        let stats = match runtime.bot.as_ref() {
            Some(bot) => {
                let mut stats = Statistics::default();
                for _ in 0..iterations {
                    stats.accumulate(bot.do_work());
                }
                stats
            }
            None => return Err(JsValue::from_str("Bot has not been started")),
        };
        runtime.stats.accumulate(stats);
        runtime.nodes_since_start += stats.nodes;
        serialize(&move_info(&runtime))
    })
}

fn start_bot(runtime: &mut Runtime, start: Start) {
    runtime.bot = Some(create_bot(
        start,
        runtime
            .config
            .get_or_insert_with(|| Arc::new(BotConfig::default()))
            .clone(),
    ));
    runtime.waiting_on_first_piece = None;
    runtime.stats = Statistics::default();
    runtime.nodes_since_start = 0;
    runtime.started_at = now();
    runtime.last_advance = runtime.started_at;
}

fn create_bot(mut start: Start, config: Arc<BotConfig>) -> Bot {
    let reserve = start.hold.unwrap_or_else(|| start.queue.remove(0));
    let speculate = matches!(start.randomizer, Randomizer::SevenBag { .. });
    let bag = match start.randomizer {
        Randomizer::Unknown => EnumSet::all(),
        Randomizer::SevenBag { mut bag_state } => {
            for &piece in start.queue.iter().rev() {
                if bag_state == EnumSet::all() {
                    bag_state = EnumSet::empty();
                }
                bag_state.insert(piece);
            }
            bag_state
        }
    };

    let state = GameState {
        reserve,
        back_to_back: start.back_to_back,
        combo: start.combo.try_into().unwrap_or(255),
        bag,
        board: start.board.into(),
    };
    Bot::new(BotOptions { speculate, config }, state, &start.queue)
}

fn move_info(runtime: &Runtime) -> MoveInfo {
    let now = now();
    let elapsed = ((now - runtime.last_advance) / 1000.0).max(0.001);
    let total_elapsed = ((now - runtime.started_at) / 1000.0).max(0.001);
    let expanded = if runtime.stats.selections == 0 {
        0.0
    } else {
        runtime.stats.expansions as f64 / runtime.stats.selections as f64 * 100.0
    };
    MoveInfo {
        nodes: runtime.stats.nodes,
        nps: runtime.stats.nodes as f64 / elapsed,
        extra: format!(
            "{:.1}% of selections expanded, overall speed: {:.1} Mnps",
            expanded,
            runtime.nodes_since_start as f64 / total_elapsed / 1_000_000.0
        ),
    }
}

fn now() -> f64 {
    js_sys::Date::now()
}

fn deserialize<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
    serde::Serialize::serialize(value, &serializer)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
