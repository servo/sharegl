use geom::size::Size2D;
use base::ShareContext;

pub type Context = DummyContext;

struct DummyContext {
    unused: int
}

impl ShareContext for DummyContext {
    static fn new(_size: Size2D<int>) -> DummyContext {
        DummyContext {
            unused: 0
        }
    }

    fn flush(&self) {
    }

    fn id(&self) -> int {
        0
    }
}
