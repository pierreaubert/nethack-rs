//! NetHack on-chain: a PolkaVM smart contract
//!
//! Runs NetHack game logic as a smart contract on Polkadot's AssetHub via
//! pallet-revive + PolkaVM. Each contract instance is one game session.
//!
//! ## Contract API (Solidity-style selectors)
//!
//! | Function    | Signature                               | Description                     |
//! |-------------|-----------------------------------------|---------------------------------|
//! | newGame     | newGame(uint8,uint8,uint8,uint256)      | Start a new game                |
//! | tick        | tick(uint8,uint8)                       | Execute one game turn           |
//! | getState    | getState()                              | Read full serialized game state |
//! | getMessages | getMessages()                           | Read current turn messages      |

#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use polkavm_derive::polkavm_export;
use simplealloc::SimpleAlloc;
use uapi::{HostFn, HostFnImpl as api, ReturnFlags, StorageFlags};

use nh_core::action::{Command, Direction};
use nh_core::player::role::{Gender, Race, Role};
use nh_core::{GameLoop, GameLoopResult, GameRng, GameState};

// ---------------------------------------------------------------------------
// Allocator — 512 KB heap for game state manipulation
// ---------------------------------------------------------------------------

#[global_allocator]
static ALLOCATOR: SimpleAlloc<{ 512 * 1024 }> = SimpleAlloc::new();

// ---------------------------------------------------------------------------
// Panic handler — trap on panic in contract context
// ---------------------------------------------------------------------------

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a PolkaVM contract, we just halt on panic
    loop {}
}

// ---------------------------------------------------------------------------
// Storage keys (32-byte)
// ---------------------------------------------------------------------------

/// Main game state blob
const KEY_GAME_STATE: [u8; 32] = [0u8; 32];

/// Contract owner (deployer address, 20 bytes stored in 32-byte key)
const KEY_OWNER: [u8; 32] = {
    let mut k = [0u8; 32];
    k[0] = 1;
    k
};

/// Turn result from last tick (for easy retrieval)
const KEY_LAST_RESULT: [u8; 32] = {
    let mut k = [0u8; 32];
    k[0] = 2;
    k
};

/// Messages from last tick
const KEY_MESSAGES: [u8; 32] = {
    let mut k = [0u8; 32];
    k[0] = 3;
    k
};

// ---------------------------------------------------------------------------
// Function selectors — keccak256(signature)[0..4]
// ---------------------------------------------------------------------------

/// Compute keccak256 of a byte slice and return the first 4 bytes (function selector).
const fn keccak_selector(sig: &[u8]) -> [u8; 4] {
    // We pre-compute selectors at compile time using a const keccak256.
    // Since const fn keccak is complex, we use pre-computed values.
    // This function exists as documentation — see the constants below.
    //
    // To recompute: `cast sig "newGame(uint8,uint8,uint8,uint256)"`
    // or use tiny_keccak at runtime.
    let _ = sig;
    [0; 4] // placeholder, actual values in the constants
}

/// newGame(uint8 role, uint8 race, uint8 gender, uint256 rngSeed)
/// Creates a new game with the given character identity and RNG seed.
const SEL_NEW_GAME: [u8; 4] = compute_selector(b"newGame(uint8,uint8,uint8,uint256)");

/// tick(uint8 commandType, uint8 commandArg)
/// Executes one game turn with the given command.
const SEL_TICK: [u8; 4] = compute_selector(b"tick(uint8,uint8)");

/// getState()
/// Returns the full serialized game state (read-only).
const SEL_GET_STATE: [u8; 4] = compute_selector(b"getState()");

/// getMessages()
/// Returns messages from the last tick (read-only).
const SEL_GET_MESSAGES: [u8; 4] = compute_selector(b"getMessages()");

/// Compile-time keccak256 selector computation.
/// Uses a simplified keccak256 permutation (no external crate needed at const time).
const fn compute_selector(input: &[u8]) -> [u8; 4] {
    let hash = const_keccak256(input);
    [hash[0], hash[1], hash[2], hash[3]]
}

