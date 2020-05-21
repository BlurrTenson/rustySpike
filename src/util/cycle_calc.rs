use crate::rbn::RBNState;
use crate::temp::Temperature;
/// Sunchronos particles have to advance in step through their cycle detection
/// For that there is a Step -> Sync cycle.
/// All particles Call step()  which calculates and stores the next state given current state
/// All particles then call sync() which updates the current state with the calculated state
pub trait IsSynchronous {
    /// Step calculates next state as a function of current state
    fn step(&self) -> RBNState;
    /// Sync changes current state to next state
    fn sync(&self);
}

/// A particle that shows period cyclic behaviour.
/// Requires IsSynchronous trait (argubaly it doesn't you can have cyclic behaviour withought
/// syncronisation but we can deal with that probably never )  
pub trait IsCyclic: IsSynchronous {
    fn calculate_cycle_ln(&mut self, init_state: Temperature) -> u64;
    fn calculate_transient_ln(&self, init_state: Temperature);
    fn calculate_liveliness(&self, init_state: Temperature);
}
