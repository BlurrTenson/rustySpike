use crate::rbn::RBN;
use crate::util::bonding::{IsBondable, IsSubSymbolic};
use crate::util::cycle_calc::IsSynchronous;
use crate::util::formatters::IsFormatable;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

///Component is the gener
pub trait Component: IsBondable + IsSynchronous + IsSubSymbolic + IsFormatable {}
pub struct Particle {
    pub components: Vec<Rc<RefCell<dyn Component>>>,
}
