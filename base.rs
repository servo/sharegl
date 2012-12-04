use geom::size::Size2D;

pub trait ShareContext {
    // Creates a new context for GL object sharing.
    static fn new(size: Size2D<int>) -> self;

    // Flushes the context.
    fn flush(&self);

    // Returns the platform-specific ID that can be passed to other processes to access the shared
    // resources.
    fn id() -> int;
}

