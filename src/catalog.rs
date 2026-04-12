#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiComponent {
    Button,
    Label,
    TextField,
    Checkbox,
    Slider,
    ListView,
    Stack,
    Image,
    ScrollView,
    Canvas,
}

impl UiComponent {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Button => "BUTTON",
            Self::Label => "LABEL",
            Self::TextField => "TEXTFIELD",
            Self::Checkbox => "CHECKBOX",
            Self::Slider => "SLIDER",
            Self::ListView => "LISTVIEW",
            Self::Stack => "STACK",
            Self::Image => "IMAGE",
            Self::ScrollView => "SCROLLVIEW",
            Self::Canvas => "CANVAS",
        }
    }
}

pub fn component_catalog() -> [UiComponent; 10] {
    [
        UiComponent::Button,
        UiComponent::Label,
        UiComponent::TextField,
        UiComponent::Checkbox,
        UiComponent::Slider,
        UiComponent::ListView,
        UiComponent::Stack,
        UiComponent::Image,
        UiComponent::ScrollView,
        UiComponent::Canvas,
    ]
}