/// Minimal const-fn keccak256 implementation for selector computation.
/// Only needs to be correct for short inputs (< 136 bytes = rate for keccak256).
const fn const_keccak256(input: &[u8]) -> [u8; 32] {
    // Keccak-256: rate=136, capacity=64, output=32
    const RATE: usize = 136;

    // Initialize state
    let mut state = [0u64; 25];

    // Absorb: pad input (keccak padding: 0x01 ... 0x80)
    let mut block = [0u8; RATE];
    let mut i = 0;
    while i < input.len() && i < RATE {
        block[i] = input[i];
        i += 1;
    }
    block[input.len()] = 0x01;
    block[RATE - 1] |= 0x80;

    // XOR block into state
    i = 0;
    while i < RATE / 8 {
        let b = i * 8;
        let lane = (block[b] as u64)
            | ((block[b + 1] as u64) << 8)
            | ((block[b + 2] as u64) << 16)
            | ((block[b + 3] as u64) << 24)
            | ((block[b + 4] as u64) << 32)
            | ((block[b + 5] as u64) << 40)
            | ((block[b + 6] as u64) << 48)
            | ((block[b + 7] as u64) << 56);
        state[i] ^= lane;
        i += 1;
    }

    // Keccak-f[1600] permutation (24 rounds)
    state = keccak_f(state);

    // Squeeze: extract 32 bytes
    let mut output = [0u8; 32];
    i = 0;
    while i < 4 {
        let lane = state[i];
        let b = i * 8;
        output[b] = lane as u8;
        output[b + 1] = (lane >> 8) as u8;
        output[b + 2] = (lane >> 16) as u8;
        output[b + 3] = (lane >> 24) as u8;
        output[b + 4] = (lane >> 32) as u8;
        output[b + 5] = (lane >> 40) as u8;
        output[b + 6] = (lane >> 48) as u8;
        output[b + 7] = (lane >> 56) as u8;
        i += 1;
    }
    output
}

const fn keccak_f(mut state: [u64; 25]) -> [u64; 25] {
    const RC: [u64; 24] = [
        0x0000000000000001, 0x0000000000008082, 0x800000000000808A,
        0x8000000080008000, 0x000000000000808B, 0x0000000080000001,
        0x8000000080008081, 0x8000000000008009, 0x000000000000008A,
        0x0000000000000088, 0x0000000080008009, 0x000000008000000A,
        0x000000008000808B, 0x800000000000008B, 0x8000000000008089,
        0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
        0x000000000000800A, 0x800000008000000A, 0x8000000080008081,
        0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
    ];
    const ROTC: [u32; 24] = [
        1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14,
        27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
    ];
    const PILN: [usize; 24] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4,
        15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
    ];

    let mut round = 0;
    while round < 24 {
        // θ (theta)
        let mut c = [0u64; 5];
        let mut x = 0;
        while x < 5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
            x += 1;
        }
        x = 0;
        while x < 5 {
            let d = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            let mut y = 0;
            while y < 25 {
                state[x + y] ^= d;
                y += 5;
            }
            x += 1;
        }

        // ρ (rho) + π (pi)
        let mut t = state[1];
        let mut i = 0;
        while i < 24 {
            let j = PILN[i];
            let tmp = state[j];
            state[j] = t.rotate_left(ROTC[i]);
            t = tmp;
            i += 1;
        }

        // χ (chi)
        let mut y = 0;
        while y < 25 {
            let mut row = [0u64; 5];
            x = 0;
            while x < 5 {
                row[x] = state[y + x];
                x += 1;
            }
            x = 0;
            while x < 5 {
                state[y + x] = row[x] ^ ((!row[(x + 1) % 5]) & row[(x + 2) % 5]);
                x += 1;
            }
            y += 5;
        }

        // ι (iota)
        state[0] ^= RC[round];

        round += 1;
    }
    state
}

// ---------------------------------------------------------------------------
// Contract entry points
// ---------------------------------------------------------------------------

/// Called once when the contract is deployed.
/// Stores the deployer's address as the contract owner.
#[polkavm_export]
pub extern "C" fn deploy() {
    let mut caller = [0u8; 20];
    api::caller(&mut caller);

    // Store deployer as owner
    api::set_storage(StorageFlags::empty(), &KEY_OWNER, &caller);
}

/// Called on every transaction to the contract.
/// Dispatches based on the 4-byte function selector.
#[polkavm_export]
pub extern "C" fn call() {
    // Read call input — allocate a generous buffer
    // Max input: 4 (selector) + 32*4 (up to 4 uint256 params) = 132 bytes
    let mut input_buf = [0u8; 256];
    api::input(&mut input_buf);

    if input_buf.len() < 4 {
        api::return_value(ReturnFlags::REVERT, b"input too short");
        return;
    }

    let selector = [input_buf[0], input_buf[1], input_buf[2], input_buf[3]];
    let data = &input_buf[4..];

    match selector {
        SEL_NEW_GAME => handle_new_game(data),
        SEL_TICK => handle_tick(data),
        SEL_GET_STATE => handle_get_state(),
        SEL_GET_MESSAGES => handle_get_messages(),
        _ => {
            api::return_value(ReturnFlags::REVERT, b"unknown selector");
        }
    }
}

