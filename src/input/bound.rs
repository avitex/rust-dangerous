/// Indication of whether [`Input`](crate::Input) will change in futher passes.
///
/// Used for retry functionality if enabled.
#[must_use]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Bound {
    /// Both sides of the [`Input`](crate::Input) may change in further passes.
    None,
    /// The start of the [`Input`](crate::Input) in further passes will not change.
    ///
    /// The end of the [`Input`](crate::Input) may however change in further passes.
    Start,
    /// Both sides of the [`Input`](crate::Input) in further passes will not change.
    StartEnd,
}

impl Bound {
    #[inline(always)]
    pub(crate) fn force_close() -> Self {
        Bound::StartEnd
    }

    /// An end is opened when it is detected a `take_consumed` reader could have
    /// continued.
    #[inline(always)]
    pub(crate) fn open_end(self) -> Self {
        match self {
            // If at least the start is bound make sure the end is unbound.
            Bound::StartEnd | Bound::Start => Bound::Start,
            // If the start is unbound both sides of the input are unbound.
            Bound::None => Bound::None,
        }
    }

    /// An end is closed when a known length of input is sucessfully taken.
    #[inline(always)]
    pub(crate) fn close_end(self) -> Self {
        // We don't care if the input has no bounds. The only place input with
        // no bounds can originate is when a reader has reached the end of input
        // and could have consumed more. In other words - input with no bounds
        // is always empty. A length of zero taken from input with no bounds
        // will always succeed but the first half will have both sides bound to
        // prevent deadlocks.
        let _ = self;
        Bound::force_close()
    }

    #[inline(always)]
    pub(crate) fn for_end(self) -> Self {
        match self {
            // If both sides are bounded nothing will change.
            Bound::StartEnd => Bound::StartEnd,
            // As we have skipped to the end without checking, we don't know
            // where the start is, perhaps the true end is not known yet!
            Bound::Start | Bound::None => Bound::None,
        }
    }
}
