pub(crate) enum Alternate<T> {
    Back(T),
    Front(T),
}

/// Iterator that alternates between the inner iterators front and back item.
pub(crate) struct AlternatingIter<I> {
    inner: I,
    front: bool,
}

impl<I> AlternatingIter<I> {
    /// Create an iterator starting with the front item.
    pub(crate) fn front(iter: I) -> Self {
        Self {
            inner: iter,
            front: true,
        }
    }
}

impl<I> Iterator for AlternatingIter<I>
where
    I: DoubleEndedIterator,
{
    type Item = Alternate<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front {
            self.front = false;
            self.inner.next().map(Alternate::Front)
        } else {
            self.front = true;
            self.inner.next_back().map(Alternate::Back)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> ExactSizeIterator for AlternatingIter<I>
where
    I: ExactSizeIterator + DoubleEndedIterator,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}
