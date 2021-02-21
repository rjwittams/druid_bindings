use crate::{BindableAccess, Binding};
use druid::{BoxConstraints, Command, CommandCtx, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Selector, Size, UpdateCtx, Widget, WidgetId, AnyCtx};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
enum BindingHostState {
    New,
    Init,
    TwoWay,
}

/// A binding host wraps a BindableAccess, and offers bindings from the Data at this stage of the hierarchy
/// to properties on that Bindable.
pub struct BindingHost<
    T,
    U,
    Contained: BindableAccess<Wrapped = Controlled> + Widget<T>,
    Controlled: Widget<U>,
    B: Binding<T, Controlled>,
> {
    contained: Contained,
    binding: B,
    pending_change: Option<B::Change>,
    state: BindingHostState,
    widget_id: Option<WidgetId>,
    phantom_u: PhantomData<U>,
}

impl<
        T,
        U,
        Contained: BindableAccess<Wrapped = Controlled> + Widget<T>,
        Controlled: Widget<U>,
        B: Binding<T, Controlled>,
    > BindingHost<T, U, Contained, Controlled, B>
{
    /// Create a binding host from a Widget and a Binding
    pub fn new(contained: Contained, binding: B) -> Self {
        BindingHost {
            contained,
            binding,
            pending_change: None,
            state: BindingHostState::New,
            widget_id: None,
            phantom_u: Default::default(),
        }
    }

    /// Add another binding, useful for method chaining of multiple bindings
    pub fn binding<BOther: Binding<T, Controlled>>(
        self,
        binding: BOther,
    ) -> BindingHost<T, U, Contained, Controlled, (B, BOther)> {
        BindingHost::new(self.contained, (self.binding, binding))
    }

    fn apply_pending_changes(&mut self, ctx: &mut EventCtx, data: &mut T, env: &Env) {
        if let Some(change) = self.pending_change.take() {
            self.binding
                .apply_change_to_data(self.contained.bindable(), data, change, ctx, env)
        }
    }

    fn check_for_changes2(&mut self, data: &T, env: &Env, ctx: &mut (impl CommandCtx + AnyCtx) ) {
        if let BindingHostState::TwoWay = self.state {
            self.binding.append_change_required(
                self.contained.bindable(),
                data,
                &mut self.pending_change,
                env,
            );
            if self.pending_change.is_some() {
                ctx.submit_command(APPLY_BINDINGS.to(ctx.widget_id()));
            }
        }
    }

    fn check_for_changes(&mut self, data: &T, env: &Env, mut submit_command: impl FnMut(Command)) {
        if let (BindingHostState::TwoWay, Some(widget_id)) = (self.state, self.widget_id) {
            self.binding.append_change_required(
                self.contained.bindable(),
                data,
                &mut self.pending_change,
                env,
            );
            if self.pending_change.is_some() {
                submit_command(APPLY_BINDINGS.to(widget_id));
            }
        }
    }
}

/// This command is sent to self in order to get the event method to run.
/// Event is the only method in which we can update the passed in data.
const APPLY_BINDINGS: Selector = Selector::new("druid-builtin.apply-bindings");
const INIT_BINDINGS: Selector = Selector::new("druid-builtin.init-bindings");

impl<
        OuterData: Data,
        InnerData,
        Contained: BindableAccess<Wrapped = Controlled> + Widget<OuterData>,
        Controlled: Widget<InnerData>,
        B: Binding<OuterData, Controlled>,
    > Widget<OuterData> for BindingHost<OuterData, InnerData, Contained, Controlled, B>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut OuterData, env: &Env) {
        match self.state {
            BindingHostState::New => {
                // When we are just created, do not want to read anything from the widget
                self.contained.event(ctx, event, data, env);
            }
            BindingHostState::Init => {
                // We use this command because the Lifecycle context
                // does not have request_update.
                // In this mode we still don't want to read anything from the widget
                match event {
                    Event::Command(cmd) if cmd.is(INIT_BINDINGS) => {
                        ctx.set_handled();
                        ctx.request_update();
                    }
                    _ => self.contained.event(ctx, event, data, env),
                }
            }
            BindingHostState::TwoWay => {
                // We are now in full binding mode
                self.apply_pending_changes(ctx, data, env);

                match event {
                    Event::Command(c) if c.is(APPLY_BINDINGS) => ctx.set_handled(),
                    _ => {
                        self.contained.event(ctx, event, data, env);
                    }
                };

                // Changes that occurred just now
                self.check_for_changes2(data, env, ctx);
            }
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &OuterData,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.state = BindingHostState::Init;
            self.widget_id = Some(ctx.widget_id());
            ctx.submit_command(INIT_BINDINGS.to(ctx.widget_id()));
        }

        self.contained.lifecycle(ctx, event, data, env);

        self.check_for_changes2(data, env, ctx);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &OuterData, data: &OuterData, env: &Env) {
        let apply_to_controlled = if let BindingHostState::Init = self.state {
            self.state = BindingHostState::TwoWay;
            true
        } else {
            !old_data.same(data)
        };

        if apply_to_controlled {
            self.binding
                .apply_data_to_controlled(data, self.contained.bindable_mut(), ctx, env);
        }

        self.contained.update(ctx, old_data, data, env);
        self.check_for_changes2(data, env, ctx);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &OuterData,
        env: &Env,
    ) -> Size {
        let size = self.contained.layout(ctx, bc, data, env);
        self.check_for_changes2(data, env, ctx);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &OuterData, env: &Env) {
        self.contained.paint(ctx, data, env);
        // Can't submit commands from here currently.
        // No point pending it yet
        // have to assume that any bound state will get picked up later
    }
}
