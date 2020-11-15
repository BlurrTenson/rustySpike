use crate::rbn::RBNState;
use crate::temp::Temperature;
use crate::util::bonding::{BondingSite, IsBondable, IsSubSymbolic, Stability};
use crate::util::cycle_calc::IsSynchronous;
use crate::util::formatters::IsFormatable;
use std::fmt;
use std::fmt::Display;

///Component is the generic trait for anything that can act in the subsymbolic system, both atomic
///structures and composite are components
pub trait Component: IsBondable + IsSynchronous + IsSubSymbolic + IsFormatable + Display {}

pub struct Particle {
    pub components: Vec<Box<dyn Component>>,
    pub bonding_sites: Vec<BondingSite>,
}

impl Particle {
    pub fn new(comp: Vec<Box<dyn Component>>) -> Particle {
        //TODO: go through comp and full out all bonding_sites
        // for mut component in comp {
        //     component.generate_bonding_sites();
        // }

        Particle {
            components: comp,
            bonding_sites: Vec::new(),
        }
        .gen_bonds()
    }

    fn gen_bonds(mut self) -> Self {
        for idx in 0..self.components.len() {
            self.components[idx].generate_bonding_sites();
        }
        self
    }
}

impl Component for Particle {}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut form_string = String::new();
        write!(f, "{}", form_string)
    }
}

impl IsFormatable for Particle {
    /// Returns a header for the formatted output
    fn fmt_header(&self) -> String {
        return String::new();
    }
    /// Returns the current state of the component in accordance to the formattingin the header
    fn fmt_state(&self) -> String {
        return String::new();
    }

    /// Returns cycle liviliness TODO rename to generic version
    fn fmt_cycle_liveliness(&self) -> String {
        return String::new();
    }

    /// Returns transient liveliness TODO rename to generic version
    fn fmt_trans_liveliness(&self) -> String {
        return String::new();
    }
}

impl IsSubSymbolic for Particle {
    fn calculate_particle(&mut self, init_state: Temperature, verbose: bool) -> Stability {
        return Stability::Stable;
    }
}

impl IsSynchronous for Particle {
    /// Step calculates next state as a function of current state
    fn step(&self) -> RBNState {
        RBNState::from(0 as u64)
    }
    /// Sync changes current state to next state
    fn sync(&self) {}
}

impl IsBondable for Particle {
    /// Generates the bonding sites based on the underlying representatin
    fn generate_bonding_sites(&mut self) -> Vec<BondingSite> {
        return vec![];
    }

    /// Returns Bonding Property for a specific &BondingSite
    /// If the BondingSite is not present on the particle returns None
    fn get_bonding_prop(&self, bs: &BondingSite) -> Option<i32> {
        return None;
    }

    /// Returns pointers to all BondingSites on the Particle
    fn get_all_bonding_sites(&self) -> Vec<&BondingSite> {
        return vec![];
    }

    /// Returns pointers to all BondingSites not currently part of a bond on the Particle
    /// If there are no free sites returns None
    fn get_free_bonding_sites(&self) -> Option<Vec<&BondingSite>> {
        return None;
    }

    /// Returns a random bonding site not currently part of a bond
    /// If there are no free sites returns None
    fn get_rand_free_bonding_site(&self) -> Option<&BondingSite> {
        return None;
    }
}
