use crate::{ContextRequests, Property};

use druid::{Env, EventCtx, Lens, UpdateCtx};
use std::marker::PhantomData;

/// This is a two way binding between some data, and something it is controlling.
///
/// Usually this will be synchronising one bit of information in each,
/// eg one field of T bound to one 'property' of a controlled Widget.
///
/// You shouldn't need to implement this yourself, but rather use the two combinators below,
/// LensPropBinding and any nesting of 2-tuples of the same.
///
pub trait Binding<T, Controlled> {
    /// The type of built up change from internal widget state that needs to be applied to the data.
    type Change;

    /// Take the bound value from the data T and apply it to the controlled item (usually a widget)
    /// This will occur during update, as that is when data has changed.
    fn apply_data_to_controlled(
        &self,
        data: &T,
        controlled: &mut Controlled,
        ctx: &mut UpdateCtx,
        env: &Env,
    );

    /// Mutate the passed in Change option to indicate to the BindingHost that an update to the data will be needed.
    /// This could get called from any Widget method in BindingHost, and allows changes to data to be queued up for the next event.
    /// This has no ctxt argument, because there is no common trait between contexts
    fn append_change_required(
        &self,
        controlled: &Controlled,
        data: &T,
        change: &mut Option<Self::Change>,
        env: &Env,
    );

    /// This will take the built up Change from internal widget state, and apply it to the data.
    /// If it is possible to read that change directly from the controlled item, the Change type can be () and the Some-ness of it alone
    /// will be enough to trigger the change to be applied.
    fn apply_change_to_data(
        &self,
        controlled: &Controlled,
        data: &mut T,
        change: Self::Change,
        ctx: &mut EventCtx,
        env: &Env,
    );

    fn initialise_data(&self, controlled: &Controlled, data: &mut T, ctx: &mut EventCtx, env: &Env);
}

/// This implementation allows a tuple of bindings to act as a compound binding.
/// Because each side could also be a compound binding, this allows arbitrary composition of bindings that
/// between the same Data (T) and Widget (Controlled)
impl<T, Controlled, Bind1: Binding<T, Controlled>, Bind2: Binding<T, Controlled>>
    Binding<T, Controlled> for (Bind1, Bind2)
{
    type Change = (Option<Bind1::Change>, Option<Bind2::Change>);

    fn apply_data_to_controlled(
        &self,
        data: &T,
        controlled: &mut Controlled,
        ctx: &mut UpdateCtx,
        env: &Env,
    ) {
        self.0.apply_data_to_controlled(data, controlled, ctx, env);
        self.1.apply_data_to_controlled(data, controlled, ctx, env);
    }

    fn append_change_required(
        &self,
        controlled: &Controlled,
        data: &T,
        change: &mut Option<Self::Change>,
        env: &Env,
    ) {
        let (change0, change1) = change.get_or_insert_with(|| (None, None));
        self.0
            .append_change_required(controlled, data, change0, env);
        self.1
            .append_change_required(controlled, data, change1, env);
        if let Some((None, None)) = change {
            *change = None;
        }
    }

    fn apply_change_to_data(
        &self,
        controlled: &Controlled,
        data: &mut T,
        change: Self::Change,
        ctx: &mut EventCtx,
        env: &Env,
    ) {
        let (change0, change1) = change;

        if let Some(change0) = change0 {
            self.0
                .apply_change_to_data(controlled, data, change0, ctx, env);
        }

        if let Some(change1) = change1 {
            self.1
                .apply_change_to_data(controlled, data, change1, ctx, env);
        }
    }

    fn initialise_data(
        &self,
        controlled: &Controlled,
        data: &mut T,
        ctx: &mut EventCtx,
        env: &Env,
    ) {
        self.0.initialise_data(controlled, data, ctx, env);
        self.1.initialise_data(controlled, data, ctx, env);
    }
}

/// This binds a lens (LT) on some data (T) to a bindable property (PropC) on a widget (Controlled)
pub struct LensPropBinding<
    T,
    Controlled,
    PropValue,
    LT: Lens<T, PropValue>,
    PropC: Property<Controlled = Controlled, Value = PropValue>,
> {
    lens_from_data: LT,
    prop_from_controlled: PropC,
    phantom_t: PhantomData<*const T>,
    phantom_c: PhantomData<*const Controlled>,
    phantom_p: PhantomData<*const PropValue>,
}

impl<
        T,
        Controlled,
        PropValue,
        LT: Lens<T, PropValue>,
        PropC: Property<Controlled = Controlled, Value = PropValue>,
    > LensPropBinding<T, Controlled, PropValue, LT, PropC>
{
    /// Create a binding between a lens to data, and a property on a controlled item (usually a widget)
    pub fn new(lens_from_data: LT, prop_from_controlled: PropC) -> Self {
        LensPropBinding {
            lens_from_data,
            prop_from_controlled,
            phantom_t: PhantomData,
            phantom_c: PhantomData,
            phantom_p: PhantomData,
        }
    }
}

impl<
        T,
        Controlled,
        PropValue,
        LT: Lens<T, PropValue>,
        PropC: Property<Controlled = Controlled, Value = PropValue>,
    > Binding<T, Controlled> for LensPropBinding<T, Controlled, PropValue, LT, PropC>
{
    type Change = PropC::Change;

    fn apply_data_to_controlled(
        &self,
        data: &T,
        controlled: &mut Controlled,
        ctx: &mut UpdateCtx,
        env: &Env,
    ) {
        self.lens_from_data.with(data, |field_val| {
            self.prop_from_controlled
                .write_prop(controlled, ctx, field_val, env);
            PropC::Requests::notify(ctx)
        });
    }

    fn append_change_required(
        &self,
        controlled: &Controlled,
        data: &T,
        change: &mut Option<Self::Change>,
        env: &Env,
    ) {
        self.lens_from_data.with(data, |field_val| {
            self.prop_from_controlled
                .append_changes(controlled, field_val, change, env)
        })
    }

    fn apply_change_to_data(
        &self,
        controlled: &Controlled,
        data: &mut T,
        change: Self::Change,
        ctx: &mut EventCtx,
        env: &Env,
    ) {
        self.lens_from_data.with_mut(data, |field| {
            self.prop_from_controlled
                .update_data_from_change(controlled, ctx, field, change, env)
        })
    }

    fn initialise_data(
        &self,
        controlled: &Controlled,
        data: &mut T,
        ctx: &mut EventCtx,
        env: &Env,
    ) {
        self.lens_from_data.with_mut(data, |field| {
            self.prop_from_controlled
                .initialise_data(controlled, ctx, field, env)
        })
    }
}
