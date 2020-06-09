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