// ---------------------------------------------------------------------------
// Handler: newGame(uint8 role, uint8 race, uint8 gender, uint256 rngSeed)
// ---------------------------------------------------------------------------

fn handle_new_game(data: &[u8]) {
    // ABI: each parameter is padded to 32 bytes
    if data.len() < 128 {
        api::return_value(ReturnFlags::REVERT, b"invalid newGame params");
        return;
    }

    // Extract parameters from ABI-encoded data (last byte of each 32-byte word)
    let role_byte = data[31];
    let race_byte = data[63];
    let gender_byte = data[95];
    // RNG seed: last 8 bytes of the uint256 (we only use u64)
    let seed = u64::from_be_bytes([
        data[120], data[121], data[122], data[123],
        data[124], data[125], data[126], data[127],
    ]);

    let role = match role_byte {
        0 => Role::Archeologist,
        1 => Role::Barbarian,
        2 => Role::Caveman,
        3 => Role::Healer,
        4 => Role::Knight,
        5 => Role::Monk,
        6 => Role::Priest,
        7 => Role::Ranger,
        8 => Role::Rogue,
        9 => Role::Samurai,
        10 => Role::Tourist,
        11 => Role::Valkyrie,
        12 => Role::Wizard,
        _ => {
            api::return_value(ReturnFlags::REVERT, b"invalid role");
            return;
        }
    };

    let race = match race_byte {
        0 => Race::Human,
        1 => Race::Elf,
        2 => Race::Dwarf,
        3 => Race::Gnome,
        4 => Race::Orc,
        _ => {
            api::return_value(ReturnFlags::REVERT, b"invalid race");
            return;
        }
    };

    let gender = match gender_byte {
        0 => Gender::Male,
        1 => Gender::Female,
        2 => Gender::Neuter,
        _ => {
            api::return_value(ReturnFlags::REVERT, b"invalid gender");
            return;
        }
    };

    // Create RNG from on-chain seed
    let rng = GameRng::new(seed);

    // Build player name from caller address
    let mut caller = [0u8; 20];
    api::caller(&mut caller);
    let name = hex_name(&caller[..4]);

    // Create new game state
    let state = GameState::new_with_identity(rng, name, role, race, gender);

    // Serialize and store
    match postcard::to_allocvec(&state) {
        Ok(bytes) => {
            api::set_storage(StorageFlags::empty(), &KEY_GAME_STATE, &bytes);
            // Return success (empty return = success)
            api::return_value(ReturnFlags::empty(), &[1]); // 1 = success
        }
        Err(_) => {
            api::return_value(ReturnFlags::REVERT, b"serialization failed");
        }
    }
}

// ---------------------------------------------------------------------------
// Handler: tick(uint8 commandType, uint8 commandArg)
// ---------------------------------------------------------------------------

/// Command type encoding for the on-chain interface.
/// Maps uint8 command types to nh-core Command variants.
fn decode_command(cmd_type: u8, cmd_arg: u8) -> Option<Command> {
    match cmd_type {
        // Movement commands: arg encodes direction
        0 => Some(Command::Move(decode_direction(cmd_arg)?)),
        1 => Some(Command::MoveUntilInteresting(decode_direction(cmd_arg)?)),
        2 => Some(Command::Run(decode_direction(cmd_arg)?)),
        3 => Some(Command::Rest),
        4 => Some(Command::GoUp),
        5 => Some(Command::GoDown),

        // Combat
        6 => Some(Command::Fight(decode_direction(cmd_arg)?)),

        // Object manipulation (arg = inventory letter)
        10 => Some(Command::Pickup),
        11 => Some(Command::Drop(cmd_arg as char)),
        12 => Some(Command::Eat(cmd_arg as char)),
        13 => Some(Command::Quaff(cmd_arg as char)),
        14 => Some(Command::Read(cmd_arg as char)),
        15 => Some(Command::Apply(cmd_arg as char)),
        16 => Some(Command::Wear(cmd_arg as char)),
        17 => Some(Command::TakeOff(cmd_arg as char)),
        18 => Some(Command::Wield(Some(cmd_arg as char))),

        // Information
        20 => Some(Command::Inventory),
        21 => Some(Command::Look),
        22 => Some(Command::WhatsHere),
        23 => Some(Command::Discoveries),
        24 => Some(Command::History),
        25 => Some(Command::Search),

        // Actions
        30 => Some(Command::Open(decode_direction(cmd_arg)?)),
        31 => Some(Command::Close(decode_direction(cmd_arg)?)),
        32 => Some(Command::Kick(decode_direction(cmd_arg)?)),
        33 => Some(Command::Pray),

        // Meta
        40 => Some(Command::Save),
        41 => Some(Command::Quit),

        _ => None,
    }
}

