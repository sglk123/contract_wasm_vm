use std::collections::HashMap;
use std::{env, slice};
use bincode::deserialize;
use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug)]
struct VoteState {
    value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct VotePollState {
    state: HashMap<String, i32>,
    event: Event,
    value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
enum Event {
    Poll(String),
    Vote(String),
}

static mut CONTRACT_INIT_STATE: i32 = 42;

trait WasmContract {
    fn init(len: i32) -> i32;
    fn invoke(init_state: i32, func: i32, args: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn init(len: i32) -> i32 {
    let mut contract_init_state_ptr: *mut i32 = &mut 0;
    unsafe {
        contract_init_state_ptr = &mut CONTRACT_INIT_STATE;
    }
    let allocated_memory = unsafe {
        std::alloc::alloc(std::alloc::Layout::from_size_align(10, std::mem::size_of::<i32>()).unwrap())
    };
    if !allocated_memory.is_null() { // memory release after use
        unsafe {
            std::ptr::write(contract_init_state_ptr, 42);
        }
    }
    contract_init_state_ptr as i32
}

#[no_mangle]
pub extern "C" fn allocate(len: i32) -> i32 {
    write_bytes(Vec::with_capacity(len as usize)) as i32
}

pub fn write_bytes(v: Vec<u8>) -> usize {
    v.leak().as_ptr() as usize
}

// Manual memory free after invoke
#[no_mangle]
pub extern "C" fn invoke(init_state: i32, func: i32, args: i32) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn vote(init_state: i32, len: i32) -> i32 {
    let slice = unsafe { slice::from_raw_parts(init_state as *const u8, len as usize) };
    let vec_u8 = slice.iter().map(|&x| x as u8).collect::<Vec<u8>>();
    let slice_u8 = vec_u8.as_slice();
    let mut deserialized: VoteState = bincode::deserialize(slice_u8).unwrap();
    info!("{:?}", deserialized);

    deserialized.value += 1;

    deserialized.value
}

// wasm call without static memory
#[no_mangle]
pub extern "C" fn poll_vote(init_state: i32, len: i32) -> i32 {
    info!("poll_vote");
    let mut state = get_state(init_state, len);
    match &state.event {
        Event::Poll(p) => {
            let p0: &str = p;
            if let None = state.state.get_mut(p0) {
                state.state.insert(p.clone(), 0);
            }
            construct_state(state)
        }
        Event::Vote(v) => {
            let v0: &str = v;
            if let Some(value) = state.state.get_mut(v0) {
                *value += 1;
            }
            construct_state(state)
        }
    }
}

static mut LENGTH: i32 = 0;

#[no_mangle]
pub extern "C" fn get_length() -> i32 {
    unsafe { LENGTH }
}

fn construct_state(state: VotePollState) -> i32 {
    let serialized: Vec<u8> = bincode::serialize(&state).unwrap();
    unsafe {
        LENGTH = serialized.len() as i32;
    }
    serialized.leak().as_ptr() as i32
}


fn get_state(init_state: i32, len: i32) -> VotePollState {
    let slice = unsafe { slice::from_raw_parts(init_state as *const u8, len as usize) };
    let vec_u8 = slice.iter().map(|&x| x as u8).collect::<Vec<u8>>();
    let slice_u8 = vec_u8.as_slice();
    let deserialized: VotePollState = bincode::deserialize(slice_u8).unwrap();
    info!("{:?}", deserialized);
    deserialized
}


