// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// ignore-linux #7340 fails on 32-bit Linux
// ignore-macos #7340 fails on 32-bit macos

use std::mem;

enum Tag<A,B> {
    VarA(A),
    VarB(B),
}

struct Rec<A,B> {
    chA: u8,
    tA: Tag<A,B>,
    chB: u8,
    tB: Tag<A,B>,
}

fn mk_rec<A,B>(a: A, b: B) -> Rec<A,B> {
    Rec { chA:0, tA:Tag::VarA(a), chB:1, tB:Tag::VarB(b) }
}

fn is_aligned<A>(amnt: uint, u: &A) -> bool {
    let p: uint = unsafe { mem::transmute(u) };
    return (p & (amnt-1)) == 0;
}

fn variant_data_is_aligned<A,B>(amnt: uint, u: &Tag<A,B>) -> bool {
    match u {
      &Tag::VarA(ref a) => is_aligned(amnt, a),
      &Tag::VarB(ref b) => is_aligned(amnt, b)
    }
}

pub fn main() {
    let x = mk_rec(22, 23);
    assert!(is_aligned(8, &x.tA));
    assert!(variant_data_is_aligned(8, &x.tA));
    assert!(is_aligned(8, &x.tB));
    assert!(variant_data_is_aligned(8, &x.tB));

    let x = mk_rec(22, 23);
    assert!(is_aligned(8, &x.tA));
    assert!(variant_data_is_aligned(8, &x.tA));
    assert!(is_aligned(8, &x.tB));
    assert!(variant_data_is_aligned(4, &x.tB));

    let x = mk_rec(22, 23);
    assert!(is_aligned(8, &x.tA));
    assert!(variant_data_is_aligned(4, &x.tA));
    assert!(is_aligned(8, &x.tB));
    assert!(variant_data_is_aligned(8, &x.tB));

    let x = mk_rec(22, 23);
    assert!(is_aligned(4, &x.tA));
    assert!(variant_data_is_aligned(4, &x.tA));
    assert!(is_aligned(4, &x.tB));
    assert!(variant_data_is_aligned(4, &x.tB));

    let x = mk_rec(22f64, 23f64);
    assert!(is_aligned(8, &x.tA));
    assert!(variant_data_is_aligned(8, &x.tA));
    assert!(is_aligned(8, &x.tB));
    assert!(variant_data_is_aligned(8, &x.tB));
}
