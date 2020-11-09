use crate::rbn::RBN;
use crate::util::bonding::{BondingSite, IsBondable, IsSubSymbolic};
use crate::util::cycle_calc::IsSynchronous;
use crate::util::formatters::IsFormatable;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

///Component is the generic trait for anything that can act in the subsymbolic system, both atomic
///structures and composite are components
pub trait Component: IsBondable + IsSynchronous + IsSubSymbolic + IsFormatable + Display {}

pub struct Particle {
    pub components: Vec<Rc<RefCell<dyn Component>>>,
    pub bonding_sites: Vec<BondingSite>,
}

impl Particle {
    pub fn new(comp: Vec<Rc<RefCell<dyn Component>>>) -> Particle {
        //TODO: go through comp and full out all bonding_sites
        Particle {
            components: comp,
            bonding_sites: Vec::new(),
        }
    }
}
//impl Component for Particle {}
