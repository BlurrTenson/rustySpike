use crate::node::Node;
use crate::temp::Temperature;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
///Particles are either stable or unstable depending on the sub-symbolic dynamics. Unstable
///particles undergo some stabalising action (such as decomposition)
/// TODO This needs to be redone correctly, I think an RBN_Properties stcuture is not a bad idea to
/// store the relevant bits of data that aren't stability specific (eg cycle and transient)
#[derive(Debug)]
pub enum Stability {
    Stable,
    Unstable { cycle: u64, transient: u64 },
}

#[derive(Debug)]
pub struct BondingSite {
    interaction_list: Vec<Rc<RefCell<Node>>>,
}

impl BondingSite {
    pub fn new(il: Vec<Rc<RefCell<Node>>>) -> BondingSite {
        BondingSite {
            interaction_list: il,
        }
    }
}

impl fmt::Display for BondingSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut form_string = String::new();
        form_string.push_str("[");
        for n in &self.interaction_list {
            form_string.push_str(&format!("{}, ", n.borrow().get_id()))
        }
        form_string.pop();
        form_string.pop();
        form_string.push_str("]");
        write!(f, "{}", form_string)
    }
}

/// A Particle which IsSubSymbolic must be recaluclated when system changes in order to determine
/// if the particle's internal state has changed  
pub trait IsSubSymbolic {
    fn calculate_particle(&mut self, init_state: Temperature, verbose: bool) -> Stability;
}

/// A particle which is bondable has a number of bonding sites each of which some associated
/// Bonding Property
pub trait IsBondable {
    /// Generates the bonding sites based on the underlying representatin
    fn generate_bonding_sites(&mut self) -> Vec<BondingSite>;

    /// Returns Bonding Property for a specific &BondingSite
    /// If the BondingSite is not present on the particle returns None
    fn get_bonding_prop(&self, bs: &BondingSite) -> Option<i32>;

    /// Returns pointers to all BondingSites on the Particle
    fn get_all_bonding_sites(&self) -> Vec<&BondingSite>;

    /// Returns pointers to all BondingSites not currently part of a bond on the Particle
    /// If there are no free sites returns None
    fn get_free_bonding_sites(&self) -> Option<Vec<&BondingSite>>;

    /// Returns a random bonding site not currently part of a bond
    /// If there are no free sites returns None
    fn get_rand_free_bonding_site(&self) -> Option<&BondingSite>;
}
