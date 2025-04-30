pub use iced::widget::scrollable::{AbsoluteOffset, Rail, Status};
use iced::{Size, advanced, widget::scrollable::RelativeOffset};

pub struct Style {
    pub vertical_rail: Rail,
    pub horizontal_rail: Rail,
    pub gap: Option<iced::Background>,
}

static SCROLLBAR_THICKNESS: f32 = 10.0;

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme: &Self, status: Status| {
            let style = iced::widget::scrollable::default(theme, status);
            Style {
                vertical_rail: style.vertical_rail,
                horizontal_rail: style.horizontal_rail,
                gap: style.gap,
            }
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        (class)(self, status)
    }
}

#[derive(Debug, Clone)]
enum Offset {
    Relative(f32),
    Absolute(f32),
}

impl Default for Offset {
    fn default() -> Self {
        Self::Relative(0.0)
    }
}

impl Offset {
    fn relative(&self, outer_size: f32, inner_size: f32) -> f32 {
        match self {
            Offset::Relative(relative) => *relative,
            Offset::Absolute(absolute) => {
                if outer_size >= inner_size {
                    0.0
                } else {
                    absolute / (inner_size - outer_size)
                }
            }
        }
    }

    fn absolute(&self, outer_size: f32, inner_size: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => *absolute,
            Offset::Relative(relative) => relative * scroll_space(outer_size, inner_size),
        }
    }
}

fn scroll_space(outer_size: f32, inner_size: f32) -> f32 {
    0.0f32.max(inner_size - outer_size)
}

#[derive(Default, Debug)]
pub struct State {
    scroll_x: Offset,
    scroll_y: Offset,
    mouse_grabbed_at_x: Option<f32>,
    mouse_grabbed_at_y: Option<f32>,
    keyboard_modifiers: iced::keyboard::Modifiers,
}

impl State {
    fn horizontal_scroll_factor(&self, outer_size: f32, inner_size: f32) -> f32 {
        self.scroll_x.relative(outer_size, inner_size)
    }

    fn vertical_scroll_factor(&self, outer_size: f32, inner_size: f32) -> f32 {
        self.scroll_y.relative(outer_size, inner_size)
    }

    fn scroll_by(&mut self, offset: AbsoluteOffset, outer_size: Size, inner_size: Size) {
        self.scroll_to(
            AbsoluteOffset {
                x: self.scroll_x.absolute(outer_size.width, inner_size.width) + offset.x,
                y: self.scroll_y.absolute(outer_size.height, inner_size.height) + offset.y,
            },
            outer_size,
            inner_size,
        );
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset, outer_size: Size, inner_size: Size) {
        let scroll_space_x = scroll_space(outer_size.width, inner_size.width);
        let scroll_space_y = scroll_space(outer_size.height, inner_size.height);
        self.scroll_x = Offset::Absolute(offset.x.clamp(0.0, scroll_space_x));
        self.scroll_y = Offset::Absolute(offset.y.clamp(0.0, scroll_space_y));
    }

    fn snap_to(&mut self, offset: RelativeOffset) {
        self.scroll_x = Offset::Relative(offset.x);
        self.scroll_y = Offset::Relative(offset.y);
    }

    pub fn translation(&self, outer_size: Size, inner_size: Size) -> iced::Vector {
        let scroll_space_x = scroll_space(outer_size.width, inner_size.width);
        let scroll_space_y = scroll_space(outer_size.height, inner_size.height);
        let scroll_x =
            scroll_space_x.min(self.scroll_x.absolute(outer_size.width, inner_size.width));
        let scroll_y =
            scroll_space_y.min(self.scroll_y.absolute(outer_size.height, inner_size.height));
        iced::Vector::new(-scroll_x, -scroll_y)
    }
}

impl advanced::widget::operation::Scrollable for State {
    fn scroll_by(
        &mut self,
        offset: AbsoluteOffset,
        bounds: iced::Rectangle,
        content_bounds: iced::Rectangle,
    ) {
        self.scroll_by(offset, bounds.size(), content_bounds.size());
    }

    fn scroll_to(&mut self, _offset: AbsoluteOffset) {
        panic!(
            "This is not supported yet. Iced does not offer the bounds in this widget operation yet."
        );
    }

