use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

use crate::data::{Board, Piece, Placement};

#[derive(Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(
    target_arch = "wasm32",
    tsify(
        type = "{ type: 'rules' } | ({ type: 'start' } & Start) | { type: 'play'; move: Placement } | { type: 'new_piece'; piece: Piece } | { type: 'suggest' } | { type: 'stop' } | { type: 'quit' } | { type: string; [key: string]: unknown }"
    )
)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum FrontendMessage {
    Rules,
    Start(Start),
    Play {
        #[serde(rename = "move")]
        mv: Placement,
    },
    NewPiece {
        piece: Piece,
    },
    Suggest,
    Stop,
    Quit,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum BotMessage {
    Info {
        name: &'static str,
        version: &'static str,
        author: &'static str,
        features: &'static [&'static str],
    },
    Ready,
    Suggestion {
        moves: Vec<Placement>,
        move_info: MoveInfo,
    },
}

#[derive(Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Start {
    pub board: Board,
    pub queue: Vec<Piece>,
    pub hold: Option<Piece>,
    pub combo: u32,
    pub back_to_back: bool,
    #[serde(default)]
    pub randomizer: Randomizer,
}

#[derive(Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(
    target_arch = "wasm32",
    tsify(
        type = "{ type: 'seven_bag'; bag_state: Piece[] } | { type: string; [key: string]: unknown }"
    )
)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Randomizer {
    SevenBag {
        #[serde(deserialize_with = "collect_enumset")]
        #[cfg_attr(target_arch = "wasm32", tsify(type = "Piece[]"))]
        bag_state: EnumSet<Piece>,
    },
    #[serde(other)]
    Unknown,
}

impl Default for Randomizer {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct MoveInfo {
    pub nodes: u64,
    pub nps: f64,
    pub extra: String,
}

impl From<Vec<[Option<char>; 10]>> for Board {
    fn from(v: Vec<[Option<char>; 10]>) -> Self {
        let mut cols = [0; 10];
        for x in 0..10 {
            for y in 0..40 {
                if v[y][x].is_some() {
                    cols[x] |= 1 << y;
                }
            }
        }
        Board { cols }
    }
}

fn collect_enumset<'de, D, T>(de: D) -> Result<EnumSet<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: EnumSetType + Deserialize<'de>,
{
    Ok(Vec::<T>::deserialize(de)?.into_iter().collect())
}
