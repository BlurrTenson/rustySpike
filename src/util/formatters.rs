/// A particle whic is Formattable can niceley print out its current state and core dynamic
/// properties with a bit of formatting.
/// TODO this may become save functionality in the future
pub trait IsFormatable {
    /// Returns a header for the formatted output
    fn fmt_header(&self) -> String;
    /// Returns the current state of the component in accordance to the formattingin the header
    fn fmt_state(&self) -> String;
    /// Returns cycle liviliness TODO rename to generic version
    fn fmt_cycle_liveliness(&self) -> String;
    /// Returns transient liveliness TODO rename to generic version
    fn fmt_trans_liveliness(&self) -> String;
}
