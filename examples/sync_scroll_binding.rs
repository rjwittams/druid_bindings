use druid::piet::{Color, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid_bindings::*;
use druid::widget::{
    Axis, DefaultScopePolicy, Flex, Label, Padding, Scope,
    Scroll, TextBox,
};
use druid::{AppLauncher, Data, Lens, LocalizedString, WidgetExt, WindowDesc};

#[derive(Data, Lens, Debug, Clone)]
struct OuterState {
    name: String,
    job: String,
}

impl OuterState {
    pub fn new(name: String, job: String) -> Self {
        OuterState { name, job }
    }
}

#[derive(Data, Lens, Debug, Clone)]
struct InnerState {
    text: String,
    font: String,
    scroll_y: f64,
}

impl InnerState {
    pub fn new(text: String) -> Self {
        InnerState {
            text,
            font: "Courier".into(),
            scroll_y: 0.0,
        }
    }
}

pub fn main() {
    let window = WindowDesc::new(build_widget)
        .window_size(Size::new(700.0, 300.0)) // build_inner_widget)
        .title(LocalizedString::new("scroll-demo-window-title").with_placeholder("Scroll demo"));
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(OuterState::new("Piet Mondrian".into(), "Artist".into()))
        //.launch(InnerState::new("bob".into()))
        .expect("launch failed");
}

#[derive(Lens)]
struct LensedWidget {
    font_name: String,
    text: String,
}

impl Bindable for LensedWidget {}

impl LensedWidget {
    pub fn new(font_name: String, text: String) -> Self {
        LensedWidget { font_name, text }
    }
}

impl Widget<String> for LensedWidget {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut String, _env: &Env) {}

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &String,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &String, _data: &String, _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &String,
        _env: &Env,
    ) -> Size {
        bc.constrain(Size::new(200.0, 100.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, _env: &Env) {
        let rect = ctx.region().bounding_box();
        ctx.fill(rect, &Color::WHITE);

        let try_font = ctx.text().font_family(&self.font_name);

        let (font, found) = match try_font {
            Some(font) => (font, true),
            _ => (ctx.text().font_family("Arial").unwrap(), false),
        };

        if let Ok(layout) = ctx
            .text()
            .new_text_layout(format!(
                "Data: {} Field: {} Font: {} Found: {}",
                data, self.text, self.font_name, found
            ))
            .max_width(200.0)
            .font(font, 15.0)
            .text_color(Color::BLACK)
            .build()
        {
            ctx.draw_text(&layout, (0.0, 0.0));
        }
    }
}

fn build_widget() -> impl Widget<OuterState> {
    let row = Flex::row()
        .with_child(TextBox::new().lens(OuterState::name))
        .with_child(TextBox::new().lens(OuterState::job));

    let scope = Scope::new(
        DefaultScopePolicy::from_lens(
            InnerState::new,  // How to construct the inner state from its input
            InnerState::text, // How to extract the input back out of the inner state
        ),
        build_inner_widget(), // Widgets operating on inner state
    )
    .lens(OuterState::job);

    row.with_flex_child(scope, 1.)
}

fn build_inner_widget() -> impl Widget<InnerState> {
    let mut row = Flex::row();

    let lensed = LensedWidget::new("Arial".into(), "Stuff".into())
        .lens(InnerState::text)
        .binding(
            // Bindings are bi directional- A lens from Data->Prop,  Prop<-Widget.
            InnerState::font
                .bind_lens(LensedWidget::font_name)
                // And combines bindings - they are syncing different props
                .and(InnerState::text.bind_lens(LensedWidget::text).forward()),
            // choose one direction, both ways is default
        );

    row.add_child(
        Flex::column()
            .with_child(TextBox::new().lens(InnerState::font))
            .with_child(lensed),
    );

    let follower = Scroll::new(make_col(1)).lens(InnerState::text).binding(
        InnerState::scroll_y
            .bind(ScrollToProperty::new(Axis::Vertical))
            .forward(),
    );

    row.add_flex_child(follower, 0.5);

    // You can have a series of wrappers before the binding, and it will reach inside and get
    // the nearest Bindable widget to control. The wrappers must trivially implement BindableAccess
    let leader = Scroll::new(make_col(0))
        .lens(InnerState::text)
        .with_id(WidgetId::next())
        .binding(InnerState::scroll_y.bind(ScrollToProperty::new(Axis::Vertical)));
    row.add_flex_child(leader, 0.5);

    row
}

fn make_col(i: i32) -> impl Widget<String> {
    let mut col = Flex::column();

    for j in 0..30 {
        if i == j {
            col.add_child(Padding::new(3.0, TextBox::new()));
        } else {
            col.add_child(Padding::new(
                3.0,
                Label::new(move |d: &String, _env: &_| format!("Label {}, {}, {}", i, j, d)),
            ));
        };
    }
    col
}
