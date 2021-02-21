use crate::binding::LensPropBinding;
use crate::ContextRequests;
use druid::{Data, Env, EventCtx, Lens, Size, UpdateCtx, Widget};
use std::marker::PhantomData;

/// This represents a property (usually on a widget) that can be bound
pub trait Property: Sized {
    /// The controlled item - usually a widget.
    /// Its not constrained to a widget as it could be some subpart of it.
    type Controlled;
    /// The type of the property
    type Value;

    /// The type of a change to the property.
    /// This is its own type to allow conflation of small changes to large values.
    /// If that kind of optimisation isn't needed (the default case) it can simply be (), so Some(())
    /// indicates that there has been a change, and the field value can be accessed
    /// directly in update_data_from_change.
    type Change;

    /// This is a marker type indicating what request methods should be called on
    /// the UpdateCtx
    type Requests: ContextRequests;

    /// Write the value from a data change to the property on the controlled item.
    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        env: &Env,
    );

    /// Modify the change parameter to include any additional changes.
    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        env: &Env,
    );

    /// Update the mutable 'field' reference with the changes accrued in 'change'.
    /// The controlled value can also be copied over directly if that is efficient enough.
    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        change: Self::Change,
        env: &Env,
    );

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        env: &Env,
    );

    fn read(self) -> ReadOnlyProperty<Self> {
        ReadOnlyProperty(self)
    }

    fn write(self) -> WriteOnlyProperty<Self> {
        WriteOnlyProperty(self)
    }

    fn with<T, L: Lens<T, Self::Value>>(
        self,
        lens: L,
    ) -> LensPropBinding<T, Self::Controlled, Self::Value, L, Self> {
        LensPropBinding::new(lens, self)
    }
}

pub struct ReadOnlyProperty<B>(pub B);

impl<B: Property> Property for ReadOnlyProperty<B> {
    type Controlled = B::Controlled;
    type Value = B::Value;
    type Change = B::Change;
    type Requests = ();

    fn write_prop(
        &self,
        _controlled: &mut Self::Controlled,
        _ctx: &mut UpdateCtx,
        _field_val: &Self::Value,
        _env: &Env,
    ) {
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        env: &Env,
    ) {
        self.0.append_changes(controlled, field_val, change, env)
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        change: Self::Change,
        env: &Env,
    ) {
        self.0
            .update_data_from_change(controlled, ctx, field, change, env)
    }

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        env: &Env,
    ) {
        self.0.initialise_data(controlled, ctx, field, env)
    }
}

pub struct WriteOnlyProperty<B>(pub B);

impl<B: Property> Property for WriteOnlyProperty<B> {
    type Controlled = B::Controlled;
    type Value = B::Value;
    type Change = B::Change;
    type Requests = B::Requests;

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        env: &Env,
    ) {
        self.0.write_prop(controlled, ctx, field_val, env)
    }

    fn append_changes(
        &self,
        _controlled: &Self::Controlled,
        _field_val: &Self::Value,
        _change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
    }

    fn update_data_from_change(
        &self,
        _controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        _field: &mut Self::Value,
        _change: Self::Change,
        _env: &Env,
    ) {
    }

    fn initialise_data(
        &self,
        _controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        _field: &mut Self::Value,
        _env: &Env,
    ) {
    }
}

// This wrapper exists to have blanket impls of Property for the various convenience property types.

// It takes an extra type argument (Marker) in order to disambiguate trait impls.
// This is generally a unit struct.
pub struct PropertyWrapper<Marker, Property> {
    marker: PhantomData<Marker>,
    wrapped: Property,
}

impl<Marker, Property> PropertyWrapper<Marker, Property> {
    pub const fn new(wrapped: Property) -> Self {
        PropertyWrapper {
            marker: PhantomData,
            wrapped,
        }
    }
}

