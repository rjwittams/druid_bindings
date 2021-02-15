use crate::bindable_access::*;
use crate::binding::*;
use druid::kurbo::Rect;
use druid::widget::prelude::*;
use druid::widget::{Axis, IdentityWrapper, LensWrap, Scroll, WidgetWrapper};
use std::marker::PhantomData;

impl<W: BindableAccess> BindableAccess for IdentityWrapper<W> {
    bindable_wrapper_body!();
}

impl<T, U, L, W: BindableAccess> BindableAccess for LensWrap<T, U, L, W> {
    bindable_wrapper_body!();
}

impl<T, W> BindableAccess for Scroll<T, W> {
    bindable_self_body!();
}

/// A bindable property to allow scroll offsets to be linked to app data.
/// Useful within composite components with linked scroll areas (eg tables)
pub struct ScrollToProperty<T, W> {
    direction: Axis,
    phantom_t: PhantomData<T>,
    phantom_w: PhantomData<W>,
}

impl<T, W> ScrollToProperty<T, W> {
    /// Create a Scroll To property for the specified axis.
    pub fn new(direction: Axis) -> Self {
        ScrollToProperty {
            direction,
            phantom_t: Default::default(),
            phantom_w: Default::default(),
        }
    }
}

impl<T, W: Widget<T>> BindableProperty for ScrollToProperty<T, W> {
    type Controlled = Scroll<T, W>;
    type Value = f64;
    type Change = ();

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        ctx: &mut UpdateCtx,
        position: &Self::Value,
        _env: &Env,
    ) {
        controlled.scroll_to_on_axis(self.direction, *position);
        ctx.request_paint()
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
        if !controlled.offset_for_axis(self.direction).same(field_val) {
            log::info!("Scroll change in binding");
            *change = Some(())
        }
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        _ctx: &EventCtx,
        field: &mut Self::Value,
        _change: Self::Change,
        _env: &Env,
    ) {
        *field = controlled.offset_for_axis(self.direction)
    }
}

pub struct ScrollRectProperty<T, W> {
    phantom_t: PhantomData<T>,
    phantom_w: PhantomData<W>,
}

impl<T, W> Default for ScrollRectProperty<T, W> {
    fn default() -> Self {
        Self {
            phantom_t: Default::default(),
            phantom_w: Default::default(),
        }
    }
}

impl<T, W: Widget<T>> BindableProperty for ScrollRectProperty<T, W> {
    type Controlled = Scroll<T, W>;
    type Value = Rect;
    type Change = ();

    fn write_prop(
        &self,
        _controlled: &mut Self::Controlled,
        _ctx: &mut UpdateCtx,
        _position: &Self::Value,
        _env: &Env,
    ) {
        //controlled.scroll_to_on_axis(self.direction, *position);
        //ctx.request_paint()
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
        if !controlled.viewport_rect().same(field_val) {
            log::info!("Scroll rect change in binding");
            *change = Some(())
        }
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        _ctx: &EventCtx,
        field: &mut Self::Value,
        _change: Self::Change,
        _env: &Env,
    ) {
        *field = controlled.viewport_rect()
    }
}
