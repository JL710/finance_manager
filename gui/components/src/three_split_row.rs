pub fn three_split_row<'a, Message: 'a>(
    left: impl Into<iced::Element<'a, Message>>,
    middle: impl Into<iced::Element<'a, Message>>,
    right: impl Into<iced::Element<'a, Message>>,
    alignment: iced::alignment::Vertical,
) -> iced::Element<'a, Message> {
    crate::custom_layout::CustomLayout::new(
        vec![left.into(), middle.into(), right.into()],
        move |elements, states, renderer, limits| {
            let mut middle_layout =
                elements[1]
                    .as_widget()
                    .layout(&mut states[1], renderer, limits);
            let side_limits = iced::advanced::layout::Limits::new(
                (limits.min() - middle_layout.size()) * 0.5,
                (limits.max() - middle_layout.size()) * 0.5,
            );
            let mut left_layout =
                elements[0]
                    .as_widget()
                    .layout(&mut states[0], renderer, &side_limits);
            let mut right_layout =
                elements[2]
                    .as_widget()
                    .layout(&mut states[2], renderer, &side_limits);

            let height = middle_layout
                .size()
                .height
                .max(left_layout.size().height)
                .max(right_layout.size().height);

            left_layout = left_layout.clone().move_to((
                0.0,
                align_position(height, left_layout.size().height, alignment),
            ));
            middle_layout = middle_layout.clone().move_to((
                align_position(
                    limits.max().width,
                    middle_layout.size().width,
                    iced::Alignment::Center,
                ),
                align_position(height, middle_layout.size().height, alignment),
            ));
            right_layout = right_layout.clone().move_to((
                align_position(
                    limits.max().width,
                    right_layout.size().width,
                    iced::Alignment::End,
                ),
                align_position(height, right_layout.size().height, alignment),
            ));

            iced::advanced::layout::Node::with_children(
                iced::Size::new(limits.max().width, height),
                vec![left_layout, middle_layout, right_layout],
            )
        },
    )
    .width(iced::Fill)
    .into()
}

fn align_position(limit: f32, size: f32, alignment: impl Into<iced::Alignment>) -> f32 {
    match alignment.into() {
        iced::Alignment::Start => 0.0,
        iced::Alignment::Center => (limit - size) / 2.0,
        iced::Alignment::End => limit - size,
    }
}
