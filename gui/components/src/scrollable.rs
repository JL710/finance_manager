use iced::advanced;
pub use iced::widget::scrollable::{Rail, Status};

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

#[derive(Default, Debug)]
pub struct State {
    scroll_x: f32,
    scroll_y: f32,
    scroll_space_x: f32,
    scroll_space_y: f32,
    mouse_grabbed_at_x: Option<f32>,
    mouse_grabbed_at_y: Option<f32>,
    keyboard_modifiers: iced::keyboard::Modifiers,
}

impl State {
    fn horizontal_scroll_factor(&self) -> f32 {
        0.0f32.max(self.scroll_x / self.scroll_space_x)
    }

    fn vertical_scroll_factor(&self) -> f32 {
        0.0f32.max(self.scroll_y / self.scroll_space_y)
    }

    fn scroll_by(&mut self, x: f32, y: f32) {
        self.set_absolute_position(self.scroll_x + x, self.scroll_y + y);
    }

    fn set_absolute_position(&mut self, x: f32, y: f32) {
        self.scroll_y = 0.0f32.max(self.scroll_space_y.min(y));
        self.scroll_x = 0.0f32.max(self.scroll_space_x.min(x));
    }

    fn set_relative_scroll(&mut self, x: f32, y: f32) {
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            panic!("relative scroll position can only be between 0.0 and 1.0")
        }
        self.scroll_x = self.scroll_space_x * x;
        self.scroll_y = self.scroll_space_y * y;
    }

    pub fn translation(&self) -> iced::Vector {
        iced::Vector::new(-self.scroll_x, -self.scroll_y)
    }
}

impl advanced::widget::operation::Scrollable for State {
    fn scroll_by(
        &mut self,
        offset: iced::widget::scrollable::AbsoluteOffset,
        _bounds: iced::Rectangle,
        _content_bounds: iced::Rectangle,
    ) {
        self.scroll_by(offset.x, offset.y);
    }

    fn scroll_to(&mut self, offset: iced::widget::scrollable::AbsoluteOffset) {
        self.set_absolute_position(offset.x, offset.y);
    }

    fn snap_to(&mut self, offset: iced::widget::scrollable::RelativeOffset) {
        self.set_relative_scroll(offset.x, offset.y);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw<Theme: Catalog, Renderer: advanced::Renderer>(
    state: &State,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &advanced::renderer::Style,
    cursor: advanced::mouse::Cursor,
    inner_size: iced::Size,
    outer_bounds: iced::Rectangle,
    viewport: &iced::Rectangle,
    draw_job: impl FnOnce(&mut Renderer, &iced::Rectangle, advanced::mouse::Cursor),
) {
    let Some(visible_bounds) = outer_bounds.intersection(viewport) else {
        // return if nothing would be visible on the screen
        return;
    };

    // draw inner content
    let translation = state.translation();
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

pub fn update_state(state: &mut State, scroll_space_x: f32, scroll_space_y: f32) {
    state.scroll_space_y = scroll_space_y;
    state.scroll_space_x = scroll_space_x;
}

pub fn mouse_interaction<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    widget: &dyn advanced::Widget<Message, Theme, Renderer>,
    tree: &iced::advanced::widget::Tree,
    layout: iced::advanced::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
    renderer: &Renderer,
) -> iced::advanced::mouse::Interaction {
    widget.mouse_interaction(
        tree,
        layout,
        if let iced::mouse::Cursor::Available(point) = cursor {
            iced::mouse::Cursor::Available(point + state.translation() * -1.0)
        } else {
            cursor
        },
        &(*viewport + state.translation() * -1.0),
        renderer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn on_event<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    widget: &mut dyn advanced::Widget<Message, Theme, Renderer>,
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
            iced::mouse::Cursor::Available(point + state.translation() * -1.0)
        } else {
            cursor
        },
        renderer,
        clipboard,
        shell,
        &(*viewport - state.translation()),
    )
}

pub fn scroll_wheel_on_event(
    state: &mut State,
    event: iced::Event,
    cursor: advanced::mouse::Cursor,
    bounds: iced::Rectangle,
) -> advanced::graphics::core::event::Status {
    if let Some(position) = cursor.position() {
        if !bounds.contains(position) {
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
            state.scroll_y = 0.0f32.max(state.scroll_space_y.min(state.scroll_y + y));
            state.scroll_x = 0.0f32.max(state.scroll_space_x.min(state.scroll_x + x));
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
    inner_size: iced::Size,
) -> advanced::graphics::core::event::Status {
    let scrollbar_bounds = scrollbar_bounds(
        bounds,
        inner_size.width > bounds.width,
        inner_size.height > bounds.height,
    );
    let scroller_bounds =
        scroller_bounds(state, inner_size, scrollbar_bounds.0, scrollbar_bounds.1);

    // mouse drag movement
    if state.mouse_grabbed_at_x.is_some() || state.mouse_grabbed_at_y.is_some() {
        if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) = event {
            if let Some(x) = state.mouse_grabbed_at_x {
                let diff = position.x - x;
                state.scroll_by(
                    state.scroll_space_x
                        * (diff / (scrollbar_bounds.0.width - scroller_bounds.0.width)),
                    0.0,
                );
                state.mouse_grabbed_at_x = Some(position.x);
                return advanced::graphics::core::event::Status::Captured;
            }
            if let Some(y) = state.mouse_grabbed_at_y {
                let diff = position.y - y;
                state.scroll_by(
                    0.0,
                    state.scroll_space_y
                        * (diff / (scrollbar_bounds.1.height - scroller_bounds.1.height)),
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
    inner_size: iced::Size,
    horizontal_scrollbar_bounds: iced::Rectangle,
    vertical_scrollbar_bounds: iced::Rectangle,
) -> (iced::Rectangle, iced::Rectangle) {
    (
        {
            // horizontal scroller
            let (scroller_start, scroller_end) = scroller_position(
                state.horizontal_scroll_factor(),
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
                state.vertical_scroll_factor(),
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
