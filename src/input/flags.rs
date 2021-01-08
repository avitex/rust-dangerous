const TAG_BOUND: u8 = 0b1000_0000;
const TAG_ISSTR: u8 = 0b0100_0000;
const TAG_START_UNDETERMINED: u8 = 0b0010_0000;

#[derive(Copy, Clone)]
pub(super) struct Flags(u8);

impl Default for Flags {
    #[inline(always)]
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl Flags {
    #[inline(always)]
    pub(super) const fn new(bound: bool, is_str: bool) -> Self {
        Self(0).bound(bound).str(is_str)
    }

    #[inline]
    pub(super) const fn is_bound(self) -> bool {
        self.0 & TAG_BOUND == TAG_BOUND
    }

    #[inline]
    pub(super) const fn is_start_undetermined(self) -> bool {
        self.0 & TAG_START_UNDETERMINED == TAG_START_UNDETERMINED
    }

    #[inline]
    pub(super) const fn is_str(self) -> bool {
        self.0 & TAG_ISSTR == TAG_ISSTR
    }

    #[inline(always)]
    pub(super) const fn bound(mut self, value: bool) -> Self {
        self.0 = if value {
            self.0 | TAG_BOUND
        } else {
            self.0 & !TAG_BOUND
        };
        self
    }

    #[inline(always)]
    pub(super) const fn str(mut self, value: bool) -> Self {
        self.0 = if value {
            self.0 | TAG_ISSTR
        } else {
            self.0 & !TAG_ISSTR
        };
        self
    }

    #[inline(always)]
    pub(super) const fn start_undetermined(mut self, value: bool) -> Self {
        self.0 = if value {
            self.0 | TAG_START_UNDETERMINED
        } else {
            self.0 & !TAG_START_UNDETERMINED
        };
        self
    }
}
