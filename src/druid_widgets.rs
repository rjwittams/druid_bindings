use crate::bindable_access::*;
use crate::property::{PropertyWrapper, Value, ValueProperty, Writing, WritingProperty};
use crate::{Layout, Paint, Property};
use druid::kurbo::Rect;
use druid::text::TextStorage;
use druid::widget::prelude::*;
use druid::widget::{
    Axis, ClipBox, IdentityWrapper, Label, LensWrap, LineBreaking, RawLabel, Scope, ScopePolicy,
    Scroll, Tabs, TabsPolicy, WidgetWrapper,
};
use druid::{Color, TextAlignment};
use std::marker::PhantomData;
use std::sync::Arc;

impl<W: BindableAccess> BindableAccess for IdentityWrapper<W> {
    bindable_wrapper_body!();
}

impl<T, U, L, W: BindableAccess> BindableAccess for LensWrap<T, U, L, W> {
    bindable_wrapper_body!();
}

impl<SP: ScopePolicy, W: Widget<SP::State> + BindableAccess> BindableAccess for Scope<SP, W> {
    bindable_wrapper_body!();
}

impl<TP: TabsPolicy> BindableAccess for Tabs<TP> {
    bindable_self_body!();
}

impl<T, W> BindableAccess for Scroll<T, W> {
    bindable_self_body!();
}

impl<T, W> BindableAccess for ClipBox<T, W> {
    bindable_self_body!();
}

impl<T> BindableAccess for Label<T> {
    bindable_self_body!();
}

impl<T> BindableAccess for RawLabel<T> {
    bindable_self_body!();
}

pub trait HasAxisPosition {
    fn set_axis_position(&mut self, direction: Axis, position: f64) -> bool;
    fn get_axis_position(&self, direction: Axis) -> f64;
    fn get_axis_limit(&self, direction: Axis) -> f64;
}

impl<T, W: Widget<T>> HasAxisPosition for Scroll<T, W> {
    fn set_axis_position(&mut self, axis: Axis, position: f64) -> bool {
        self.scroll_to_on_axis(axis, position)
    }

    fn get_axis_position(&self, axis: Axis) -> f64 {
        self.offset_for_axis(axis)
    }

    fn get_axis_limit(&self, direction: Axis) -> f64 {
        direction.major(self.child_size() - self.viewport_rect().size())
    }
}

impl<T, W: Widget<T>> HasAxisPosition for ClipBox<T, W> {
    fn set_axis_position(&mut self, axis: Axis, position: f64) -> bool {
        self.pan_to_on_axis(axis, position)
    }

    fn get_axis_position(&self, axis: Axis) -> f64 {
        axis.major_pos(self.viewport_origin())
    }

    fn get_axis_limit(&self, direction: Axis) -> f64 {
        direction.major(self.content_size() - self.viewport_rect().size())
    }
}

/// A bindable property to allow scroll offsets to be linked to app data.
/// Useful within composite components with linked scroll areas (eg tables)
pub struct AxisPositionProperty<H> {
    direction: Axis,
    phantom_h: PhantomData<H>,
}

impl<H> AxisPositionProperty<H> {
    pub const VERTICAL: PropertyWrapper<Value, Self> =
        PropertyWrapper::new(Self::new(Axis::Vertical));
    pub const HORIZONTAL: PropertyWrapper<Value, Self> =
        PropertyWrapper::new(Self::new(Axis::Horizontal));

    /// Create a Axis Position property for the specified axis.
    pub const fn new(direction: Axis) -> Self {
        AxisPositionProperty {
            direction,
            phantom_h: PhantomData,
        }
    }
}

impl<H: HasAxisPosition> ValueProperty for AxisPositionProperty<H> {
    type Controlled = H;
    type Value = f64;
    type Requests = Paint;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        controlled.set_axis_position(self.direction, *value);
    }

    fn read(&self, controlled: &Self::Controlled) -> Self::Value {
        controlled.get_axis_position(self.direction)
    }
}

pub struct AxisFractionProperty<H> {
    written: Option<f64>,
    direction: Axis,
    phantom_h: PhantomData<H>,
}

impl<H> AxisFractionProperty<H> {
    /// Create a Axis Position property for the specified axis.
    pub const fn new(direction: Axis) -> Self {
        AxisFractionProperty {
            written: None,
            direction,
            phantom_h: PhantomData,
        }
    }

    pub fn vertical() -> impl Property<Controlled = H, Value = f64>
    where
        H: HasAxisPosition,
    {
        PropertyWrapper::<Value, _>::new(Self::new(Axis::Vertical))
    }
}

impl<H: HasAxisPosition> ValueProperty for AxisFractionProperty<H> {
    type Controlled = H;
    type Value = f64;
    type Requests = Paint;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        let limit = controlled.get_axis_limit(self.direction);

        controlled.set_axis_position(self.direction, (*value) * limit);
    }

    fn read(&self, controlled: &Self::Controlled) -> Self::Value {
        let limit = controlled.get_axis_limit(self.direction);
        let pos = controlled.get_axis_position(self.direction);

        ((pos / limit) * 1_000.).floor() / 1_000.
    }
}

