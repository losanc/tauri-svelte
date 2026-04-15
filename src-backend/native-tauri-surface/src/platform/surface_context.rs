pub trait CursorContext {
    /// Push a named cursor onto the cursor stack.
    fn push_cursor(&self, cursor: &str);

    /// Pop the top cursor from the cursor stack.
    /// Must be paired with a prior [`push_cursor`](Self::push_cursor) call.
    fn pop_cursor(&self);
}
