use crate::binding::Binding;
use crate::{BindableAccess, BindingHost};
use druid::Widget;

/// This trait provides combinators for building up bindings on widgets.
/// Would go on WidgetExt
pub trait WidgetBindingExt<T, U>: Widget<T> + Sized + BindableAccess
where
    Self::Wrapped: Widget<U>,
{
    /// Bind properties in this widget using the binding B
    fn binding<B: Binding<T, Self::Wrapped>>(
        self,
        binding: B,
    ) -> BindingHost<T, U, Self, Self::Wrapped, B> {
        BindingHost::new(self, binding)
    }
}

impl<T, U, W> WidgetBindingExt<T, U> for W
where
    W: Widget<T> + Sized + BindableAccess,
    Self::Wrapped: Widget<U>,
{
}