    fn snap_to(&mut self, offset: iced::widget::scrollable::RelativeOffset) {
        self.snap_to(offset);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw<Theme: Catalog, Renderer: advanced::Renderer>(
    state: &State,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &advanced::renderer::Style,
    cursor: advanced::mouse::Cursor,
    inner_size: Size,
    outer_bounds: iced::Rectangle,
    viewport: &iced::Rectangle,
    draw_job: impl FnOnce(&mut Renderer, &iced::Rectangle, advanced::mouse::Cursor),
) {
    let Some(visible_bounds) = outer_bounds.intersection(viewport) else {
        // return if nothing would be visible on the screen
        return;
    };

    // draw inner content
    let translation = state.translation(outer_bounds.size(), inner_size);
    renderer.with_layer(visible_bounds, |renderer| {
        renderer.with_translation(translation, |renderer| {
            (draw_job)(
                renderer,
                &iced::Rectangle {
                    x: viewport.x - translation.x,
                    y: viewport.y - translation.y,
                    ..*viewport
                },
                match cursor {
                    iced::mouse::Cursor::Available(point) => {
                        iced::mouse::Cursor::Available(point + translation * -1.0)
                    }
                    iced::mouse::Cursor::Unavailable => iced::mouse::Cursor::Unavailable,
                },
            )
        });
    });

    let horizontal_scrollbar = inner_size.width > outer_bounds.width;
    let vertical_scrollbar = inner_size.height > outer_bounds.height;

    let (horizontal_scrollbar_bounds, vertical_scrollbar_bounds) =
        scrollbar_bounds(outer_bounds, horizontal_scrollbar, vertical_scrollbar);
    let (horizontal_scroller_bounds, vertical_scroller_bounds) = scroller_bounds(
        state,
        inner_size,
        outer_bounds.size(),
        horizontal_scrollbar_bounds,
        vertical_scrollbar_bounds,
    );

    let style = theme.style(
        &<Theme as Catalog>::default(),
        calculate_status(
            vertical_scrollbar_bounds,
            horizontal_scrollbar_bounds,
            cursor,
            state.mouse_grabbed_at_x.is_some(),
            state.mouse_grabbed_at_y.is_some(),
        ),
    );

    renderer.with_layer(outer_bounds, |renderer| {
        if vertical_scrollbar && horizontal_scrollbar {
            renderer.fill_quad(
                advanced::renderer::Quad {
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    bounds: iced::Rectangle {
                        x: horizontal_scrollbar_bounds.width + horizontal_scrollbar_bounds.x,
                        y: vertical_scrollbar_bounds.height + vertical_scrollbar_bounds.y,
                        width: SCROLLBAR_THICKNESS,
                        height: SCROLLBAR_THICKNESS,
                    },
                },
                style
                    .gap
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
        }
        if horizontal_scrollbar {
            // draw horizontal scrollbar
            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: horizontal_scrollbar_bounds,
                    border: style.horizontal_rail.border,
                    shadow: iced::Shadow::default(),
                },
                style
                    .horizontal_rail
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
            // draw horizontal scroller
            renderer.fill_quad(
                advanced::renderer::Quad {
                    shadow: iced::Shadow::default(),
                    border: style.horizontal_rail.scroller.border,
                    bounds: horizontal_scroller_bounds,
                },
                style.horizontal_rail.scroller.color,
            );
        }
        if vertical_scrollbar {
            // draw vertical scrollbar
            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: vertical_scrollbar_bounds,
                    border: style.vertical_rail.border,
                    shadow: iced::Shadow::default(),
                },
                style
                    .horizontal_rail
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
            // draw vertical scroller
            renderer.fill_quad(
                advanced::renderer::Quad {
                    shadow: iced::Shadow::default(),
                    border: style.vertical_rail.scroller.border,
                    bounds: vertical_scroller_bounds,
                },
                style.vertical_rail.scroller.color,
            );
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn mouse_interaction<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    outer_bounds: iced::Rectangle,
    inner_size: Size,
    widget: &dyn advanced::Widget<Message, Theme, Renderer>,
    tree: &advanced::widget::Tree,
    layout: advanced::Layout<'_>,
    cursor: advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
    renderer: &Renderer,
) -> advanced::mouse::Interaction {
    if state.mouse_grabbed_at_x.is_some() || state.mouse_grabbed_at_y.is_some() {
        return advanced::mouse::Interaction::Idle;
    } else if let iced::mouse::Cursor::Available(position) = cursor {
        let scrollbar_bounds = scrollbar_bounds(
            outer_bounds,
            outer_bounds.height < inner_size.height,
            outer_bounds.width < inner_size.width,
        );
        let scroller_bounds = scroller_bounds(
            state,
            inner_size,
            outer_bounds.size(),
            scrollbar_bounds.0,
            scrollbar_bounds.1,
        );
        if scroller_bounds.0.contains(position) || scroller_bounds.1.contains(position) {
            return advanced::mouse::Interaction::Idle;
        }
    }

    widget.mouse_interaction(
        tree,
        layout,
        if let iced::mouse::Cursor::Available(point) = cursor {
            iced::mouse::Cursor::Available(
                point + state.translation(outer_bounds.size(), inner_size) * -1.0,
            )
        } else {
            cursor
        },
        &(*viewport + state.translation(outer_bounds.size(), inner_size) * -1.0),
        renderer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn on_event<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    widget: &mut dyn advanced::Widget<Message, Theme, Renderer>,
    outer_size: Size,
    inner_size: Size,
    tree: &mut advanced::widget::Tree,
    event: iced::Event,
    layout: advanced::Layout<'_>,
    cursor: advanced::mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn advanced::Clipboard,
    shell: &mut advanced::Shell<'_, Message>,
    viewport: &iced::Rectangle,
) -> advanced::graphics::core::event::Status {
    widget.on_event(
        tree,
        event,
        layout,
        if let iced::mouse::Cursor::Available(point) = cursor {
            iced::mouse::Cursor::Available(point + state.translation(outer_size, inner_size) * -1.0)
        } else {
            cursor
        },
        renderer,
        clipboard,
        shell,
        &(*viewport - state.translation(outer_size, inner_size)),
    )
}

pub fn scroll_wheel_on_event(
    state: &mut State,
    event: iced::Event,
    cursor: advanced::mouse::Cursor,
    outer_bounds: iced::Rectangle,
    inner_size: Size,
) -> advanced::graphics::core::event::Status {
    if let Some(position) = cursor.position() {
        if !outer_bounds.contains(position) {
            return advanced::graphics::core::event::Status::Ignored;
        }
    } else {
        return advanced::graphics::core::event::Status::Ignored;
    };
    match event {
        iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Lines { mut y, mut x },
        }) => {
            if state.keyboard_modifiers.shift() {
                (x, y) = (y, x);
            }
            y *= -30.0;
            x *= -30.0;
            state.scroll_by(AbsoluteOffset { x, y }, outer_bounds.size(), inner_size);
            return advanced::graphics::core::event::Status::Captured;
        }
        iced::Event::Keyboard(keyboard_event) => {
            state.keyboard_modifiers = match keyboard_event {
                iced::keyboard::Event::KeyPressed { modifiers, .. } => modifiers,
                iced::keyboard::Event::KeyReleased { modifiers, .. } => modifiers,
                iced::keyboard::Event::ModifiersChanged(modifiers) => modifiers,
            };
        }
        _ => {}
    }
    advanced::graphics::core::event::Status::Ignored
}

pub fn scroll_grab_on_event(
    state: &mut State,
    event: iced::Event,
    cursor: advanced::mouse::Cursor,
    bounds: iced::Rectangle,
    inner_size: Size,
) -> advanced::graphics::core::event::Status {
    let scrollbar_bounds = scrollbar_bounds(
        bounds,
        inner_size.width > bounds.width,
        inner_size.height > bounds.height,
    );
    let scroller_bounds = scroller_bounds(
        state,
        inner_size,
        bounds.size(),
        scrollbar_bounds.0,
        scrollbar_bounds.1,
    );

    // mouse drag movement
    if state.mouse_grabbed_at_x.is_some() || state.mouse_grabbed_at_y.is_some() {
        if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) = event {
            if let Some(x) = state.mouse_grabbed_at_x {
                let diff = position.x - x;

                state.scroll_by(
                    AbsoluteOffset {
                        x: scroll_space(bounds.size().width, inner_size.width)
                            * (diff / (scrollbar_bounds.0.width - scroller_bounds.0.width)),
                        y: 0.0,
                    },
                    bounds.size(),
                    inner_size,
                );
                state.mouse_grabbed_at_x = Some(position.x);
                return advanced::graphics::core::event::Status::Captured;
            }
            if let Some(y) = state.mouse_grabbed_at_y {
                let diff = position.y - y;

                state.scroll_by(
                    AbsoluteOffset {
                        x: 0.0,
                        y: scroll_space(bounds.height, inner_size.height)
                            * (diff / (scrollbar_bounds.1.height - scroller_bounds.1.height)),
                    },
                    bounds.size(),
                    inner_size,
                );
                state.mouse_grabbed_at_y = Some(position.y);
                return advanced::graphics::core::event::Status::Captured;
            }
        }
    }

    if let iced::Event::Mouse(iced::mouse::Event::ButtonReleased(button)) = event {
        if button == iced::mouse::Button::Left {
            state.mouse_grabbed_at_x = None;
            state.mouse_grabbed_at_y = None;
        }
    }

    let mouse_position = if let Some(position) = cursor.position() {
        if !bounds.contains(position) {
            return advanced::graphics::core::event::Status::Ignored;
        }
        position
    } else {
        return advanced::graphics::core::event::Status::Ignored;
    };
    if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(button)) = event {
        if button == iced::mouse::Button::Left {
            if scroller_bounds.0.contains(mouse_position) {
                state.mouse_grabbed_at_x = Some(mouse_position.x);
                return advanced::graphics::core::event::Status::Captured;
            } else if scroller_bounds.1.contains(mouse_position) {
                state.mouse_grabbed_at_y = Some(mouse_position.y);
                return advanced::graphics::core::event::Status::Captured;
            }
        }
    }
    advanced::graphics::core::event::Status::Ignored
}

fn calculate_status(
    vertical_scrollbar_bounds: iced::Rectangle,
    horizontal_scrollbar_bounds: iced::Rectangle,
    cursor: iced::mouse::Cursor,
    mouse_drag_x: bool,
    mouse_drag_y: bool,
) -> Status {
    if let iced::mouse::Cursor::Available(position) = cursor {
        if mouse_drag_y {
            return Status::Dragged {
                is_horizontal_scrollbar_dragged: false,
                is_vertical_scrollbar_dragged: true,
            };
        } else if mouse_drag_x {
            return Status::Dragged {
                is_horizontal_scrollbar_dragged: true,
                is_vertical_scrollbar_dragged: false,
            };
        }
        if vertical_scrollbar_bounds.contains(position) {
            return Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
            };
        } else if horizontal_scrollbar_bounds.contains(position) {
            return Status::Hovered {
                is_horizontal_scrollbar_hovered: true,
                is_vertical_scrollbar_hovered: false,
            };
        }
    }
    Status::Active
}

