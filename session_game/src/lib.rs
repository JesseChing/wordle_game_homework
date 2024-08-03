#![no_std]
// use gclient::metadata::runtime_types::gear_core::message::user;
use gstd::{
    collections::HashMap,
    exec, msg,
    prelude::*,
    ActorId, MessageId,
};
use io::{Action, Event};
use session_io::{
    GameStatus, GamgeResult, InitParam, ProxyAction, ProxyEvent, Session, SessionState,
    SessionStatus, UserData,
};

static mut SESSION_STATE: Option<SessionState> = None;
static mut INIT_PARAMS: Option<InitParam> = None;
// static mut SESSION_HANDLE: Option<HashSet<ActorId>> = None;
static mut USER_STATE: Option<HashMap<ActorId, UserData>> = None;

#[no_mangle]
extern "C" fn init() {
    let init_param: InitParam = msg::load().expect("Unable to decode ");
    unsafe {
        SESSION_STATE = Some(HashMap::new());
        USER_STATE = Some(HashMap::new());
        INIT_PARAMS = Some(init_param);
        // SESSION_HANDLE = Some(HashSet::new());
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: ProxyAction = msg::load().expect("Unable to decode ");
    let sessions = unsafe {
        SESSION_STATE
            .as_mut()
            .expect("The program is not initialized")
    };

    let users = unsafe { USER_STATE.as_mut().expect("The program is not initialized") };

    let current_session = sessions.get(&msg::id());
    let init_params = unsafe { INIT_PARAMS.unwrap() };

    if current_session.is_some() {
        let current_session = current_session.unwrap();
        let has_user = users.contains_key(&msg::source());
        if current_session.owner_id == msg::source() && has_user && current_session.session_status == SessionStatus::Finish {
            let current_user = users.get(&msg::source()).expect("Can't find the user data");
            let reply_info = ProxyEvent::Status(current_user.game_status.clone());
            sessions.remove(&msg::id());
            if current_user.game_status == GameStatus::GameOver(GamgeResult::Win) ||
            current_user.game_status == GameStatus::GameOver(GamgeResult::Lose) {
                users.remove(&msg::source());
            }
            msg::reply(reply_info, 0).expect("send reply error");
        }
    } else {
        match action {
            ProxyAction::StartGame => {
                let msg_id = msg::send(
                    init_params.target_program_id,
                    Action::StartGame {
                        user: msg::source(),
                    },
                    0,
                )
                .expect("error in sending a reply");
                let session = Session {
                    owner_id: msg::source(),
                    msg_ids: (msg_id, msg::id()),
                    session_status: SessionStatus::Waiting,
                };
                sessions.insert(msg_id, session);
                let _ = msg::send_delayed(
                    exec::program_id(),
                    ProxyAction::CheckGameStatus(msg::source()),
                    0,
                    100,
                )
                .expect("error in delaying msg");
                // msg::reply(ProxyEvent::MessageAlreadySent, 0).expect("error in sending a reply");
                exec::wait();
            }

            ProxyAction::CheckWord { word } => {
                if word.len() != 5 {
                    msg::reply(ProxyEvent::ParamError, 0).expect("reply error");
                    return;
                }

                let user_data = unsafe {
                    USER_STATE
                        .as_ref()
                        .expect("The program is not initialized")
                        .get(&msg::source())
                };

                if user_data.is_some() {
                    let data = user_data.unwrap();
                    if data.try_num <= init_params.max_num {
                        let msg_id = msg::send(
                            init_params.target_program_id,
                            Action::CheckWord {
                                user: msg::source(),
                                word: word.to_lowercase(),
                            },
                            0,
                        )
                        .expect("error in sending a reply");
        
                        let session_status = Session {
                            owner_id: msg::source(),
                            msg_ids: (msg_id, msg::id()),
                            session_status: SessionStatus::Waiting,
                        };
                        sessions.insert(msg_id, session_status);
                        // let _ = msg::reply(ProxyEvent::MessageAlreadySent, 0).expect("error in sending a reply");
                        exec::wait();
                    } else {
                        let _ = msg::reply(ProxyEvent::Status(GameStatus::GameOver(GamgeResult::Lose)), 0).expect("error in sending a reply");
                    }
                } else {
                    let _ = msg::reply(ProxyEvent::Status(GameStatus::GameOver(GamgeResult::Lose)), 0).expect("error in sending a reply");
                }
            }

            ProxyAction::CheckGameStatus(user_id) => {
                if msg::source() == exec::program_id() {
                    let has_user = users.contains_key(&user_id);
                    if has_user {
                        let user = users.get_mut(&user_id).unwrap();
                        if user.try_num > init_params.max_num {
                            user.game_status = GameStatus::GameOver(GamgeResult::Lose);
                        }
                        let _ = msg::reply(ProxyEvent::Status(user.game_status.clone()), 0).expect("error in sending a reply");
                    } else {
                        let _ = msg::reply(ProxyEvent::ParamError, 0).expect("error in sending a reply");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_msg_id = msg::reply_to().expect("Failed to decode reply_to data");

    let state = unsafe {
        SESSION_STATE
            .as_mut()
            .expect("The program is not initialized")
    };

    if state.contains_key(&reply_msg_id) {
        let reply_session = state.remove(&reply_msg_id).unwrap();

        if reply_session.session_status == SessionStatus::Waiting {
            let reply_message: Event = msg::load().expect("unable to load event");
            let origin_msg_id = reply_session.msg_ids.1;
            let owner = reply_session.owner_id;
            let new_session = Session {
                owner_id: reply_session.owner_id,
                msg_ids: reply_session.msg_ids,
                session_status: SessionStatus::Finish,
            };
            state.insert(origin_msg_id, new_session);

            let user_data = unsafe {
                USER_STATE
                    .as_mut()
                    .expect("The program is not initialized")
                    .entry(owner)
                    .or_insert(UserData {
                        game_status: GameStatus::Prepare,
                        try_num: 0,
                    })
            };

            let max_num = unsafe {
                INIT_PARAMS
                    .as_mut()
                    .expect("The program is not initialized")
            }
            .max_num;

            match reply_message {
                Event::GameStarted { user } => {
                    if owner == user {
                        user_data.game_status = GameStatus::Start;
                    }
                }
                Event::WordChecked {
                    user,
                    correct_positions,
                    contained_in_word,
                } => {
                    if owner == user {
                        user_data.try_num += 1;
                        if user_data.try_num > max_num {
                            user_data.game_status = GameStatus::GameOver(session_io::GamgeResult::Lose);
                        } else {
                            if correct_positions.len() == 5 {
                                user_data.game_status =
                                    GameStatus::GameOver(session_io::GamgeResult::Win);
                            } else {
                                user_data.game_status =
                                    GameStatus::CheckWork((correct_positions, contained_in_word));
                            }
                        }
                    }
                }
            }

            let _ = exec::wake(origin_msg_id).expect("Failed to wake message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let user_id: ActorId = msg::load().expect("Unable to decode ");
    let users = unsafe { USER_STATE.as_ref().expect("The program is not initialized") };
    let user_data = users.get(&user_id);
    if user_data.is_some() {
        let data = user_data.unwrap();
        let _ = msg::reply(UserData{ game_status:data.game_status.clone(), try_num: data.try_num}, 0).expect("reply error"); 
    }else {
        let _ = msg::reply(UserData{ game_status: GameStatus::Prepare, try_num: 0}, 0).expect("reply error"); 
    }
}
