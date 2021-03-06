extern crate bit_field;
extern crate rand;

use std::cell::{RefCell, RefMut};
use std::fmt;
use std::rc::Rc;
mod node;
mod particle;
mod rbn;
pub mod temp;
pub mod util;
use crate::particle::*;
use crate::util::bonding::{IsBondable, IsSubSymbolic};
use std::borrow::Borrow;

fn main() {
    let mut nds = Vec::new();

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(false);
    tbl.push(false);
    tbl.push(true);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(true);
    tbl.push(false);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(false);
    tbl.push(false);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(false);
    tbl.push(true);
    tbl.push(true);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(true);
    tbl.push(false);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(false);
    tbl.push(false);
    tbl.push(false);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(true);
    tbl.push(true);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(false);
    tbl.push(true);
    tbl.push(true);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(false);
    tbl.push(false);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(false);
    tbl.push(false);
    tbl.push(true);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(false);
    tbl.push(true);
    tbl.push(true);
    tbl.push(false);
    nds.push(tbl);

    let mut tbl = Vec::new();
    tbl.push(true);
    tbl.push(false);
    tbl.push(true);
    tbl.push(true);
    nds.push(tbl);

    let mut rbn_struct = Vec::<(usize, usize)>::new();
    rbn_struct.push((4, 5));
    rbn_struct.push((3, 5));
    rbn_struct.push((0, 10));
    rbn_struct.push((1, 4));
    rbn_struct.push((3, 4));
    rbn_struct.push((4, 6));
    rbn_struct.push((4, 8));
    rbn_struct.push((11, 4));
    rbn_struct.push((2, 3));
    rbn_struct.push((2, 11));
    rbn_struct.push((0, 5));
    rbn_struct.push((9, 8));
    // let mut newrbn = rbn::RBN::new(2, 12);
    let mut newrbn = rbn::RBN::new_from_def(nds, rbn_struct);
    newrbn.calculate_particle(0, true);
    print!("{}", newrbn);
    newrbn.generate_bonding_sites();
    println!("---------------------------------------");
    newrbn.generate_bonding_sites();
    let mut atom = particle::Particle {
        bonding_sites: newrbn.generate_bonding_sites(),
        components: vec![Rc::new(RefCell::new(newrbn))],
    };
    // println!("---------------------------------------");
    // let mut newrbn2 = rbn::RBN::new(2, 12);
    // newrbn2.calculate_particle(0, true);
    // print!("{}", newrbn2);
    // newrbn2.generate_interaction_groups_inf(23, false);
    // println!("---------------------------------------");
    // newrbn2.generate_interaction_groups_inf(23, true);

    // atom.components.push(Rc::new(RefCell::new(newrbn2)));
    // for _n in 1..20 {
    //     atom.components
    //         .push(Rc::new(RefCell::new(rbn::RBN::new(2, 12))));
    // }

    // for particle in atom.components {
    //     let mut brwRBN: RefMut<_> = particle.borrow_mut();

    //     println!("{}", brwRBN);
    //     eprintln!("{:?}", brwRBN.calculate_particle(0b000000000101, false));
    //     // println!("{}", brwRBN.fmt_cycle_liveliness());
    //     //  o
    //     // println!("{}", brwRBN.fmt_trans_liveliness());
    // }
    // newrbn.step();
    // newrbn.sync();
    // println!("{}", newrbn.fmt_state());
    // newrbn.step();
    // newrbn.sync();
    // println!("{}", newrbn.fmt_state());
    // newrbn.step();
    // newrbn.sync();
    // println!("{}", newrbn.fmt_state());
    // newrbn.step();
    // newrbn.sync();
    // println!("{}", newrbn.fmt_state());

    //let mut cl2 = 1;
    //let mut cl1 = 1;

    //while cl1 == cl2 {
    //    eprintln!("Searching for that special something...");
    //    cl1 = newrbn.calculate_cycle_ln(0b0101);
    //    cl2 = newrbn.calculate_cycle_ln(0b1010);
    //    newrbn = rbn::RBN::new(2, 12);
    //}
    //println!("{}, {}", cl1, cl2);
    //println!("{:#?}", newrbn);
}
