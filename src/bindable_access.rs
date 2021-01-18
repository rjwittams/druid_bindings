/// This trait indicates that a class is a wrapper of another widget that may have API you wish to access.
/// Used by BindingHost to "reach inside" things like LensWrapped in order to find the right widget to control,
/// given the bindings that it has.
///
/// Widgets that expect to be "bound" should implement this trait and return themselves.
/// Widgets that wrap another widget and do not provide anything likely to need binding should
/// recurse and call the same method on their inner widget.
///
/// This scheme isn't perfect as it stops at the first Bindable in all cases. However it covers the common case of
/// binding something that is already lensed
pub trait BindableAccess {
    /// What is the wrapped type being accessed through this implementation
    type Wrapped;
    /// Get immutable access to the wrapped instance.
    fn bindable(&self) -> &Self::Wrapped;
    /// Get mutable access to the wrapped instance.
    fn bindable_mut(&mut self) -> &mut Self::Wrapped;
}

/// We can't blanket implement these because of coherence
#[macro_export]
macro_rules! bindable_wrapper_body{
    () => {
        type Wrapped = W::Wrapped;

        fn bindable(&self) -> &Self::Wrapped {
            self.wrapped().bindable()
        }

        fn bindable_mut(&mut self) -> &mut Self::Wrapped {
            self.wrapped_mut().bindable_mut()
        }
    }
}

#[macro_export]
macro_rules! bindable_self_body{
    () => {
        type Wrapped = Self;

        fn bindable(&self) -> &Self::Wrapped {
            self
        }

        fn bindable_mut(&mut self) -> &mut Self::Wrapped {
            self
        }
    }
}