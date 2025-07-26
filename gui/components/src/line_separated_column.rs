pub struct LineSeparatedColumn<'a, Message, Theme: iced::widget::rule::Catalog, Renderer> {
    elements: Vec<iced::Element<'a, Message, Theme, Renderer>>,
    width: iced::Length,
    height: iced::Length,
    rule_size: f32,
    spacing: f32,
    alignment: iced::alignment::Horizontal,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Default for LineSeparatedColumn<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::rule::Catalog,
{
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            width: iced::Shrink,
            height: iced::Shrink,
            rule_size: 5.0,
            spacing: 0.0,
            alignment: iced::alignment::Horizontal::Left,
            class: Theme::default(),
        }
    }
}

impl<'a, Message, Theme, Renderer> LineSeparatedColumn<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::rule::Catalog,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, width: impl Into<iced::Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<iced::Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn align_x(mut self, alignment: impl Into<iced::alignment::Horizontal>) -> Self {
        self.alignment = alignment.into();
        self
    }

    pub fn spacing(mut self, spacing: impl Into<f32>) -> Self {
        self.spacing = spacing.into();
        self
    }

    pub fn push(mut self, element: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
        self.elements.push(element.into());
        self
    }

    pub fn push_maybe(
        mut self,
        element: Option<impl Into<iced::Element<'a, Message, Theme, Renderer>>>,
    ) -> Self {
        if let Some(element) = element {
            self.elements.push(element.into());
        }
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> iced::widget::rule::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<iced::widget::rule::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as iced::widget::rule::StyleFn<'a, Theme>).into();
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message: 'a, Theme, Renderer> From<LineSeparatedColumn<'a, Message, Theme, Renderer>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::rule::Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(mut value: LineSeparatedColumn<'a, Message, Theme, Renderer>) -> Self {
        let mut elements = Vec::new();
        let rule_number = (elements.len() as isize - 1).min(0) as usize;
        while !value.elements.is_empty() {
            elements.push(value.elements.remove(0));
            if !value.elements.is_empty() {
                elements.push(iced::widget::horizontal_rule(value.rule_size).into());
            }
        }
        iced_aw::widgets::CustomLayout::new(elements, move |elements, states, renderer, limits| {
            let rule_height = if elements.len() > 1 {
                elements[1]
                    .as_widget()
                    .layout(&mut states[1], renderer, limits)
                    .size()
                    .height
            } else {
                0.0
            };
            let mut height = rule_height * rule_number as f32;
            let mut nodes = Vec::new();
            let mut total_portions = 0;
            // initial iteration for non Fill elements
            for (i, element) in elements.iter().enumerate() {
                nodes.push(if !i.is_multiple_of(2) {
                    None
                } else if element.as_widget().size_hint().height.fluid() == iced::Length::Shrink {
                    let node = element.as_widget().layout(
                        &mut states[i],
                        renderer,
                        &limits.shrink(iced::Size::new(0.0, height)),
                    );
                    height += node.size().height;
                    Some(node)
                } else {
                    total_portions += element.as_widget().size().height.fill_factor();
                    None
                });
            }
            // iteration for Fill elements
            let portion_size = (limits.max().height - height) / total_portions as f32;
            for (i, element) in elements.iter().enumerate() {
                if i.is_multiple_of(2)
                    && element.as_widget().size_hint().height.fluid() == iced::Length::Fill
                {
                    nodes[i] = Some(element.as_widget().layout(
                        &mut states[i],
                        renderer,
                        &limits.max_height(
                            portion_size * element.as_widget().size().height.fill_factor() as f32,
                        ),
                    ));
                }
            }

            let mut max_width: f32 = 0.0;
            for node in nodes.iter_mut().flatten() {
                max_width = max_width.max(node.size().width);
            }

            // rule layouts
            if rule_number > 0 {
                max_width = limits
                    .resolve(value.width, value.height, iced::Size::new(max_width, 0.0))
                    .width;
                let rule_node = elements[1].as_widget().layout(
                    &mut states[1],
                    renderer,
                    &iced::advanced::layout::Limits::new(
                        iced::Size::ZERO,
                        iced::Size::new(max_width, rule_height),
                    ),
                );
                nodes.iter_mut().for_each(|x| {
                    if x.is_none() {
                        *x = Some(rule_node.clone())
                    }
                });
            }

            let mut nodes = nodes.into_iter().map(|x| x.unwrap()).collect::<Vec<_>>();

            // place items
            let mut height = 0.0;
            for node in &mut nodes {
                node.move_to_mut((
                    crate::align_position(max_width, node.size().width, value.alignment),
                    height,
                ));
                height += node.size().height;
            }

            iced::advanced::layout::Node::with_children(
                limits.resolve(
                    value.width,
                    value.height,
                    iced::Size::new(max_width, height),
                ),
                nodes,
            )
        })
        .width(value.width)
        .height(value.height)
        .into()
    }
}
