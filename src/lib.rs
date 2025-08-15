#![feature(trivial_bounds)]

use ark_ff::BigInt;

pub mod affine;
mod arithmetics;
pub mod group;
pub mod xsk233;

fn bigint_to_le_bytes(scalar: BigInt<4>) -> Vec<u8> {
    let limbs = scalar.0;

    let mut bytes = Vec::with_capacity(32);
    for limb in limbs.iter() {
        bytes.extend_from_slice(&limb.to_le_bytes());
    }
    bytes.truncate(30);

    // remove trailing zeros
    // helps reduce iteration in double-and-add iterations
    while let Some(&last) = bytes.last() {
        if last == 0 {
            bytes.pop();
        } else {
            break;
        }
    }
    bytes
}
