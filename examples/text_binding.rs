// Copyright 2020 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An example of various text layout features.

use druid::lens::Map;
use druid::text::{Attribute, RichText};
use druid::widget::{
    Axis, Flex, Label, LineBreaking, ProgressBar, RadioGroup, RawLabel, Scope, Scroll, Stepper,
    Tabs,
};
use druid::{
    AppLauncher, Color, Data, Env, EventCtx, FontFamily, FontStyle, FontWeight, Lens, LensExt,
    LocalizedString, TextAlignment, Widget, WidgetExt, WindowDesc,
};
use druid_bindings::{
    AxisFractionProperty, LabelProps, Property, RawLabelProps, TabsProps, WidgetBindingExt,
};

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Text Options");

const TEXT: &str = r#"Contrary to what we would like to believe, there is no such thing as a structureless group. Any group of people of whatever nature that comes together for any length of time for any purpose will inevitably structure itself in some fashion. The structure may be flexible; it may vary over time; it may evenly or unevenly distribute tasks, power and resources over the members of the group. But it will be formed regardless of the abilities, personalities, or intentions of the people involved. The very fact that we are individuals, with different talents, predispositions, and backgrounds makes this inevitable. Only if we refused to relate or interact on any basis whatsoever could we approximate structurelessness -- and that is not the nature of a human group.
This means that to strive for a structureless group is as useful, and as deceptive, as to aim at an "objective" news story, "value-free" social science, or a "free" economy. A "laissez faire" group is about as realistic as a "laissez faire" society; the idea becomes a smokescreen for the strong or the lucky to establish unquestioned hegemony over others. This hegemony can be so easily established because the idea of "structurelessness" does not prevent the formation of informal structures, only formal ones. Similarly "laissez faire" philosophy did not prevent the economically powerful from establishing control over wages, prices, and distribution of goods; it only prevented the government from doing so. Thus structurelessness becomes a way of masking power, and within the women's movement is usually most strongly advocated by those who are the most powerful (whether they are conscious of their power or not). As long as the structure of the group is informal, the rules of how decisions are made are known only to a few and awareness of power is limited to those who know the rules. Those who do not know the rules and are not chosen for initiation must remain in confusion, or suffer from paranoid delusions that something is happening of which they are not quite aware."#;

const SPACER_SIZE: f64 = 8.0;

#[derive(Clone, Data, Lens)]
struct AppState {
    line_break_mode: LineBreaking,
    alignment: TextAlignment,
    color: Color,
    tab_index: usize,
    scroll_pos: f64,
    raw_scroll_pos: f64,
}

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .window_size((450.0, 600.0));

    // create the initial app state
    let initial_state = AppState {
        line_break_mode: LineBreaking::WordWrap,
        alignment: TextAlignment::Start,
        color: Color::BLACK,
        tab_index: 0,
        scroll_pos: 0.,
        raw_scroll_pos: 0.,
    };

    // start the application
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn rich_text() -> RichText {
    RichText::new(TEXT.into())
        .with_attribute(0..9, Attribute::text_color(Color::rgb(1.0, 0.2, 0.1)))
        .with_attribute(0..9, Attribute::size(24.0))
        .with_attribute(0..9, Attribute::font_family(FontFamily::SERIF))
        .with_attribute(194..239, Attribute::weight(FontWeight::BOLD))
        .with_attribute(764.., Attribute::size(12.0))
        .with_attribute(764.., Attribute::style(FontStyle::Italic))
}

fn build_root_widget() -> impl Widget<AppState> {
    let label = Scroll::new(
        Label::new(TEXT)
            .with_text_color(Color::BLACK)
            .binding(LabelProps::text_alignment.with(AppState::alignment))
            .binding(LabelProps::line_break_mode.with(AppState::line_break_mode))
            .binding(LabelProps::text_color.with(AppState::color))
            .background(Color::WHITE)
            .expand_width()
            .padding((SPACER_SIZE * 4.0, SPACER_SIZE))
            .background(Color::grey8(222)),
    )
    .vertical()
    .binding(
        AxisFractionProperty::vertical()
            .read()
            .with(AppState::scroll_pos),
    );

    let raw_label = Scroll::new(
        Scope::isolate(rich_text(), RawLabel::new())
            .binding(RawLabelProps::text_alignment.with(AppState::alignment))
            .binding(RawLabelProps::line_break_mode.with(AppState::line_break_mode))
            .binding(RawLabelProps::text_color.with(AppState::color))
            .background(Color::WHITE)
            .expand_width()
            .padding((SPACER_SIZE * 4.0, SPACER_SIZE))
            .background(Color::grey8(222)),
    )
    .vertical()
    .binding(
        AxisFractionProperty::vertical()
            .read()
            .with(AppState::raw_scroll_pos),
    );

    let line_break_chooser = Flex::column()
        .with_child(Label::new("Line break mode"))
        .with_spacer(SPACER_SIZE)
        .with_child(RadioGroup::new(vec![
            ("Clip", LineBreaking::Clip),
            ("Wrap", LineBreaking::WordWrap),
            ("Overflow", LineBreaking::Overflow),
        ]))
        .lens(AppState::line_break_mode);

    let alignment_picker = Flex::column()
        .with_child(Label::new("Justification"))
        .with_spacer(SPACER_SIZE)
        .with_child(RadioGroup::new(vec![
            ("Start", TextAlignment::Start),
            ("End", TextAlignment::End),
            ("Center", TextAlignment::Center),
            ("Justified", TextAlignment::Justified),
        ]))
        .lens(AppState::alignment);

    let color_picker = Flex::column()
        .with_child(Label::new("Color"))
        .with_spacer(SPACER_SIZE)
        .with_child(RadioGroup::new(vec![
            ("Black", Color::BLACK),
            ("Blue", Color::BLUE),
            ("Red", Color::RED),
        ]))
        .lens(AppState::color);

    let controls = Flex::row()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_spacer(SPACER_SIZE)
        .with_child(alignment_picker)
        .with_spacer(SPACER_SIZE)
        .with_child(line_break_chooser)
        .with_spacer(SPACER_SIZE)
        .with_child(color_picker)
        .with_spacer(SPACER_SIZE)
        .with_child(
            Flex::column()
                .with_child(
                    Flex::row()
                        .with_child(Label::new(|idx: &usize, _env: &Env| {
                            format!("Current tab {}", idx)
                        }))
                        .with_flex_spacer(1.)
                        .with_child(
                            Stepper::new()
                                .with_range(0., 2.)
                                .lens(Map::new(|x| *x as f64, |x, y| *x = y as usize)),
                        )
                        .lens(AppState::tab_index)
                        .fix_width(130.),
                )
                .with_spacer(SPACER_SIZE)
                .with_child(Label::new("Label position"))
                .with_child(ProgressBar.lens(AppState::scroll_pos))
                .with_spacer(SPACER_SIZE)
                .with_child(Label::new("Raw label position"))
                .with_child(ProgressBar.lens(AppState::raw_scroll_pos)),
        )
        .padding(SPACER_SIZE);

    Flex::column()
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .with_child(controls)
        .with_flex_child(
            Tabs::new()
                .with_tab("Label", label)
                .with_tab("Raw label", raw_label)
                .with_tab_index(1)
                .binding(TabsProps::tab_index.with(AppState::tab_index)),
            1.0,
        )
}
