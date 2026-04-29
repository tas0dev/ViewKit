// Extension trait to provide .view() modifier for Components
// Allows both concrete Component types and boxed trait objects to produce a View builder.

pub trait ComponentExt {
    fn view(self) -> crate::components::View;
}

impl ComponentExt for Box<dyn crate::components::Component> {
    fn view(self) -> crate::components::View {
        crate::components::View::new(self)
    }
}

impl<T: crate::components::Component + 'static> ComponentExt for T {
    fn view(self) -> crate::components::View {
        crate::components::View::new(Box::new(self))
    }
}
