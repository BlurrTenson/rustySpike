use crate::temp::Temperature;
///Particles are either stable or unstable depending on the sub-symbolic dynamics. Unstable
///particles undergo some stabalising action (such as decomposition)
/// TODO This needs to be redone correctly, I think an RBN_Properties stcuture is not a bad idea to
/// store the relevant bits of data that aren't stability specific (eg cycle and transient)
#[derive(Debug)]
pub enum Stability {
    Stable,
    Unstable { cycle: u64, transient: u64 },
}

pub struct BondingSite {}

/// A Particle which IsSubSymbolic must be recaluclated when system changes in order to determine
/// if the particle's internal state has changed  
pub trait IsSubSymbolic {
    fn calculate_particle(&mut self, init_state: Temperature) -> Stability;
}

/// A particle which is bondable has a number of bonding sites each of which some associated
/// Bonding Property
trait IsBondable {
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