fn scrollbar_bounds(
    bounds: iced::Rectangle,
    vertical_scrollbar: bool,
    horizontal_scrollbar: bool,
) -> (iced::Rectangle, iced::Rectangle) {
    (
        iced::Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: bounds.width
                + if vertical_scrollbar {
                    -SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            height: SCROLLBAR_THICKNESS,
        },
        iced::Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: bounds.y,
            width: SCROLLBAR_THICKNESS,
            height: bounds.height
                + if horizontal_scrollbar {
                    -SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
        },
    )
}

fn scroller_bounds(
    state: &State,
    inner_size: Size,
    outer_size: Size,
    horizontal_scrollbar_bounds: iced::Rectangle,
    vertical_scrollbar_bounds: iced::Rectangle,
) -> (iced::Rectangle, iced::Rectangle) {
    (
        {
            // horizontal scroller
            let (scroller_start, scroller_end) = scroller_position(
                state.horizontal_scroll_factor(outer_size.width, inner_size.width),
                horizontal_scrollbar_bounds.x,
                horizontal_scrollbar_bounds.x + horizontal_scrollbar_bounds.width,
                inner_size.width,
            );
            iced::Rectangle {
                height: horizontal_scrollbar_bounds.height,
                width: scroller_end - scroller_start,
                y: horizontal_scrollbar_bounds.y,
                x: scroller_start,
            }
        },
        {
            // vertical scroller
            let (scroller_start, scroller_end) = scroller_position(
                state.vertical_scroll_factor(outer_size.height, inner_size.height),
                vertical_scrollbar_bounds.y,
                vertical_scrollbar_bounds.y + vertical_scrollbar_bounds.height,
                inner_size.height,
            );
            iced::Rectangle {
                height: scroller_end - scroller_start,
                width: vertical_scrollbar_bounds.width,
                y: scroller_start,
                x: vertical_scrollbar_bounds.x,
            }
        },
    )
}

