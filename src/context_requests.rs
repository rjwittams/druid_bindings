use druid::UpdateCtx;

pub trait ContextRequests {
    fn notify(ctx: &mut UpdateCtx);
}

impl ContextRequests for () {
    fn notify(ctx: &mut UpdateCtx) {
        ctx.request_layout();
    }
}

pub struct Layout;
impl ContextRequests for Layout {
    fn notify(ctx: &mut UpdateCtx) {
        ctx.request_layout();
    }
}

pub struct Paint;
impl ContextRequests for Paint {
    fn notify(ctx: &mut UpdateCtx) {
        ctx.request_layout();
    }
}

pub struct AnimFrame;
impl ContextRequests for AnimFrame {
    fn notify(ctx: &mut UpdateCtx) {
        ctx.request_layout();
    }
}

impl<T1: ContextRequests, T2: ContextRequests> ContextRequests for (T1, T2) {
    fn notify(ctx: &mut UpdateCtx) {
        T1::notify(ctx);
        T2::notify(ctx);
    }
}

impl<T1: ContextRequests, T2: ContextRequests, T3: ContextRequests> ContextRequests
    for (T1, T2, T3)
{
    fn notify(ctx: &mut UpdateCtx) {
        T1::notify(ctx);
        T2::notify(ctx);
        T3::notify(ctx);
    }
}