pub struct Value;
pub trait ValueProperty {
    type Controlled;
    type Value: Data;
    type Requests: ContextRequests;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value);
    fn read(&self, controlled: &Self::Controlled) -> Self::Value;
}

impl<TP: ValueProperty> Property for PropertyWrapper<Value, TP> {
    type Controlled = TP::Controlled;
    type Value = TP::Value;
    type Change = ();
    type Requests = TP::Requests;

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        _ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        _env: &Env,
    ) {
        self.wrapped.write(controlled, field_val)
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
        let val = self.wrapped.read(controlled);
        if !val.same(field_val) {
            *change = Some(())
        }
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        _change: Self::Change,
        env: &Env,
    ) {
        self.initialise_data(controlled, ctx, field, env);
    }

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        field: &mut Self::Value,
        _env: &Env,
    ) {
        let val = self.wrapped.read(controlled);
        if !val.same(&field) {
            *field = val;
        }
    }
}

pub struct Ref;
pub trait RefProperty {
    type Controlled;
    type Value: Data;
    type Requests: ContextRequests;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value);
    fn read(&self, controlled: &Self::Controlled) -> Option<&Self::Value>;
}

impl<RP: RefProperty> Property for PropertyWrapper<Ref, RP> {
    type Controlled = RP::Controlled;
    type Value = RP::Value;
    type Change = ();
    type Requests = RP::Requests;

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        _ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        _env: &Env,
    ) {
        self.wrapped.write(controlled, field_val);
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
        if let Some(item) = self.wrapped.read(controlled) {
            if !item.same(field_val) {
                *change = Some(())
            }
        }
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        _change: Self::Change,
        _env: &Env,
    ) {
        self.initialise_data(controlled, ctx, field, _env);
    }

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        field: &mut Self::Value,
        _env: &Env,
    ) {
        if let Some(item) = self.wrapped.read(controlled) {
            *field = item.clone()
        }
    }
}

pub struct Writing;
pub trait WritingProperty {
    type Controlled;
    type Value: Data;
    type Requests: ContextRequests;

    fn write(&self, controlled: &mut Self::Controlled, value: &Self::Value);
}

impl<S: WritingProperty> Property for PropertyWrapper<Writing, S> {
    type Controlled = S::Controlled;
    type Value = S::Value;
    type Change = ();
    type Requests = S::Requests;

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        _ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        _env: &Env,
    ) {
        self.wrapped.write(controlled, field_val);
    }

    fn append_changes(
        &self,
        _controlled: &Self::Controlled,
        _field_val: &Self::Value,
        _change: &mut Option<Self::Change>,
        _env: &Env,
    ) {
    }

    fn update_data_from_change(
        &self,
        _controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        _field: &mut Self::Value,
        _change: Self::Change,
        _env: &Env,
    ) {
    }

    fn initialise_data(
        &self,
        _controlled: &Self::Controlled,
        _ctx: &mut EventCtx,
        _field: &mut Self::Value,
        _env: &Env,
    ) {
    }
}

struct SizeProperty<T, W>(PhantomData<T>, PhantomData<W>);

impl<W: Widget<T>, T: Data> Property for SizeProperty<T, W> {
    type Controlled = W;
    type Value = Size;
    type Change = ();
    type Requests = ();

    fn write_prop(
        &self,
        controlled: &mut Self::Controlled,
        ctx: &mut UpdateCtx,
        field_val: &Self::Value,
        env: &Env,
    ) {
    }

    fn append_changes(
        &self,
        controlled: &Self::Controlled,
        field_val: &Self::Value,
        change: &mut Option<Self::Change>,
        env: &Env,
    ) {
        *change = Some(())
    }

    fn update_data_from_change(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        change: Self::Change,
        env: &Env,
    ) {
        if *field != ctx.size() {
            *field = ctx.size()
        }
    }

    fn initialise_data(
        &self,
        controlled: &Self::Controlled,
        ctx: &mut EventCtx,
        field: &mut Self::Value,
        env: &Env,
    ) {
    }
}
