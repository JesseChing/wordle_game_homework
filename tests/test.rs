use gstd::ActorId;
use gtest::{Log, Program, ProgramBuilder, System};
use session_io::{GameStatus, GamgeResult, InitParam, ProxyAction, ProxyEvent, UserData};

#[test]
fn test_init() {
    let system = System::new();
    system.init_logger();

    /*** init wordle_game program*/
    let target_program = ProgramBuilder::from_file(
        "target/wasm32-unknown-unknown/wasm32-unknown-unknown/release/wordle_game.opt.wasm",
    )
    .with_id(1)
    .build(&system);

    let mut result = target_program.send_bytes(10, []);
    assert!(!result.main_failed());

    /**Init session program */
    let proxy_program = ProgramBuilder::from_file(
        "target/wasm32-unknown-unknown/wasm32-unknown-unknown/release/session_game.opt.wasm",
    )
    .with_id(2)
    .build(&system);

    result = proxy_program.send(10, InitParam{ target_program_id: target_program.id(), max_num: 3});
    assert!(!result.main_failed());


}

#[test]
fn test_game() {
    let system = System::new();
    system.init_logger();

    /*** init wordle_game program*/
    let target_program = ProgramBuilder::from_file(
        "target/wasm32-unknown-unknown/release/wordle_game.opt.wasm",
    )
    .with_id(1)
    .build(&system);

    let mut result = target_program.send_bytes(10, []);
    assert!(!result.main_failed());

    /**Init session program */
    let proxy_program = ProgramBuilder::from_file(
        "target/wasm32-unknown-unknown/release/session_game.opt.wasm",
    )
    .with_id(2)
    .build(&system);

    result = proxy_program.send(10, InitParam{ target_program_id: target_program.id(), max_num: 3});
    assert!(!result.main_failed());

    // lose game case
    result = proxy_program.send(10, ProxyAction::StartGame);
    assert!(!result.main_failed());

    let mut state: UserData = proxy_program.read_state(ActorId::from(10)).unwrap();
    assert_eq!(state.try_num , 0);

    result = proxy_program.send(10, ProxyAction::CheckWord{word: "test".to_string()});
    assert!(!result.main_failed());

    let mut log = Log::builder().payload(ProxyEvent::ParamError);
    assert!(result.contains(&log));

    result = proxy_program.send(10, ProxyAction::CheckWord{word: "test2".to_string()});
    assert!(!result.main_failed());

    state = proxy_program.read_state(ActorId::from(10)).unwrap();
    assert_eq!(state.try_num , 1);
    
    result = proxy_program.send(10, ProxyAction::CheckWord{word: "test3".to_string()});
    assert!(!result.main_failed());
    state = proxy_program.read_state(ActorId::from(10)).unwrap();
    assert_eq!(state.try_num , 2);

    result = proxy_program.send(10, ProxyAction::CheckWord{word: "test3".to_string()});
    assert!(!result.main_failed());
    state = proxy_program.read_state(ActorId::from(10)).unwrap();
    assert_eq!(state.try_num , 3);

    result = proxy_program.send(10, ProxyAction::CheckWord{word: "test3".to_string()});
    assert!(!result.main_failed());
    log = Log::builder().payload(ProxyEvent::Status(GameStatus::GameOver(GamgeResult::Lose)));
    assert!(result.contains(&log));


    // win game case
    result = proxy_program.send(11, ProxyAction::StartGame);
    assert!(!result.main_failed());

    result = proxy_program.send(11, ProxyAction::CheckWord{word: "house".to_string()});
    assert!(!result.main_failed());

    result = proxy_program.send(11, ProxyAction::CheckWord{word: "human".to_string()});
    assert!(!result.main_failed());

    result = proxy_program.send(11, ProxyAction::CheckWord{word: "horse".to_string()});
    assert!(!result.main_failed());

    log = Log::builder().payload(ProxyEvent::Status(GameStatus::GameOver(GamgeResult::Win)));
    assert!(result.contains(&log));

}