pub struct ReadScrollRect<T, W> {
    phantom_t: PhantomData<T>,
    phantom_w: PhantomData<W>,
}

impl<T, W> ReadScrollRect<T, W> {
    pub const PROP: PropertyWrapper<Value, Self> = PropertyWrapper::new(Self {
        phantom_t: PhantomData,
        phantom_w: PhantomData,
    });
}

impl<T, W: Widget<T>> ValueProperty for ReadScrollRect<T, W> {
    type Controlled = Scroll<T, W>;
    type Value = Rect;
    type Requests = ();

    fn write(&self, _controlled: &mut Self::Controlled, _value: &Self::Value) {
        // Read only
    }

    fn read(&self, controlled: &Self::Controlled) -> Self::Value {
        controlled.viewport_rect()
    }
}

pub struct RawLabelProps<T>(PhantomData<*const T>);

impl<T: TextStorage> RawLabelProps<T> {
    pub const text_color: PropertyWrapper<Writing, RawLabelTextColor<T>> =
        PropertyWrapper::new(RawLabelTextColor(PhantomData));
    pub const text_alignment: PropertyWrapper<Writing, RawLabelTextAlignment<T>> =
        PropertyWrapper::new(RawLabelTextAlignment(PhantomData));
    pub const line_break_mode: PropertyWrapper<Writing, RawLabelLineBreakMode<T>> =
        PropertyWrapper::new(RawLabelLineBreakMode(PhantomData));
}

pub struct RawLabelTextColor<T>(PhantomData<*const T>);
impl<T: TextStorage> WritingProperty for RawLabelTextColor<T> {
    type Controlled = RawLabel<T>;
    type Value = Color;
    type Requests = Layout;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        controlled.set_text_color(value.clone());
    }
}

pub struct RawLabelTextAlignment<T>(PhantomData<*const T>);
impl<T: TextStorage> WritingProperty for RawLabelTextAlignment<T> {
    type Controlled = RawLabel<T>;
    type Value = TextAlignment;
    type Requests = Layout;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        controlled.set_text_alignment(*value);
    }
}

pub struct RawLabelLineBreakMode<T>(PhantomData<*const T>);
impl<T: TextStorage> WritingProperty for RawLabelLineBreakMode<T> {
    type Controlled = RawLabel<T>;
    type Value = LineBreaking;
    type Requests = Layout;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        controlled.set_line_break_mode(*value);
    }
}

pub struct LabelAsRaw<T, RLProp>(PhantomData<*const T>, RLProp);
impl<T, RLProp> LabelAsRaw<T, RLProp> {
    const fn new(prop: RLProp) -> Self {
        LabelAsRaw(PhantomData, prop)
    }
}

impl<T, RLProp: Property<Controlled = RawLabel<Arc<str>>>> Property for LabelAsRaw<T, RLProp> {
    type Controlled = Label<T>;
    type Value = RLProp::Value;
    type Change = RLProp::Change;
    type Requests = RLProp::Requests;

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        env: &Env,
    ) {
        self.1.write_prop(controlled, ctx, field_val, env)
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        env: &Env,
    ) {
        self.1.append_changes(controlled, field_val, change, env)
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        change: Self::Change,
        env: &Env,
    ) {
        self.1
            .update_data_from_change(controlled, ctx, field, change, env)
    }

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        env: &Env,
    ) {
        self.1.initialise_data(controlled, ctx, field, env)
    }
}

pub struct LabelProps<T>(PhantomData<*const T>);

impl<T> LabelProps<T> {
    pub const text_color: LabelAsRaw<T, PropertyWrapper<Writing, RawLabelTextColor<Arc<str>>>> =
        LabelAsRaw::new(PropertyWrapper::new(RawLabelTextColor(PhantomData)));
    pub const text_alignment: LabelAsRaw<
        T,
        PropertyWrapper<Writing, RawLabelTextAlignment<Arc<str>>>,
    > = LabelAsRaw::new(PropertyWrapper::new(RawLabelTextAlignment::<Arc<str>>(
        PhantomData,
    )));
    pub const line_break_mode: LabelAsRaw<
        T,
        PropertyWrapper<Writing, RawLabelLineBreakMode<Arc<str>>>,
    > = LabelAsRaw::new(PropertyWrapper::new(RawLabelLineBreakMode(PhantomData)));
}

pub struct TabsProps<T>(PhantomData<*const T>);

impl<T> TabsProps<T> {
    pub const tab_index: PropertyWrapper<Value, TabIndex<T>> =
        PropertyWrapper::new(TabIndex(PhantomData));
}

pub struct TabIndex<T>(PhantomData<*const T>);

impl<TP: TabsPolicy> ValueProperty for TabIndex<TP> {
    type Controlled = Tabs<TP>;
    type Value = usize;
    type Requests = Layout;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value) {
        controlled.set_tab_index(*value)
    }

    fn read(&self, controlled: &Self::Controlled) -> Self::Value {
        controlled.tab_index()
    }
}
