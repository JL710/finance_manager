use iced::advanced;

pub struct Scrollable<'a, Message> {
    content: iced::Element<'a, Message>,
    width: iced::Length,
    height: iced::Length,
    direction: super::advanced::Direction,
    class: <iced::Theme as super::Catalog>::Class<'a>,
    vertical_scrollbar_placement: super::Placement,
    horizontal_scrollbar_placement: super::Placement,
}

impl<'a, Message> Scrollable<'a, Message> {
    pub fn new(content: impl Into<iced::Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            width: iced::Shrink,
            height: iced::Shrink,
            direction: super::advanced::Direction::Vertical,
            class: <iced::Theme as super::Catalog>::default(),
            vertical_scrollbar_placement: super::Placement::default(),
            horizontal_scrollbar_placement: super::Placement::default(),
        }
    }
    pub fn vertical_scrollbar_placement(mut self, placement: super::Placement) -> Self {
        self.vertical_scrollbar_placement = placement;
        self
    }

    pub fn horizontal_scrollbar_placement(mut self, placement: super::Placement) -> Self {
        self.horizontal_scrollbar_placement = placement;
        self
    }
    pub fn width(mut self, width: iced::Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: iced::Length) -> Self {
        self.height = height;
        self
    }

    pub fn direction(mut self, direction: super::advanced::Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn style(mut self, style: impl Into<<iced::Theme as super::Catalog>::Class<'a>>) -> Self {
        self.class = style.into();
        self
    }
}

impl<Message> iced::advanced::Widget<Message, iced::Theme, iced::Renderer>
    for Scrollable<'_, Message>
{
    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![advanced::widget::Tree::new(&self.content)]
    }

    fn state(&self) -> advanced::widget::tree::State {
        advanced::widget::tree::State::Some(Box::new(super::advanced::State::new(
            self.direction,
            self.horizontal_scrollbar_placement,
            self.vertical_scrollbar_placement,
        )))
    }

    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size::new(self.width, self.height)
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        iced::Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        super::advanced::layout(self.direction, *limits, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, &limits)
        })
    }

    fn draw(
        &self,
        tree: &advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        super::advanced::draw(
            tree.state.downcast_ref(),
            &self.class,
            renderer,
            theme,
            cursor,
            layout.children().next().unwrap().bounds().size(),
            layout.bounds(),
            viewport,
            |renderer, viewport, cursor| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.children().next().unwrap(),
                    cursor,
                    viewport,
                )
            },
        );
    }

    fn diff(&self, tree: &mut advanced::widget::Tree) {
        if let advanced::widget::tree::State::Some(state) = &mut tree.state {
            let state: &mut super::advanced::State = state.downcast_mut().unwrap();
            state.direction(self.direction);
            state.horizontal_scrollbar_placement(self.horizontal_scrollbar_placement);
            state.vertical_scrollbar_placement(self.vertical_scrollbar_placement);
        } else {
            tree.state = self.state();
        }
        tree.diff_children(&[&self.content]);
    }

    fn operate(
        &self,
        tree: &mut advanced::widget::Tree,
        layout: advanced::layout::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        state: &mut advanced::widget::Tree,
        event: iced::Event,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> advanced::graphics::core::event::Status {
        [
            super::advanced::on_event(
                state.state.downcast_mut(),
                self.content.as_widget_mut(),
                layout.bounds().size(),
                layout.children().next().unwrap().bounds().size(),
                &mut state.children[0],
                event.clone(),
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ),
            super::advanced::scroll_grab_on_event(
                state.state.downcast_mut(),
                event.clone(),
                cursor,
                layout.bounds(),
                layout.children().next().unwrap().bounds().size(),
            ),
            super::advanced::scroll_wheel_on_event(
                state.state.downcast_mut(),
                event.clone(),
                cursor,
                layout.bounds(),
                layout.children().next().unwrap().bounds().size(),
            ),
        ]
        .into_iter()
        .fold(iced::event::Status::Ignored, iced::event::Status::merge)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut advanced::widget::Tree,
        layout: advanced::layout::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message> From<Scrollable<'a, Message>> for iced::Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: Scrollable<'a, Message>) -> Self {
        iced::Element::new(value)
    }
}