fn decode_direction(arg: u8) -> Option<Direction> {
    match arg {
        0 => Some(Direction::North),
        1 => Some(Direction::South),
        2 => Some(Direction::East),
        3 => Some(Direction::West),
        4 => Some(Direction::NorthEast),
        5 => Some(Direction::NorthWest),
        6 => Some(Direction::SouthEast),
        7 => Some(Direction::SouthWest),
        _ => None,
    }
}

fn handle_tick(data: &[u8]) {
    // ABI: two uint8 params, each padded to 32 bytes
    if data.len() < 64 {
        api::return_value(ReturnFlags::REVERT, b"invalid tick params");
        return;
    }

    let cmd_type = data[31];
    let cmd_arg = data[63];

    let command = match decode_command(cmd_type, cmd_arg) {
        Some(cmd) => cmd,
        None => {
            api::return_value(ReturnFlags::REVERT, b"invalid command");
            return;
        }
    };

    // Load game state from storage
    let state = match load_game_state() {
        Some(s) => s,
        None => {
            api::return_value(ReturnFlags::REVERT, b"no game state");
            return;
        }
    };

    // Execute the tick
    let mut game_loop = GameLoop::new(state);
    let result = game_loop.tick(command);

    // Collect messages before moving state out
    let messages = game_loop.state().messages.clone();
    let state = game_loop.into_state();

    // Encode result as a single byte
    let result_byte = match &result {
        GameLoopResult::Continue => 0u8,
        GameLoopResult::PlayerDied(_) => 1,
        GameLoopResult::PlayerQuit => 2,
        GameLoopResult::PlayerWon => 3,
        GameLoopResult::SaveAndQuit => 4,
    };

    // Store updated state
    match postcard::to_allocvec(&state) {
        Ok(bytes) => {
            api::set_storage(StorageFlags::empty(), &KEY_GAME_STATE, &bytes);
        }
        Err(_) => {
            api::return_value(ReturnFlags::REVERT, b"state serialization failed");
            return;
        }
    }

    // Store messages for getMessages()
    match postcard::to_allocvec(&messages) {
        Ok(bytes) => {
            api::set_storage(StorageFlags::empty(), &KEY_MESSAGES, &bytes);
        }
        Err(_) => {
            // Non-fatal: messages are secondary
        }
    }

    // Store result
    api::set_storage(StorageFlags::empty(), &KEY_LAST_RESULT, &[result_byte]);

    // Return result + death message if applicable
    let mut output = vec![result_byte];
    if let GameLoopResult::PlayerDied(msg) = result {
        output.extend_from_slice(msg.as_bytes());
    }
    api::return_value(ReturnFlags::empty(), &output);
}

// ---------------------------------------------------------------------------
// Handler: getState() — read-only
// ---------------------------------------------------------------------------

fn handle_get_state() {
    // Large buffer for game state (up to 256KB)
    let mut buf = vec![0u8; 256 * 1024];
    match api::get_storage(&KEY_GAME_STATE, &mut buf) {
        Ok(ret) => {
            api::return_value(ReturnFlags::empty(), &buf[..ret.len()]);
        }
        Err(_) => {
            api::return_value(ReturnFlags::REVERT, b"no game state");
        }
    }
}

// ---------------------------------------------------------------------------
// Handler: getMessages() — read-only
// ---------------------------------------------------------------------------

fn handle_get_messages() {
    let mut buf = vec![0u8; 16 * 1024];
    match api::get_storage(&KEY_MESSAGES, &mut buf) {
        Ok(ret) => {
            api::return_value(ReturnFlags::empty(), &buf[..ret.len()]);
        }
        Err(_) => {
            // No messages — return empty
            api::return_value(ReturnFlags::empty(), &[]);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load and deserialize game state from contract storage.
fn load_game_state() -> Option<GameState> {
    // Allocate buffer (game state can be large with multiple visited levels)
    let mut buf = vec![0u8; 256 * 1024];
    let ret = api::get_storage(&KEY_GAME_STATE, &mut buf).ok()?;
    let bytes = &buf[..ret.len()];
    postcard::from_bytes(bytes).ok()
}

/// Convert first 4 bytes of an address to a hex-based player name.
fn hex_name(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut name = String::with_capacity(2 + bytes.len() * 2);
    name.push_str("0x");
    for &b in bytes {
        name.push(HEX[(b >> 4) as usize] as char);
        name.push(HEX[(b & 0x0f) as usize] as char);
    }
    name
}