fn scroller_position(factor: f32, start: f32, end: f32, inner_size: f32) -> (f32, f32) {
    let outer_size = end - start;
    let size = 30.0f32.max(outer_size * 1.0f32.min(outer_size / inner_size));

    let start_pos = start + (outer_size - size) * factor;
    (start_pos, start_pos + size)
}

pub fn x_start_end(layouts: &Vec<advanced::Layout<'_>>) -> (f32, f32) {
    let mut layout_iter = layouts.iter();
    let (mut start, mut end) = if let Some(first) = layout_iter.next() {
        (first.position().x, first.position().x)
    } else {
        return (0.0, 0.0);
    };
    for layout in layout_iter {
        start = start.min(layout.position().x);
        end = end.max(layout.position().x + layout.bounds().width);
    }
    (start, end)
}

pub fn y_start_end(layouts: &Vec<advanced::Layout<'_>>) -> (f32, f32) {
    let mut layout_iter = layouts.iter();
    let (mut start, mut end) = if let Some(first) = layout_iter.next() {
        (first.position().y, first.position().y)
    } else {
        return (0.0, 0.0);
    };
    for layout in layout_iter {
        start = start.min(layout.position().y);
        end = end.max(layout.position().y + layout.bounds().height);
    }
    (start, end)
}
