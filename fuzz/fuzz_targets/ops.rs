#![no_main]
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary)]
#[derive(Debug)]
struct Target {
    ctor: Constructor,
    ops: Vec<Op>,
    dtor: Destructor,
}

#[derive(arbitrary::Arbitrary)]
#[derive(Debug)]
enum Constructor {
    New,
    FromVec(Vec<Box<i32>>),
}

#[derive(arbitrary::Arbitrary)]
#[derive(Debug)]
enum Op {
    Push(Box<i32>),
    TryPush(Box<i32>),
    Pop,
    AsMutSlice,
    Remove(usize),
    SwapPop(usize),
    SwapRemove(usize),
    Insert(usize, Box<i32>),
    TryInsert(usize, Box<i32>),
    IterMut(usize),
    Drain(usize, usize),
    Splice(usize, usize, Vec<Box<i32>>, usize),
    Truncate(usize),
    SplitOff(usize),
}

#[derive(arbitrary::Arbitrary)]
#[derive(Debug)]
enum Destructor {
    LetDrop,
    IntoVec,
}

use staticvec::StaticVec;

fn run(ctor: Constructor, ops: Vec<Op>) -> Option<StaticVec<Box<i32>, 16>> {
    let mut sv: StaticVec<Box<i32>, 16> = match ctor {
        Constructor::New => StaticVec::new(),
        Constructor::FromVec(v) => StaticVec::from_vec(v),
    };

    for op in ops {
        match op {
            Op::Push(b) if sv.len() < sv.capacity() => sv.push(b),
            Op::AsMutSlice => sv.as_mut_slice().into_iter().for_each(|x| **x = x.wrapping_add(1)),
            Op::TryPush(b) => drop(sv.try_push(b)),
            Op::Pop => drop(sv.pop()),
            Op::Remove(index) if index < sv.len() => drop(sv.remove(index)),
            Op::SwapPop(index) => drop(sv.swap_pop(index)),
            Op::SwapRemove(index) if index < sv.len() => drop(sv.swap_remove(index)),
            Op::Insert(index, item) if sv.len() < 16 && index < sv.len() => sv.insert(index, item),
            Op::TryInsert(index, item) => drop(sv.try_insert(index, item)),
            Op::IterMut(take) => {
                let i = sv.iter_mut();
                i.bounds_to_string();
                i.take(take).for_each(|x| **x = x.wrapping_add(1));
            }
            Op::Drain(start, end) if !(start > end || end > sv.len()) => drop(sv.drain(start..end)),
            Op::Splice(start, end, iterator, take) if !(start > end || end > sv.len()) => {
                sv.splice(start..end, iterator).take(take).for_each(drop);
            }
            Op::Truncate(index) => drop(sv.truncate(index)),
            Op::SplitOff(index) if false => drop(sv.split_off(index)),
            _ => return None,
        }
    }

    Some(sv)
}

fuzz_target!(|target: Target| {
    if let Some(sv) = run(target.ctor, target.ops) {
        match target.dtor {
            Destructor::LetDrop => drop(sv),
            Destructor::IntoVec => drop(sv.into_vec()),
        }
    }
});
