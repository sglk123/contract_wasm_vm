use std::collections::HashMap;
use std::slice;
use bincode;
use serde::{Deserialize, Serialize};
use wasmer::{imports, Instance, Module, Store, Value, WasmPtr};
use wasmer::Value::I32;
use crate::Event::Poll;

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

#[derive(Serialize, Deserialize, Debug)]
struct VoteState {
    value: i32,
}

fn main() -> anyhow::Result<()> {
    let wasm_bytes = std::fs::read("vote_poll_contract.wasm").expect("Failed to read Wasm file");
    let mut store = Store::default();
    let module = Module::new(&store, &wasm_bytes)?; // artifact - machine code
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;


    let vote_state = VoteState {
        value: 22,
    };
    let serialized_vote: Vec<u8> = bincode::serialize(&vote_state).unwrap();

    println!("init contract...");
    let init = instance.exports.get_function("allocate")?;
    let result = init.call(&mut store, &[Value::I32(100)])?;
    let ptr = result[0].i32().unwrap();
    let wasm_ptr = WasmPtr::<i32>::new(ptr as u32);

    let memory = instance.exports.get_memory("memory")?;
    let memory_view = memory.view(&store);

    let values = wasm_ptr.slice(&memory_view, serialized_vote.len() as u32).unwrap();
    for i in 0..serialized_vote.len() {
        values.index(i as u64).write(serialized_vote[i] as i32).unwrap();
    }

    println!("invoke poll_vote...");

    let poll_vote = instance.exports.get_function("vote")?;
    let result = poll_vote.call(&mut store, &[Value::I32(ptr), I32(serialized_vote.len() as i32)])?;

    println!("after invoke, value is {:?}", result[0].i32().unwrap());

    Ok(())

    // sample 2
    // println!("invoke poll_vote...");
    //
    // let init_vote_poll_state = VotePollState {
    //     value: 22,
    //     state: Default::default(),
    //     event: Poll("kingsgg".to_string()),
    // };
    // let serialized: Vec<u8> = bincode::serialize(&init_vote_poll_state).unwrap();
    //
    // println!("init contract...");
    // let init = instance.exports.get_function("allocate")?;
    // let result = init.call(&mut store, &[Value::I32(100)])?;
    // let ptr = result[0].i32().unwrap();
    // let wasm_ptr = WasmPtr::<i32>::new(ptr as u32);
    //
    // let memory = instance.exports.get_memory("memory")?;
    // let memory_view = memory.view(&store);
    //
    // let values = wasm_ptr.slice(&memory_view, serialized.len() as u32).unwrap();
    // for i in 0..serialized.len() {
    //     values.index(i as u64).write(serialized[i] as i32).unwrap();
    // }
    //
    // println!("invoke poll_vote...");
    // let poll_vote = instance.exports.get_function("poll_vote")?;
    // let result = poll_vote.call(&mut store, &[Value::I32(ptr), I32(serialized.len() as i32)])?;
    // let ptr = result[0].i32().unwrap();
    //
    // let wasm_ptr = WasmPtr::<i32>::new(ptr as u32);
    //
    // let length = instance.exports.get_function("get_length")?;
    // let result = length.call(&mut store, &[])?;
    // let len = result[0].i32().unwrap();
    // println!("{:?}", len);
    // let memory = instance.exports.get_memory("memory")?;
    // let memory_view = memory.view(&store);
    //
    // let values = wasm_ptr.slice(&memory_view, len as u32).unwrap();
    // let x = values.read_to_vec().unwrap();
    // println!("{:?}", x);
    //
    // let vec_u8: Vec<u8> = x.iter().map(|&x| x as u8).collect();
    // let deserialized: VotePollState = bincode::deserialize(&vec_u8).unwrap();
    //
    // println!("{:?}", deserialized);
    //
    // Ok(())
}