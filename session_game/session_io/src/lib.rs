#![no_std]

use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId, collections::HashMap, MessageId};
use io::{Action, Event};

pub type SentMessageId = MessageId;
pub type OriginalMessageId = MessageId;
pub type SessionState = HashMap<MessageId, Session>;

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub struct Session {
    // target_program_id: ActorId,
    pub owner_id: ActorId,
    pub msg_ids: (SentMessageId, OriginalMessageId),
    pub session_status: SessionStatus,
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum SessionStatus {
    Waiting,
    Prepare,
    Finish,
}


#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum  GameStatus {
    Prepare,
    Start,
    CheckWork((Vec<u8>, Vec<u8>)),
    GameOver(GamgeResult),
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum GamgeResult {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub struct UserData{
    pub game_status: GameStatus,
    pub try_num: u8,
}


#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum ProxyAction {
    StartGame,
    CheckWord {
        word: String,
    },
    CheckGameStatus(ActorId),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum ProxyEvent {
    Status(GameStatus),
    ParamError,
    MessageAlreadySent,
    // GameOver,
}


#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum GameEvent {
    WIN,
    LOSE
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub struct InitParam{
    pub target_program_id: ActorId,
    pub max_num: u8,
}

pub struct ProxyMetadata;

impl Metadata for ProxyMetadata {
    type Init = In<InitParam>;
    type Handle = InOut<ProxyAction, ProxyEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<ActorId, UserData>;
}